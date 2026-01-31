//! Unified Inspector Panel
//! 
//! Manages Global > Line > Character hierarchy with inheritance support.

use iced::{
    widget::{column, container, row, scrollable, text, text_input, button, slider, pick_list, Space},
    Element, Length, Alignment,
};
use crate::message::Message;
use crate::state::AppState;
use crate::theme;
use klyric_renderer::model::Style;

// --- Smart Input Component ---

/// Renders a numeric smart input
pub fn smart_number_input<'a, F>(
    label: &'a str,
    value: Option<f32>,
    resolved: f32,
    _inherited: f32,
    on_change: F,
    on_reset: Message,
) -> Element<'a, Message> 
where F: Fn(String) -> Message + 'a
{
    let is_overridden = value.is_some();
    
    // Calculate display value
    let display_val = if is_overridden { value.unwrap() } else { resolved };
    let display_text = format!("{:.1}", display_val); 

    let label_text = text(label).size(12).color(theme::colors::TEXT_SECONDARY);

    let input = text_input("", &display_text)
        .on_input(on_change) // Pass raw string
        .style(if is_overridden { theme::text_input_style } else { theme::text_input_inherit_style })
        .size(12)
        .padding([4, 8])
        .width(Length::Fill);

    let mut content = row![
        label_text.width(Length::Fixed(70.0)),
        input,
    ].align_y(Alignment::Center).spacing(8);

    if is_overridden {
        let reset_btn = button(
            text("↺").size(12)
        )
        .on_press(on_reset)
        .style(theme::button_icon_style)
        .padding(4);
        
        content = content.push(reset_btn);
    } else {
        // Placeholder
        content = content.push(Space::new().width(Length::Fixed(24.0)));
    }

    content.into()
}

/// Renders a text smart input
#[allow(dead_code)]
pub fn smart_text_input<'a, F>(
    label: &'a str,
    value: Option<String>,
    resolved: String,
    _inherited: String,
    on_change: F,
    on_reset: Message,
) -> Element<'a, Message> 
where F: Fn(String) -> Message + 'a
{
    let is_overridden = value.is_some();
    
    let display_text = if is_overridden { value.unwrap() } else { resolved };
    
    let label_text = text(label).size(12).color(theme::colors::TEXT_SECONDARY);

    let input = text_input("", &display_text)
        .on_input(on_change)
        .style(if is_overridden { theme::text_input_style } else { theme::text_input_inherit_style })
        .size(12)
        .padding([4, 8])
        .width(Length::Fill);

    let mut content = row![
        label_text.width(Length::Fixed(70.0)),
        input,
    ].align_y(Alignment::Center).spacing(8);

    if is_overridden {
        let reset_btn = button(
            text("↺").size(12)
        )
        .on_press(on_reset)
        .style(theme::button_icon_style)
        .padding(4);
        
        content = content.push(reset_btn);
    } else {
        content = content.push(Space::new().width(Length::Fixed(24.0)));
    }

    content.into()
}

/// Renders a slider with input for transform properties
pub fn smart_slider<'a, F>(
    label: &'a str,
    value: Option<f32>,
    resolved: f32,
    _inherited: f32,
    range: std::ops::RangeInclusive<f32>,
    step: f32,
    on_change: F,
    on_reset: Message,
) -> Element<'a, Message> 
where F: Fn(f32) -> Message + 'a + Clone
{
    let is_overridden = value.is_some();
    let display_val = if is_overridden { value.unwrap() } else { resolved };
    
    let label_text = text(label).size(12).color(theme::colors::TEXT_SECONDARY);
    
    let slider_widget = slider(
        range,
        display_val,
        on_change.clone(),
    )
    .step(step)
    .style(theme::property_slider_style)
    .width(Length::Fill);
    
    let input = text_input("", &format!("{:.1}", display_val))
        .on_input(move |s| {
            if let Ok(val) = s.parse::<f32>() {
                on_change(val)
            } else {
                Message::NoOp
            }
        })
        .style(if is_overridden { theme::text_input_style } else { theme::text_input_inherit_style })
        .size(12)
        .padding([4, 8])
        .width(Length::Fixed(50.0));

    let mut content = row![
        label_text.width(Length::Fixed(70.0)),
        slider_widget,
        input,
    ].align_y(Alignment::Center).spacing(8);

    if is_overridden {
        let reset_btn = button(text("↺").size(12))
            .on_press(on_reset)
            .style(theme::button_icon_style)
            .padding(4);
        content = content.push(reset_btn);
    } else {
        content = content.push(Space::new().width(Length::Fixed(24.0)));
    }

    content.into()
}

/// Renders a font picker dropdown
pub fn smart_font_picker<'a>(
    label: &'a str,
    value: Option<String>,
    resolved: String,
    _inherited: String,
    options: Vec<String>,
    on_change: impl Fn(String) -> Message + 'a,
    on_reset: Message,
) -> Element<'a, Message> {
    let is_overridden = value.is_some();
    // For pick_list, we need a selected value. 
    // If overridden, show that. If inherited, show resolved but maybe we need visual cue?
    // standard pick_list doesn't support "dimmed" text easily unless we wrap it or custom style.
    // For now, just show the value.
    
    let selected = if is_overridden { value } else { Some(resolved.clone()) };
    
    let label_text = text(label).size(12).color(theme::colors::TEXT_SECONDARY);
    
    // We need to map strings to options
    // pick_list requires PartialEq + Clone + ToString usually or just the type
    
    let picker = pick_list(
        options,
        selected,
        on_change
    )
    .text_size(12)
    .padding([4, 8])
    .placeholder(if is_overridden { "" } else { &resolved }) // Use placeholder if not overridden for cleaner look or just for fallback
    .width(Length::Fill);
    
    let mut content = row![
        label_text.width(Length::Fixed(70.0)),
        picker,
    ].align_y(Alignment::Center).spacing(8);

    if is_overridden {
        let reset_btn = button(text("↺").size(12))
            .on_press(on_reset)
            .style(theme::button_icon_style)
            .padding(4);
        content = content.push(reset_btn);
    } else {
        content = content.push(Space::new().width(Length::Fixed(24.0)));
    }

    content.into()
}

/// Renders a color input (text for now)
pub fn smart_color_input<'a, F>(
    label: &'a str,
    value: Option<String>,
    resolved: String,
    _inherited: String,
    on_change: F,
    on_reset: Message,
) -> Element<'a, Message> 
where F: Fn(String) -> Message + 'a
{
    let is_overridden = value.is_some();
    let display_text = if is_overridden { value.unwrap() } else { resolved };
    
    let label_text = text(label).size(12).color(theme::colors::TEXT_SECONDARY);

    let input = text_input("#RRGGBB", &display_text)
        .on_input(on_change)
        .style(if is_overridden { theme::text_input_style } else { theme::text_input_inherit_style })
        .size(12)
        .padding([4, 8])
        .width(Length::Fill);

    let mut content = row![
        label_text.width(Length::Fixed(70.0)),
        input,
    ].align_y(Alignment::Center).spacing(8);

    if is_overridden {
        let reset_btn = button(text("↺").size(12))
            .on_press(on_reset)
            .style(theme::button_icon_style)
            .padding(4);
        content = content.push(reset_btn);
    } else {
        content = content.push(Space::new().width(Length::Fixed(24.0)));
    }

    content.into()
}


pub fn view(state: &AppState) -> Element<'_, Message> {
    container(
        column![
            header(state),
            Space::new().height(10),
            scrollable(
                content(state)
            )
        ]
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(0) // Inner padding handled by sections
    .style(theme::panel_style)
    .into()
}

fn header(state: &AppState) -> Element<'_, Message> {
    // Breadcrumb: GLOBAL > LINE 01 > CHAR 05
    let mut parts: Vec<Element<Message>> = vec![];
    
    // Global is always root
    // To fix square box issue, avoid using bold font.
    // To make interactive, use buttons.
    
    let global_active = state.selected_line.is_none();
    
    parts.push(
        if global_active { 
            button(text("GLOBAL").size(11).color(theme::colors::TEXT_PRIMARY))
                .style(theme::list_item_style) 
                .padding(0)
                .into()
        } else {
            button(text("GLOBAL").size(11).color(theme::colors::TEXT_MUTED))
                .on_press(Message::SelectGlobal)
                .style(theme::list_item_style)
                .padding(0)
                .into()
        }
    );

    if let Some(line_idx) = state.selected_line {
        parts.push(text(" › ").size(11).color(theme::colors::TEXT_MUTED).into());
        
        let line_active = state.selected_char.is_none();
        let label = format!("LINE {:02}", line_idx + 1);
        
        parts.push(
            if line_active { 
                button(text(label).size(11).color(theme::colors::TEXT_PRIMARY))
                    .style(theme::list_item_style)
                    .padding(0)
                    .into()
            } else { 
                button(text(label).size(11).color(theme::colors::TEXT_MUTED))
                    .on_press(Message::SelectLine(line_idx))
                    .style(theme::list_item_style)
                    .padding(0)
                    .into()
            }
        );
    }
    
    if let Some(_char_idx) = state.selected_char {
        parts.push(text(" › ").size(11).color(theme::colors::TEXT_MUTED).into());
        let char_text = state.current_char().map(|c| c.char.clone()).unwrap_or("?".to_string());
        
        // Char is always active if present (it's the leaf)
        parts.push(
            button(
                text(format!("CHAR \"{}\"", char_text))
                    .size(11)
                    .color(theme::colors::TEXT_PRIMARY)
            )
            .style(theme::list_item_style)
            .padding(0)
            .into()
        );
    }

    container(row(parts).spacing(2).align_y(Alignment::Center))
        .padding([8, 12])
        .style(theme::section_header_style)
        .width(Length::Fill)
        .into()
}

fn content(state: &AppState) -> Element<'_, Message> {
    column![
        typography_section(state),
        Space::new().height(1),
        shadow_section(state),
        Space::new().height(1),
        effect_section(state),
        Space::new().height(1),
        transform_section(state),
    ].spacing(1).into()
}

// --- Logic Helpers ---

// Returns (local, resolved, inherited)
fn resolve_float<F1, F2, F3>(
    state: &AppState, 
    char_map: F1, 
    line_map: F2,
    global_map: F3,
    default: f32
) -> (Option<f32>, f32, f32) 
where 
    F1: Fn(&klyric_renderer::model::Char) -> Option<f32>,
    F2: Fn(&klyric_renderer::model::Line) -> Option<f32>,
    F3: Fn(&Style) -> Option<f32>
{
    let doc = state.document.as_ref();
    let style = doc.and_then(|d| d.styles.get("base"));
    let global_val = style.and_then(|s| global_map(s));
    let global_resolved = global_val.unwrap_or(default);

    if let Some(ch) = state.current_char() {
        let line_val = state.current_line().and_then(|l| line_map(l));
        let inherited = line_val.unwrap_or(global_resolved);
        let local = char_map(ch);
        let resolved = local.unwrap_or(inherited);
        return (local, resolved, inherited);
    }
    
    if let Some(line) = state.current_line() {
        let inherited = global_resolved;
        let local = line_map(line);
        let resolved = local.unwrap_or(inherited);
        return (local, resolved, inherited);
    }
    
    // Global
    (global_val, global_resolved, default)
}

fn resolve_string<F1, F2, F3>(
    state: &AppState, 
    char_map: F1, 
    line_map: F2,
    global_map: F3,
    default: &str
) -> (Option<String>, String, String) 
where 
    F1: Fn(&klyric_renderer::model::Char) -> Option<String>,
    F2: Fn(&klyric_renderer::model::Line) -> Option<String>,
    F3: Fn(&Style) -> Option<String>
{
    let doc = state.document.as_ref();
    let style = doc.and_then(|d| d.styles.get("base"));
    let global_val = style.and_then(|s| global_map(s));
    let global_resolved = global_val.clone().unwrap_or(default.to_string());

    if let Some(ch) = state.current_char() {
        let line_val = state.current_line().and_then(|l| line_map(l));
        let inherited = line_val.unwrap_or(global_resolved.clone());
        let local = char_map(ch);
        let resolved = local.clone().unwrap_or(inherited.clone());
        return (local, resolved, inherited);
    }
    
    if let Some(line) = state.current_line() {
        let inherited = global_resolved.clone();
        let local = line_map(line);
        let resolved = local.clone().unwrap_or(inherited.clone());
        return (local, resolved, inherited);
    }
    
    // Global
    (global_val, global_resolved, default.to_string())
}


fn typography_section(state: &AppState) -> Element<'_, Message> {
    let (fam_loc, fam_res, fam_inh) = resolve_string(
        state,
        |c| c.font.as_ref().and_then(|f| f.family.clone()),
        |l| l.font.as_ref().and_then(|f| f.family.clone()),
        |s| s.font.as_ref().and_then(|f| f.family.clone()),
        "Noto Sans SC"
    );

    let (size_loc, size_res, size_inh) = resolve_float(
        state,
        |c| c.font.as_ref().and_then(|f| f.size),
        |l| l.font.as_ref().and_then(|f| f.size),
        |s| s.font.as_ref().and_then(|f| f.size),
        72.0
    );
    
    // Fill Color
    // Note: Line/Char don't usually have partial fill override in Style struct?
    // They have StyleOverride which has Font/Stroke/Shadow, but Colors is state-based in Style.
    // Need to check Line/Char struct.
    // Line has `style: Option<String>` (ref).
    // Char has `style: Option<String>`.
    // BUT they might not have direct color overrides?
    // Actually, KLyric V2 `Char` struct has `font`, `stroke`, `shadow`.
    // Does it have `fill`? No.
    // The `Font` struct usually contains family/size/weight. Does it have color?
    // No, `Font` is just metadata. `Style` has `colors`.
    // Wait, so how do I change color of a specific character?
    // In `line_renderer.rs`, it uses `style.colors.active.fill`.
    // If Char doesn't have Color override, then I can't edit it PER CHAR yet?
    // Let's check `Char` struct in `model/line.rs` again.
    // It has `style: Option<String>`.
    
    // UPDATE: The user asked to access `Font`, `Stroke`, `Shadow`, `Glow`, `Transform`.
    // `Stroke` has `color`.
    // `Shadow` has `color`.
    // What about FILL color?
    // It seems missing from `Char` overrides in V2 unless `Style` override is used.
    // I will skip Fill Color per-char for now, or just show Stroke.
    
    let (stroke_w_loc, stroke_w_res, stroke_w_inh) = resolve_float(
        state,
        |c| c.stroke.as_ref().and_then(|s| s.width),
        |l| l.stroke.as_ref().and_then(|s| s.width),
        |s| s.stroke.as_ref().and_then(|s| s.width),
        0.0
    );

    let font_options: Vec<String> = state.available_fonts.iter().map(|f| f.name.clone()).collect();

    container(
        column![
            text("TYPOGRAPHY").size(11).color(theme::colors::TEXT_SECONDARY), // bold removed for consistency
            Space::new().height(10),
            
            smart_font_picker("Family", fam_loc, fam_res, fam_inh, font_options, Message::SetFontFamily, Message::UnsetFontFamily),
            smart_number_input("Size", size_loc, size_res, size_inh, Message::SetFontSize, Message::UnsetFontSize),
            
            Space::new().height(10),
            text("STROKE").size(11).color(theme::colors::TEXT_SECONDARY),
             smart_number_input("Width", stroke_w_loc, stroke_w_res, stroke_w_inh, Message::SetStrokeWidth, Message::UnsetStrokeWidth),
             
              // Stroke Color
             {
                 let (s_col_loc, s_col_res, s_col_inh) = resolve_string(
                    state,
                    |c| c.stroke.as_ref().and_then(|s| s.color.clone()),
                    |l| l.stroke.as_ref().and_then(|s| s.color.clone()),
                    |s| s.stroke.as_ref().and_then(|s| s.color.clone()),
                    "#000000"
                );
                smart_color_input("Color", s_col_loc, s_col_res, s_col_inh, Message::SetStrokeColor, Message::UnsetStrokeColor)
             }
        ]
        .spacing(4)
    )
    .style(theme::card_style)
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn transform_section(state: &AppState) -> Element<'_, Message> {
    // Transform is valid for Global/Line/Char
    
    
    let (x_loc, x_res, x_inh) = resolve_float(
        state,
        |c| c.transform.as_ref().and_then(|t| t.x),
        |l| l.transform.as_ref().and_then(|t| t.x),
        |s| s.transform.as_ref().and_then(|t| t.x),
        0.0
    );
     let (y_loc, y_res, y_inh) = resolve_float(
        state,
        |c| c.transform.as_ref().and_then(|t| t.y),
        |l| l.transform.as_ref().and_then(|t| t.y),
        |s| s.transform.as_ref().and_then(|t| t.y),
        0.0
    );
     let (rot_loc, rot_res, rot_inh) = resolve_float(
        state,
        |c| c.transform.as_ref().and_then(|t| t.rotation),
        |l| l.transform.as_ref().and_then(|t| t.rotation),
        |s| s.transform.as_ref().and_then(|t| t.rotation),
        0.0
    );
     let (scale_loc, scale_res, scale_inh) = resolve_float(
        state,
        |c| c.transform.as_ref().and_then(|t| t.scale),
        |l| l.transform.as_ref().and_then(|t| t.scale),
        |s| s.transform.as_ref().and_then(|t| t.scale),
        1.0
    );

     let (op_loc, op_res, op_inh) = resolve_float(
        state,
        |c| c.transform.as_ref().and_then(|t| t.opacity),
        |l| l.transform.as_ref().and_then(|t| t.opacity),
        |s| s.transform.as_ref().and_then(|t| t.opacity),
        1.0
    );

    container(
        column![
            text("TRANSFORM").size(11).color(theme::colors::TEXT_SECONDARY), // bold removed
            Space::new().height(10),
             smart_slider("Offset X", x_loc, x_res, x_inh, -960.0..=960.0, 1.0, Message::SetOffsetX, Message::UnsetOffsetX),
             smart_slider("Offset Y", y_loc, y_res, y_inh, -540.0..=540.0, 1.0, Message::SetOffsetY, Message::UnsetOffsetY),
             smart_slider("Rotation", rot_loc, rot_res, rot_inh, -360.0..=360.0, 1.0, Message::SetRotation, Message::UnsetRotation),
             smart_slider("Scale", scale_loc, scale_res, scale_inh, 0.0..=5.0, 0.1, Message::SetScale, Message::UnsetScale),
             smart_slider("Opacity", op_loc, op_res, op_inh, 0.0..=1.0, 0.01, Message::SetOpacity, Message::UnsetOpacity),
        ]
        .spacing(4)
    )
    .style(theme::card_style)
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn shadow_section(state: &AppState) -> Element<'_, Message> {
    let (col_loc, col_res, col_inh) = resolve_string(
        state,
        |c| c.shadow.as_ref().and_then(|s| s.color.clone()),
        |l| l.shadow.as_ref().and_then(|s| s.color.clone()),
        |s| s.shadow.as_ref().and_then(|s| s.color.clone()),
        "#000000"
    );

    let (x_loc, x_res, x_inh) = resolve_float(
        state,
        |c| c.shadow.as_ref().and_then(|s| s.x),
        |l| l.shadow.as_ref().and_then(|s| s.x),
        |s| s.shadow.as_ref().and_then(|s| s.x),
        0.0
    );
    
    let (y_loc, y_res, y_inh) = resolve_float(
        state,
        |c| c.shadow.as_ref().and_then(|s| s.y),
        |l| l.shadow.as_ref().and_then(|s| s.y),
        |s| s.shadow.as_ref().and_then(|s| s.y),
        0.0
    );
    
    let (blur_loc, blur_res, blur_inh) = resolve_float(
        state,
        |c| c.shadow.as_ref().and_then(|s| s.blur),
        |l| l.shadow.as_ref().and_then(|s| s.blur),
        |s| s.shadow.as_ref().and_then(|s| s.blur),
        0.0
    );

    container(
        column![
            text("SHADOW").size(11).color(theme::colors::TEXT_SECONDARY),
            Space::new().height(10),
            smart_color_input("Color", col_loc, col_res, col_inh, Message::SetShadowColor, Message::UnsetShadowColor),
            smart_slider("Offset X", x_loc, x_res, x_inh, -20.0..=20.0, 0.5, Message::SetShadowOffsetX, Message::UnsetShadowOffsetX),
            smart_slider("Offset Y", y_loc, y_res, y_inh, -20.0..=20.0, 0.5, Message::SetShadowOffsetY, Message::UnsetShadowOffsetY),
            smart_slider("Blur", blur_loc, blur_res, blur_inh, 0.0..=20.0, 0.5, Message::SetShadowBlur, Message::UnsetShadowBlur),
        ]
        .spacing(4)
    )
    .style(theme::card_style)
    .padding(12)
    .width(Length::Fill)
    .into()
}
fn effect_section(state: &AppState) -> Element<'_, Message> {
    // Show active effects list with delete buttons
    
    let active_effects = if let Some(line) = state.current_line() {
        line.effects.clone()
    } else if let Some(doc) = &state.document {
        doc.styles.get("base")
           .and_then(|s| s.effects.as_ref())
           .cloned()
           .unwrap_or_default()
    } else {
        Vec::new()
    };

    let sample_effects = vec![
        "Typewriter".to_string(), 
        "StrokeReveal".to_string(), 
        "ParticleOverride".to_string()
    ];

    let add_controls = row![
        pick_list(
            sample_effects,
            None::<String>,
            Message::AddSampleEffect
        )
        .placeholder("Add Effect...")
        .text_size(12)
        .padding([4, 8])
        .width(Length::Fill),
        
        button(text("Clear All").size(12))
            .on_press(Message::UnsetEffect)
            .style(theme::secondary_button_style)
            .padding([4, 10]),
    ].spacing(4);

    let mut list_col = column![].spacing(2);
    
    if active_effects.is_empty() {
        list_col = list_col.push(
            text("No active effects")
                .size(11)
                .color(theme::colors::TEXT_MUTED)
                .width(Length::Fill)
        );
    } else {
        for effect_name in active_effects {
            // Trim timestamp for display if possible, or just show full name
            // Name format is usually "type_timestamp"
            let display_name = if let Some(idx) = effect_name.rfind('_') {
                // If the suffix looks like timestamp, hide it? 
                // Alternatively just show the prefix.
                // But unique names are important. Let's show prefix + "..."
                &effect_name[0..idx]
            } else {
                &effect_name
            };
            
            let item = row![
                text(display_name).size(11).width(Length::Fill),
                button(text("✖").size(10))
                    .on_press(Message::RemoveEffect(effect_name.clone()))
                    .style(theme::button_icon_style)
                    .padding(2)
            ]
            .align_y(Alignment::Center)
            .padding(4)
            .spacing(4);
            
            list_col = list_col.push(container(item).style(theme::list_item_style));
        }
    }

    container(
        column![
             row![
                text("EFFECTS").size(11).color(theme::colors::TEXT_SECONDARY),
                Space::new().width(Length::Fill),
                text(format!("Count: {}", 0)).size(10).color(theme::colors::TRANSPARENT), // Hidden but kept for spacing/legacy layout match if needed
            ],
            Space::new().height(10),
            add_controls,
            Space::new().height(4),
            list_col,
        ]
        .spacing(4)
    )
    .style(theme::card_style)
    .padding(12)
    .width(Length::Fill)
    .into()
}




