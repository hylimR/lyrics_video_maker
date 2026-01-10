// Prevents additional console window on Windows in release builds
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use lyric_video_maker_lib::commands;

fn main() {
    // Initialize logging for debug builds
    #[cfg(debug_assertions)]
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::render::render_video,
            commands::render::cancel_render,
            commands::render::get_render_progress,
            commands::render::check_ffmpeg,
            commands::export::export_frame,
            commands::export::get_system_fonts,
            commands::export::download_ffmpeg,
            commands::export::check_ffmpeg_downloaded,
            commands::export::read_font_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
