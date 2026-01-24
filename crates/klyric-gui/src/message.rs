//! Application Message Types

use std::path::PathBuf;
use klyric_renderer::model::KLyricDocumentV2;

/// All application messages for Iced update loop
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are planned for future implementation
pub enum Message {
    // File operations
    OpenFile,
    FileOpened(Option<PathBuf>),
    DocumentLoaded(Result<KLyricDocumentV2, String>),
    SaveFile,
    FileSaved(Result<PathBuf, String>),
    
    // Line selection
    SelectLine(usize),
    AddLine,
    DeleteLine(usize),
    
    // Character selection
    SelectChar(usize),
    
    // Playback
    Play,
    Pause,
    Stop,
    Seek(f64),
    Tick,
    
    // Timing edits
    SetCharStart(String),
    SetCharEnd(String),
    
    // Transform edits
    SetOffsetX(f32),
    UnsetOffsetX,
    SetOffsetY(f32),
    UnsetOffsetY,
    SetRotation(f32),
    UnsetRotation,
    SetScale(f32),
    UnsetScale,
    SetOpacity(f32),
    UnsetOpacity,
    
    // Style edits
    SetFontFamily(String),
    UnsetFontFamily,
    SetFontSize(String),
    UnsetFontSize,
    SetFillColor(String),
    UnsetFillColor,
    SetStrokeColor(String),
    UnsetStrokeColor,
    SetStrokeWidth(String),
    UnsetStrokeWidth,
    
    // Shadow edits
    SetShadowColor(String),
    UnsetShadowColor,
    SetShadowOffsetX(f32),
    UnsetShadowOffsetX,
    SetShadowOffsetY(f32),
    UnsetShadowOffsetY,
    SetShadowBlur(f32),
    UnsetShadowBlur,
    
    // K-Timing specific
    MarkSyllable,           // Space key pressed - mark current time
    AdvanceChar,            // Move to next character
    RetreatChar,            // Move to previous character
    ResetLineTiming,        // Reset all character timings for current line
    
    // Global style edits
    SelectGlobal, // Select global scope
    SetGlobalFont(String),
    UnsetGlobalFont,
    SetGlobalFontSize(String),
    UnsetGlobalFontSize,
    SetInactiveColor(String),
    UnsetInactiveColor,
    SetActiveColor(String),
    UnsetActiveColor,
    SetCompleteColor(String),
    UnsetCompleteColor,
    SetEffect(String),
    UnsetEffect,
    AddSampleEffect(String),
    
    // Line-level style edits
    SetLineStrokeWidth(String),
    UnsetLineStrokeWidth,
    SetLineStrokeColor(String),
    UnsetLineStrokeColor,
    
    // Export
    OpenExportPanel,
    CloseExportPanel,
    StartExport,
    ExportProgress(f32),
    ExportComplete(Result<PathBuf, String>),
    
    // UI state
    TogglePreview,
    WindowResized(u32, u32),

    // Window management
    OpenDebugWindow,
    DebugWindowOpened(iced::window::Id),
    DebugWindowClosed(iced::window::Id),
    MainWindowOpened(iced::window::Id),
    
    // Worker messages
    PreviewRendered(iced::widget::image::Handle),
    PreviewError(String),
    WorkerDisconnected,

    // Settings
    ToggleSettings,
    FontScanComplete(Vec<crate::utils::font_loader::FontInfo>),
    SelectUiFont(crate::utils::font_loader::FontInfo),
    ToggleShowChineseOnly(bool),
    FontBytesLoaded(Result<Vec<u8>, String>),
    NoOp,
}
