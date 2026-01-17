//! Main Iced Application - Using Iced 0.13 Program API

use iced::{
    Element, Task, Subscription,
    widget::{button, column, container, row, text, horizontal_rule, Space},
    Length, Alignment,
    time,
    keyboard,
};
use std::time::Duration;

use crate::message::Message;
use crate::state::AppState;
use crate::theme;
use crate::widgets::{editor, preview, timeline, ktiming, settings, inspector};

/// Update function - handles all application messages
pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::ToggleSettings => {
            state.show_settings = !state.show_settings;
            // If opening, ensure fonts are scanned if they haven't been
            if state.show_settings && state.available_fonts.is_empty() && !state.font_scan_complete {
                 return Task::perform(
                    async {
                        tokio::task::spawn_blocking(crate::utils::font_loader::scan_system_fonts).await.unwrap()
                    },
                    Message::FontScanComplete
                );
            }
        }

        Message::FontScanComplete(fonts) => {
            state.available_fonts = fonts;
            state.font_scan_complete = true;
            
            // If we have a configured font, try to load it now that we have paths
            if let Some(font_name) = &state.config.ui_font {
                if let Some(info) = state.available_fonts.iter().find(|f| &f.name == font_name) {
                    let path = info.path.clone();
                    return Task::perform(
                        async move {
                            std::fs::read(&path).map_err(|e| e.to_string())
                        },
                        Message::FontBytesLoaded,
                    );
                }
            }
        }

        Message::SelectUiFont(info) => {
            state.config.ui_font = Some(info.name.clone());
            if let Err(e) = state.config.save() {
                log::error!("Failed to save config: {}", e);
            }
            
            let path = info.path.clone();
            return Task::perform(
                async move {
                    std::fs::read(&path).map_err(|e| e.to_string())
                },
                Message::FontBytesLoaded,
            );
        }
        
        Message::FontBytesLoaded(res) => {
            match res {
                Ok(bytes) => {
                     return iced::font::load(std::borrow::Cow::Owned(bytes)).map(|_| Message::NoOp);
                }
                Err(e) => {
                    log::error!("Failed to load font bytes: {}", e);
                }
            }
        }
        
        Message::NoOp => {}

        
        Message::ToggleShowChineseOnly(val) => {
            state.config.show_chinese_only = val;
            if let Err(e) = state.config.save() {
                log::error!("Failed to save config: {}", e);
            }
        }

        Message::MainWindowOpened(id) => {
            state.main_window = Some(id);
        }
        
        Message::OpenDebugWindow => {
            if state.debug_window.is_some() {
                return Task::none();
            }
            
            let (_, task) = iced::window::open(iced::window::Settings {
               size: iced::Size::new(600.0, 800.0),
               ..Default::default() 
            });
            
            return task.map(Message::DebugWindowOpened);
        }
        
        Message::DebugWindowOpened(id) => {
            state.debug_window = Some(id);
        }
        
        Message::DebugWindowClosed(id) => {
             if state.debug_window == Some(id) {
                 state.debug_window = None;
             }
        }

        Message::OpenFile => {
            return Task::perform(
                async {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("All Supported", &["klyric", "json", "ass", "ssa", "srt", "lrc"])
                        .add_filter("KLyric Project", &["klyric", "json"])
                        .add_filter("Subtitle", &["ass", "ssa", "srt", "lrc"])
                        .pick_file()
                        .await;
                    file.map(|f| f.path().to_path_buf())
                },
                Message::FileOpened,
            );
        }
        
        Message::FileOpened(Some(path)) => {
            let path_clone = path.clone();
            
            // Only set file_path if it's a project file (klyric/json)
            // Otherwise treat it as an import (new project)
            let is_project_file = path.extension()
                .and_then(|e| e.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("klyric") || ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false);

            if is_project_file {
                state.file_path = Some(path);
            } else {
                state.file_path = None;
            }

            return Task::perform(
                async move {
                    match std::fs::read_to_string(&path_clone) {
                        Ok(content) => {
                             let filename = path_clone.file_name().and_then(|s| s.to_str());
                             klyric_renderer::importer::import_subtitle(&content, filename)
                                 .map_err(|e| e.to_string())
                        }

                        Err(e) => Err(e.to_string()),
                    }
                },
                Message::DocumentLoaded,
            );
        }
        
        Message::FileOpened(None) => {}
        
        Message::DocumentLoaded(Ok(doc)) => {
            // Update duration from document
            state.playback.duration = doc.project.duration;
            
            // Load audio if present
            if let Some(audio_path) = &doc.project.audio {
                if let Some(am) = &mut state.audio_manager {
                    if let Err(e) = am.load(audio_path) {
                        log::error!("Failed to load audio: {}", e);
                    }
                }
            } else {
                 if let Some(am) = &mut state.audio_manager {
                     am.stop();
                 }
            }

            state.document = Some(doc);
            state.is_dirty = false;
            state.selected_line = Some(0);
            state.selected_char = None;
            // state.renderer = None; // Field removed
            update_preview(state);
        }
        
        Message::DocumentLoaded(Err(e)) => {
            log::error!("Failed to load document: {}", e);
        }
        
        Message::SaveFile => {
            if let Some(ref doc) = state.document {
                let json = serde_json::to_string_pretty(doc).unwrap_or_default();
                let existing_path = state.file_path.clone();
                
                return Task::perform(
                    async move {
                        let path = if let Some(p) = existing_path {
                            Some(p)
                        } else {
                            rfd::AsyncFileDialog::new()
                                .add_filter("KLyric", &["klyric"])
                                .save_file()
                                .await
                                .map(|f| f.path().to_path_buf())
                        };
                        
                        if let Some(p) = path {
                            std::fs::write(&p, json)
                                .map(|_| p)
                                .map_err(|e| e.to_string())
                        } else {
                            Err("No file selected".to_string())
                        }
                    },
                    Message::FileSaved,
                );
            }
        }
        
        Message::FileSaved(Ok(path)) => {
            state.file_path = Some(path);
            state.is_dirty = false;
        }
        
        Message::FileSaved(Err(e)) => {
            log::error!("Failed to save: {}", e);
        }
        
        Message::SelectLine(idx) => {
            state.selected_line = Some(idx);
            state.selected_char = None;
        }
        
        Message::SelectChar(idx) => {
            state.selected_char = Some(idx);
        }
        
        Message::AddLine => {
            if let Some(ref mut doc) = state.document {
                let new_line = klyric_renderer::model::Line::default();
                doc.lines.push(new_line);
                state.selected_line = Some(doc.lines.len() - 1);
                state.is_dirty = true;
            }
        }
        
        Message::DeleteLine(idx) => {
            if let Some(ref mut doc) = state.document {
                if idx < doc.lines.len() {
                    doc.lines.remove(idx);
                    state.selected_line = None;
                    state.is_dirty = true;
                }
            }
        }
        
        Message::Play => {
            log::info!("Starting playback. Duration: {}", state.playback.duration);
            state.playback.is_playing = true;
            if let Some(am) = &state.audio_manager {
                am.play();
            }
        }
        
        Message::Pause => {
            log::info!("Pausing playback.");
            state.playback.is_playing = false;
            if let Some(am) = &state.audio_manager {
                am.pause();
            }
        }
        
        Message::Stop => {
            log::info!("Stopping playback.");
            state.playback.is_playing = false;
            state.playback.current_time = 0.0;
            if let Some(am) = &state.audio_manager {
                am.stop();
            }
            update_preview(state);
        }
        
        Message::Seek(time) => {
            let clamped = time.clamp(0.0, state.playback.duration);
            state.playback.current_time = clamped;
            if let Some(am) = &state.audio_manager {
                if let Err(e) = am.seek(Duration::from_secs_f64(clamped)) {
                     log::error!("Seek failed: {}", e);
                }
            }
            update_preview(state);
        }
        
        Message::Tick => {
            // Poll worker for updates
            if let Some(conn) = &state.worker_connection {
                loop {
                    match conn.try_recv() {
                        Ok(crate::worker::RenderingResponse::FrameRendered(handle)) => {
                            state.preview_handle = Some(handle);
                        }
                        Ok(crate::worker::RenderingResponse::Error(_e)) => {
                             // Limit error logs to avoid spam
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                             log::error!("Preview worker disconnected unexpectedly");
                             break;
                        }
                    }
                }
            }

            if state.playback.is_playing {
                state.playback.current_time += 1.0 / 60.0; // ~60fps
                
                if state.playback.current_time >= state.playback.duration {
                    log::info!("Playback reached end of duration: {}", state.playback.duration);
                    state.playback.current_time = 0.0;
                }
                update_preview(state);
            } else if state.is_dirty && state.show_preview {
                // Real-time update for property changes when paused
                update_preview(state);
                state.is_dirty = false;
            }
        }
        
        Message::SetCharStart(val) => {
            if let Ok(v) = val.parse::<f64>() {
                if let Some(ch) = state.current_char_mut() {
                    ch.start = v;
                    state.is_dirty = true;
                }
            }
        }
        
        Message::SetCharEnd(val) => {
            if let Ok(v) = val.parse::<f64>() {
                if let Some(ch) = state.current_char_mut() {
                    ch.end = v;
                    state.is_dirty = true;
                }
            }
        }
        
        // Transform edits
        Message::SetOffsetX(v) => {
            if let Some(ch) = state.current_char_mut() {
                let t = ch.transform.get_or_insert_with(Default::default);
                t.x = Some(v);
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                let t = line.transform.get_or_insert_with(Default::default);
                t.x = Some(v);
                state.is_dirty = true;
            }
        }
        Message::UnsetOffsetX => {
            if let Some(ch) = state.current_char_mut() {
                if let Some(t) = ch.transform.as_mut() { t.x = None; }
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                if let Some(t) = line.transform.as_mut() { t.x = None; }
                state.is_dirty = true;
            }
        }
        
        Message::SetOffsetY(v) => {
            if let Some(ch) = state.current_char_mut() {
                let t = ch.transform.get_or_insert_with(Default::default);
                t.y = Some(v);
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                let t = line.transform.get_or_insert_with(Default::default);
                t.y = Some(v);
                state.is_dirty = true;
            }
        }
        Message::UnsetOffsetY => {
            if let Some(ch) = state.current_char_mut() {
                if let Some(t) = ch.transform.as_mut() { t.y = None; }
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                if let Some(t) = line.transform.as_mut() { t.y = None; }
                state.is_dirty = true;
            }
        }
        
        Message::SetRotation(v) => {
            if let Some(ch) = state.current_char_mut() {
                let t = ch.transform.get_or_insert_with(Default::default);
                t.rotation = Some(v);
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                let t = line.transform.get_or_insert_with(Default::default);
                t.rotation = Some(v);
                state.is_dirty = true;
            }
        }
        Message::UnsetRotation => {
            if let Some(ch) = state.current_char_mut() {
                if let Some(t) = ch.transform.as_mut() { t.rotation = None; }
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                if let Some(t) = line.transform.as_mut() { t.rotation = None; }
                state.is_dirty = true;
            }
        }
        
        Message::SetScale(v) => {
            if let Some(ch) = state.current_char_mut() {
                let t = ch.transform.get_or_insert_with(Default::default);
                t.scale = Some(v);
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                let t = line.transform.get_or_insert_with(Default::default);
                t.scale = Some(v);
                state.is_dirty = true;
            }
        }
        Message::UnsetScale => {
            if let Some(ch) = state.current_char_mut() {
                if let Some(t) = ch.transform.as_mut() { t.scale = None; }
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                if let Some(t) = line.transform.as_mut() { t.scale = None; }
                state.is_dirty = true;
            }
        }
        
        Message::SetOpacity(v) => {
            if let Some(ch) = state.current_char_mut() {
                let t = ch.transform.get_or_insert_with(Default::default);
                t.opacity = Some(v);
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                let t = line.transform.get_or_insert_with(Default::default);
                t.opacity = Some(v);
                state.is_dirty = true;
            }
        }
        Message::UnsetOpacity => {
            if let Some(ch) = state.current_char_mut() {
                if let Some(t) = ch.transform.as_mut() { t.opacity = None; }
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                if let Some(t) = line.transform.as_mut() { t.opacity = None; }
                state.is_dirty = true;
            }
        }
        
        // Style edits
        Message::SetFontFamily(val) => {
            if let Some(ch) = state.current_char_mut() {
                let f = ch.font.get_or_insert_with(Default::default);
                f.family = Some(val);
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                let f = line.font.get_or_insert_with(Default::default);
                f.family = Some(val);
                state.is_dirty = true;
            } else {
                 if let Some(doc) = &mut state.document {
                    let style = doc.styles.entry("base".to_string()).or_default();
                    let f = style.font.get_or_insert_with(Default::default);
                    f.family = Some(val);
                    state.is_dirty = true;
                 }
            }
        }
        Message::UnsetFontFamily => {
             if let Some(ch) = state.current_char_mut() {
                if let Some(f) = ch.font.as_mut() { f.family = None; }
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                if let Some(f) = line.font.as_mut() { f.family = None; }
                state.is_dirty = true;
            }
        }

        Message::SetFontSize(val) => {
             if let Ok(num) = val.parse::<f32>() {
                if let Some(ch) = state.current_char_mut() {
                    let f = ch.font.get_or_insert_with(Default::default);
                    f.size = Some(num);
                    state.is_dirty = true;
                } else if let Some(line) = state.current_line_mut() {
                    let f = line.font.get_or_insert_with(Default::default);
                    f.size = Some(num);
                    state.is_dirty = true;
                } else {
                    if let Some(doc) = &mut state.document {
                        let style = doc.styles.entry("base".to_string()).or_default();
                        let f = style.font.get_or_insert_with(Default::default);
                        f.size = Some(num);
                        state.is_dirty = true;
                    }
                }
             }
        }
        Message::UnsetFontSize => {
             if let Some(ch) = state.current_char_mut() {
                if let Some(f) = ch.font.as_mut() { f.size = None; }
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                if let Some(f) = line.font.as_mut() { f.size = None; }
                state.is_dirty = true;
            }
        }

        Message::SetFillColor(_val) => {
            state.is_dirty = true; 
        }
        Message::UnsetFillColor => {
            state.is_dirty = true;
        }

        Message::SetStrokeColor(val) => {
            if let Some(ch) = state.current_char_mut() {
                let stroke = ch.stroke.get_or_insert_with(Default::default);
                stroke.color = Some(val);
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                let stroke = line.stroke.get_or_insert_with(Default::default);
                stroke.color = Some(val);
                state.is_dirty = true;
            }
        }
        Message::UnsetStrokeColor => {
             if let Some(ch) = state.current_char_mut() {
                if let Some(s) = ch.stroke.as_mut() { s.color = None; }
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                if let Some(s) = line.stroke.as_mut() { s.color = None; }
                state.is_dirty = true;
            }
        }

        Message::SetStrokeWidth(val) => {
             if let Ok(num) = val.parse::<f32>() {
                if let Some(ch) = state.current_char_mut() {
                     let s = ch.stroke.get_or_insert_with(Default::default);
                     s.width = Some(num);
                     state.is_dirty = true;
                } else if let Some(line) = state.current_line_mut() {
                    let s = line.stroke.get_or_insert_with(Default::default);
                    s.width = Some(num);
                    state.is_dirty = true;
                }
             }
        }
        Message::UnsetStrokeWidth => {
             if let Some(ch) = state.current_char_mut() {
                if let Some(s) = ch.stroke.as_mut() { s.width = None; }
                state.is_dirty = true;
            } else if let Some(line) = state.current_line_mut() {
                if let Some(s) = line.stroke.as_mut() { s.width = None; }
                state.is_dirty = true;
            }
        }
        
        // ===== K-Timing Messages =====
        Message::MarkSyllable => {
            let current_time = state.playback.current_time;
            
            if let Some(char_idx) = state.selected_char {
                if let Some(line) = state.current_line_mut() {
                    if let Some(ch) = line.chars.get_mut(char_idx) {
                        ch.end = current_time;
                    }
                    
                    let next_idx = char_idx + 1;
                    if next_idx < line.chars.len() {
                        if let Some(next_ch) = line.chars.get_mut(next_idx) {
                            next_ch.start = current_time;
                        }
                        state.selected_char = Some(next_idx);
                    }
                    
                    state.is_dirty = true;
                }
            } else {
                if let Some(line) = state.current_line_mut() {
                    if !line.chars.is_empty() {
                        if let Some(ch) = line.chars.get_mut(0) {
                            ch.start = current_time;
                        }
                        state.selected_char = Some(0);
                        state.is_dirty = true;
                    }
                }
            }
        }
        
        Message::AdvanceChar => {
            if let (Some(line_idx), Some(char_idx)) = (state.selected_line, state.selected_char) {
                if let Some(doc) = &state.document {
                    if let Some(line) = doc.lines.get(line_idx) {
                        if char_idx + 1 < line.chars.len() {
                            state.selected_char = Some(char_idx + 1);
                        }
                    }
                }
            } else if state.selected_line.is_some() {
                state.selected_char = Some(0);
            }
        }
        
        Message::RetreatChar => {
            if let Some(char_idx) = state.selected_char {
                if char_idx > 0 {
                    state.selected_char = Some(char_idx - 1);
                }
            }
        }
        
        Message::ResetLineTiming => {
            if let Some(line) = state.current_line_mut() {
                let line_start = line.start;
                let line_end = line.end;
                let num_chars = line.chars.len();
                
                if num_chars > 0 {
                    let duration_per_char = (line_end - line_start) / num_chars as f64;
                    
                    for (i, ch) in line.chars.iter_mut().enumerate() {
                        ch.start = line_start + (i as f64 * duration_per_char);
                        ch.end = line_start + ((i + 1) as f64 * duration_per_char);
                    }
                    
                    state.is_dirty = true;
                }
            }
            state.selected_char = Some(0);
        }
        
        // ===== Global Style Messages =====
        Message::SelectGlobal => {
            state.selected_line = None;
            state.selected_char = None;
        }

        Message::SetGlobalFont(family) => {
            if let Some(doc) = &mut state.document {
                let style = doc.styles.entry("base".to_string()).or_default();
                let f = style.font.get_or_insert_with(Default::default);
                f.family = Some(family);
                state.is_dirty = true;
            }
        }
        Message::UnsetGlobalFont => {
             if let Some(doc) = &mut state.document {
                let style = doc.styles.entry("base".to_string()).or_default();
                if let Some(f) = style.font.as_mut() { f.family = None; }
                state.is_dirty = true;
            }
        }

        Message::SetGlobalFontSize(val) => {
            if let Ok(size) = val.parse::<f32>() {
                if let Some(doc) = &mut state.document {
                    let style = doc.styles.entry("base".to_string()).or_default();
                    let f = style.font.get_or_insert_with(Default::default);
                    f.size = Some(size);
                    state.is_dirty = true;
                }
            }
        }
        Message::UnsetGlobalFontSize => {
             if let Some(doc) = &mut state.document {
                let style = doc.styles.entry("base".to_string()).or_default();
                if let Some(f) = style.font.as_mut() { f.size = None; }
                state.is_dirty = true;
            }
        }

        Message::SetInactiveColor(val) => {
             if let Some(doc) = &mut state.document {
                let style = doc.styles.entry("base".to_string()).or_default();
                let c = style.colors.get_or_insert_with(Default::default);
                let inactive = c.inactive.get_or_insert_with(Default::default);
                inactive.fill = Some(val);
                state.is_dirty = true;
            }
        }
        Message::UnsetInactiveColor => {
             if let Some(doc) = &mut state.document {
                let style = doc.styles.entry("base".to_string()).or_default();
                if let Some(c) = style.colors.as_mut() {
                    if let Some(inactive) = c.inactive.as_mut() {
                        inactive.fill = None;
                    }
                }
                state.is_dirty = true;
            }
        }

        Message::SetActiveColor(val) => {
             if let Some(doc) = &mut state.document {
                let style = doc.styles.entry("base".to_string()).or_default();
                let c = style.colors.get_or_insert_with(Default::default);
                let active = c.active.get_or_insert_with(Default::default);
                active.fill = Some(val);
                state.is_dirty = true;
            }
        }
        Message::UnsetActiveColor => {
            if let Some(doc) = &mut state.document {
                let style = doc.styles.entry("base".to_string()).or_default();
                if let Some(c) = style.colors.as_mut() {
                    if let Some(active) = c.active.as_mut() {
                        active.fill = None;
                    }
                }
                state.is_dirty = true;
            }
        }

        Message::SetCompleteColor(_val) => {
             // 'Complete' usually maps to something else or handled via effect?
             // Assuming it maps to a specific field if it exists, or ignored for now.
             // Legacy supported it. Let's check model if needed. 
             // for now, no-op or mapped to active.
             state.is_dirty = true;
        }
        Message::UnsetCompleteColor => {
            state.is_dirty = true;
        }

        Message::SetEffect(_val) => {
             if let Some(doc) = &mut state.document {
                let _style = doc.styles.entry("base".to_string()).or_default();
                // Effects are complex in V2, usually lists.
                // Simplified string set:
                // style.effect = Some(vec![Effect::...])
                // We'll leave unimplemented or simple TODO
                log::warn!("SetEffect not fully implemented for V2");
             }
        }
        Message::UnsetEffect => {
             // Reset effects
             if let Some(doc) = &mut state.document {
                 let _style = doc.styles.entry("base".to_string()).or_default();
                 // style.effects = None; // Field name?
             }
        }
        
        // Line-level style edits
        Message::SetLineStrokeWidth(val) => {
            // Equivalent to SetStrokeWidth when line selected
             if let Ok(num) = val.parse::<f32>() {
                if let Some(line) = state.current_line_mut() {
                    let s = line.stroke.get_or_insert_with(Default::default);
                    s.width = Some(num);
                    state.is_dirty = true;
                }
             }
        }
        Message::UnsetLineStrokeWidth => {
            if let Some(line) = state.current_line_mut() {
                if let Some(s) = line.stroke.as_mut() { s.width = None; }
                state.is_dirty = true;
            }
        }

        Message::SetLineStrokeColor(val) => {
             if let Some(line) = state.current_line_mut() {
                let s = line.stroke.get_or_insert_with(Default::default);
                s.color = Some(val);
                state.is_dirty = true;
            }
        }
        Message::UnsetLineStrokeColor => {
             if let Some(line) = state.current_line_mut() {
                if let Some(s) = line.stroke.as_mut() { s.color = None; }
                state.is_dirty = true;
            }
        }
        

        
        // ===== Line-level Style Messages =====

        

        
        Message::TogglePreview => {
            state.show_preview = !state.show_preview;
            if state.show_preview {
                update_preview(state);
            }
        }
        
        Message::OpenExportPanel => {
            state.show_export = true;
        }
        
        Message::CloseExportPanel => {
            state.show_export = false;
        }
        
        Message::StartExport => {
            state.export_progress = Some(0.0);
        }
        
        Message::ExportProgress(p) => {
            state.export_progress = Some(p);
        }
        
        Message::ExportComplete(_) => {
            state.export_progress = None;
            state.show_export = false;
        }
        
        Message::WindowResized(w, h) => {
            state.window_width = w;
            state.window_height = h;
        }

        Message::PreviewRendered(handle) => {
            state.preview_handle = Some(handle);
        }

        Message::PreviewError(e) => {
            log::error!("Worker preview failed: {}", e);
        }
        Message::WorkerDisconnected => {
            log::error!("Preview worker disconnected unexpectedly");
        }
    }
    
    Task::none()
}

/// View function - builds the UI with custom styling
pub fn view(state: &AppState, window_id: iced::window::Id) -> Element<'_, Message> {
    // If this is NOT the main window, assume it's a debug window
    // This avoids race condition where debug_window ID isn't set yet when first rendering
    if Some(window_id) != state.main_window {
        return debug_view(state);
    }
    
    // If settings modal is open, show it overlaying everything
    if state.show_settings {
        return settings::view(state);
    }

    // Styled toolbar
    let toolbar = container(
        row![
            button(theme::icon_text("📁", "Open"))
                .style(theme::toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::OpenFile),
            button(theme::icon_text("💾", "Save"))
                .style(theme::toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::SaveFile),
            Space::with_width(Length::Fixed(16.0)),
            container(horizontal_rule(1)).width(Length::Fixed(1.0)).height(Length::Fixed(24.0)),
            Space::with_width(Length::Fixed(16.0)),
            button(theme::icon_text("📤", "Export"))
                .style(theme::primary_button_style)
                .padding([6, 16])
                .on_press(Message::OpenExportPanel),
            Space::with_width(Length::Fixed(12.0)),
            // Style button removed as panel is permanent
            button(theme::icon_text("🐞", "Debug"))
                .style(theme::toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::OpenDebugWindow),  
            Space::with_width(Length::Fixed(12.0)),
            button(theme::icon_text("⚙️", "Settings"))
                .style(theme::toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::ToggleSettings),
            Space::with_width(Length::Fill),
            button(theme::icon_text("👁", if state.show_preview { "Hide Preview" } else { "Preview" }))
                .style(theme::toolbar_button_style)
                .padding([6, 12])
                .on_press(Message::TogglePreview),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    )
    .style(theme::toolbar_style)
    .padding([12, 16])
    .width(Length::Fill);

    // Main content
    let content: Element<Message> = if state.document.is_some() {
        // 1. Preview Panel (Left)
        let preview_panel = container(preview::view(state))
            .style(theme::panel_style)
            .padding(0);
            
        // 2. Inspector Panel (Right, Permanent)
        let inspector_panel = container(inspector::view(state))
            .style(theme::panel_style)
            .padding(0);

        // 3. Character Selector / K-Timing Panel (Bottom Top)
        let ktiming_panel = container(ktiming::view(state))
            .style(theme::panel_style)
            .padding(0);
            
        // 4. Line Selector (Bottom Bottom) - reused editor widget
        let line_selector_panel = container(editor::view(state))
            .style(theme::panel_style)
            .padding(0);
            
        // Top Row: Preview and Inspector
        let top_row = row![
            if state.show_preview {
                container(preview_panel)
                    .width(Length::FillPortion(3))
                    .height(Length::Fill)
            } else {
                // If preview hidden, maybe show placeholder or just expand Inspector?
                // User said "Move preview to left", didn't explicitly say "Always show".
                // But generally "Move preview to left" implies it's a structural change.
                // If hidden, we can hide it.
                 container(Space::with_width(Length::Shrink)) 
                    .width(Length::Fixed(0.0))
                    .height(Length::Fill)
            },
            if state.show_preview { Space::with_width(Length::Fixed(8.0)) } else { Space::with_width(Length::Fixed(0.0)) },
            container(inspector_panel)
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        ]
        .height(Length::FillPortion(3)); // Top section takes more space
        
        // Bottom Column: K-Timing and Line Selector
        let bottom_col = column![
            container(ktiming_panel)
                .width(Length::Fill)
                .height(Length::FillPortion(1)), // Less height for timing
            Space::with_height(Length::Fixed(8.0)),
            container(line_selector_panel)
                .width(Length::Fill)
                .height(Length::FillPortion(1)), // Share bottom space
        ]
        .height(Length::FillPortion(2)); // Bottom section height

        column![
            container(top_row).padding([0, 8]),
            Space::with_height(Length::Fixed(8.0)),
            container(bottom_col).padding([0, 8]),
        ]
        .into()
    } else {
        // Empty state with nice styling
        container(
            column![
                theme::icon_sized("📽️", 64.0),
                Space::with_height(Length::Fixed(16.0)),
                text("No document loaded").size(24),
                Space::with_height(Length::Fixed(8.0)),
                text("Open a .klyric file to get started")
                    .size(14)
                    .color(theme::colors::TEXT_SECONDARY),
                Space::with_height(Length::Fixed(24.0)),
                button(text("Open File").size(14))
                    .style(theme::primary_button_style)
                    .padding([10, 24])
                    .on_press(Message::OpenFile),
            ]
            .align_x(Alignment::Center)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };

    // Timeline with styling
    let timeline_bar = container(timeline::view(state))
        .style(theme::toolbar_style)
        .width(Length::Fill);

    // Main layout with proper background
    container(
        column![
            toolbar,
            container(content)
                .style(theme::content_area_style)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([8, 0]),
            timeline_bar,
        ]
    )
    .style(theme::content_area_style)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Subscription for tick events during playback and keyboard input
pub fn subscription(state: &AppState) -> Subscription<Message> {
    let keyboard_sub = keyboard::on_key_press(|key, _modifiers| {
        match key.as_ref() {
            keyboard::Key::Named(keyboard::key::Named::Space) => {
                Some(Message::MarkSyllable)
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                Some(Message::AdvanceChar)
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                Some(Message::RetreatChar)
            }
            _ => None,
        }
    });

    let mut var_subs = vec![keyboard_sub];
    
    if state.playback.is_playing || (state.show_preview && state.worker_connection.is_some()) {
        var_subs.push(time::every(Duration::from_millis(16)).map(|_| Message::Tick));
    }
    
    // Worker subscription removed in favor of polling in Tick
    // if let Some(conn) = &state.worker_connection { ... }
    
    // Listen for window close events
    var_subs.push(iced::window::close_events().map(Message::DebugWindowClosed));

    Subscription::batch(var_subs)
}

fn update_preview(state: &mut AppState) {
    if !state.show_preview { return; }
    
    let doc = match &state.document {
        Some(d) => d,
        None => return,
    };

    let width = doc.project.resolution.width;
    let height = doc.project.resolution.height;

    // Use worker to request frame
    if let Some(conn) = &state.worker_connection {
        conn.get_worker().request_frame(doc, state.playback.current_time, width, height);
    }
}

/// Debug view helper - shows organized state summary
fn debug_view(state: &AppState) -> Element<'_, Message> {
    let mono = theme::mono_font();
    
    // Helper to create a labeled row
    fn row_item<'a>(label: &'static str, value: String) -> Element<'a, Message> {
        row![
            text(label).size(11).color(theme::colors::TEXT_SECONDARY).width(Length::Fixed(120.0)),
            text(value).size(11).font(iced::Font::MONOSPACE),
        ]
        .spacing(8)
        .into()
    }
    
    // Playback section
    let playback_section = column![
        text("▶ Playback").size(13).color(theme::colors::ACCENT),
        row_item("Current Time:", format!("{:.2}s", state.playback.current_time)),
        row_item("Duration:", format!("{:.2}s", state.playback.duration)),
        row_item("Is Playing:", format!("{}", state.playback.is_playing)),
    ]
    .spacing(4)
    .padding(8);
    
    // File section
    let file_section = column![
        text("📁 File").size(13).color(theme::colors::ACCENT),
        row_item("Path:", state.file_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "None".to_string())),
        row_item("Is Dirty:", format!("{}", state.is_dirty)),
    ]
    .spacing(4)
    .padding(8);
    
    // Selection section
    let selection_section = column![
        text("🎯 Selection").size(13).color(theme::colors::ACCENT),
        row_item("Selected Line:", state.selected_line.map(|i| i.to_string()).unwrap_or_else(|| "None".to_string())),
        row_item("Selected Char:", state.selected_char.map(|i| i.to_string()).unwrap_or_else(|| "None".to_string())),
    ]
    .spacing(4)
    .padding(8);
    
    // Document section
    let doc_section = if let Some(doc) = &state.document {
        column![
            text("📄 Document").size(13).color(theme::colors::ACCENT),
            row_item("Title:", doc.project.title.clone()),
            row_item("Artist:", doc.project.artist.clone().unwrap_or_default()),
            row_item("Lines:", format!("{}", doc.lines.len())),
            row_item("Resolution:", format!("{}x{}", doc.project.resolution.width, doc.project.resolution.height)),
            row_item("FPS:", format!("{}", doc.project.fps)),
            row_item("Audio:", doc.project.audio.clone().unwrap_or_else(|| "None".to_string())),
        ]
        .spacing(4)
        .padding(8)
    } else {
        column![
            text("📄 Document").size(13).color(theme::colors::ACCENT),
            text("No document loaded").size(11).color(theme::colors::TEXT_MUTED),
        ]
        .spacing(4)
        .padding(8)
    };
    
    // Windows section
    let windows_section = column![
        text("🪟 Windows").size(13).color(theme::colors::ACCENT),
        row_item("Main Window:", state.main_window.map(|id| format!("{:?}", id)).unwrap_or_else(|| "None".to_string())),
        row_item("Debug Window:", state.debug_window.map(|id| format!("{:?}", id)).unwrap_or_else(|| "None".to_string())),
    ]
    .spacing(4)
    .padding(8);
    
    // UI state section
    let ui_section = column![
        text("🎨 UI State").size(13).color(theme::colors::ACCENT),
        row_item("Show Preview:", format!("{}", state.show_preview)),

        row_item("Show Export:", format!("{}", state.show_export)),
        row_item("Window Size:", format!("{}x{}", state.window_width, state.window_height)),
    ]
    .spacing(4)
    .padding(8);
    
    container(
        iced::widget::scrollable(
            column![
                text("🔧 Debug State View").size(16).font(mono),
                horizontal_rule(1),
                playback_section,
                horizontal_rule(1),
                file_section,
                horizontal_rule(1),
                selection_section,
                horizontal_rule(1),
                doc_section,
                horizontal_rule(1),
                windows_section,
                horizontal_rule(1),
                ui_section,
            ]
            .spacing(4)
            .width(Length::Fill)
        )
    )
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::canvas_container_style)
    .into()
}
