//! KLyric GUI - Main Entry Point

mod app;
mod message;
mod state;
mod widgets;
mod theme;
mod worker;
mod audio;
mod utils;
mod config;


fn new() -> (state::AppState, iced::Task<message::Message>) {
    let state = state::AppState::new();
    
    // Initial font scan task
    let scan_task = iced::Task::perform(
        async {
            tokio::task::spawn_blocking(utils::font_loader::scan_system_fonts).await.unwrap()
        },
        message::Message::FontScanComplete
    );

    (state, scan_task)
}

fn view(state: &state::AppState) -> iced::Element<'_, message::Message> {
    app::view(state)
}

fn app_theme(_state: &state::AppState) -> iced::Theme {
    theme::dark_theme()
}

pub fn main() -> iced::Result {
    env_logger::init();

    // Check for Microsoft YaHei availability
    let mut default_family = "Segoe UI";
    let yahei_path = std::path::PathBuf::from("C:\\Windows\\Fonts\\msyh.ttc");
    let load_yahei = yahei_path.exists();
    if load_yahei {
        default_family = "Microsoft YaHei";
    }

    let mut app = iced::application(new, app::update, view)
        .subscription(app::subscription)
        .theme(app_theme)
        .font(include_bytes!("../fonts/segoeui.ttf").as_slice())
        .font(include_bytes!("../fonts/seguiemj.ttf").as_slice())
        .settings(iced::Settings {
            // Define fonts via settings or load them?
            // 0.14 application builder handles fonts via .font()
            // Setting default font:
            default_font: iced::Font::with_name(default_family),
            ..Default::default()
        });

    if load_yahei {
        if let Ok(bytes) = std::fs::read(yahei_path) {
             log::info!("Loaded Microsoft YaHei font from system");
             // Static leak to keep bytes alive (iced .font() takes slice)
             app = app.font(&*Box::leak(bytes.into_boxed_slice()));
        } else {
             log::warn!("Failed to read Microsoft YaHei font");
        }
    }

    app.run()
}




