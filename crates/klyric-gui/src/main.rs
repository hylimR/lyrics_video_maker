//! KLyric GUI - Main Entry Point

mod app;
mod audio;
mod config;
mod message;
mod state;
mod theme;
mod utils;
mod widgets;
mod worker;

fn main() -> iced::Result {
    env_logger::init();

    let mut daemon = iced::daemon("KLyric", app::update, app::view)
        .subscription(app::subscription)
        .theme(|_, _| theme::dark_theme())
        .font(include_bytes!("../fonts/segoeui.ttf").as_slice())
        .font(include_bytes!("../fonts/seguiemj.ttf").as_slice());

    // Load Microsoft YaHei for Chinese support if available
    let mut default_family = "Segoe UI";
    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
        log::info!("Loaded Microsoft YaHei font from system");
        daemon = daemon.font(&*Box::leak(bytes.into_boxed_slice()));
        default_family = "Microsoft YaHei";
    } else {
        log::warn!("Microsoft YaHei font not found, Chinese characters may not render correctly");
    }

    daemon
        .default_font(iced::Font::with_name(default_family))
        .run_with(|| {
            let mut state = state::AppState::new();

            // Daemon doesn't auto-create a window, so we must open the main window explicitly
            let (id, open_task) = iced::window::open(iced::window::Settings {
                size: iced::Size::new(1400.0, 900.0),
                ..Default::default()
            });

            // Store the main window ID synchronously to avoid race condition in view()
            state.main_window = Some(id);

            // Initial font scan task
            let scan_task = iced::Task::perform(
                async {
                    tokio::task::spawn_blocking(utils::font_loader::scan_system_fonts)
                        .await
                        .unwrap()
                },
                message::Message::FontScanComplete,
            );

            (
                state,
                iced::Task::batch(vec![
                    open_task.map(message::Message::MainWindowOpened),
                    scan_task,
                ]),
            )
        })
}
