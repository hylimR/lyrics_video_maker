//! Timeline Widget - Playback controls and seek slider
//! Redesigned with custom dark theme styling

use iced::{
    widget::{button, container, row, slider, text, Space},
    Alignment, Element, Length,
};

use crate::message::Message;
use crate::state::AppState;
use crate::theme;

/// View for the timeline panel
pub fn view(state: &AppState) -> Element<'_, Message> {
    let current = state.playback.current_time;
    let duration = state.playback.duration;

    // Format time as MM:SS.ms
    let format_time = |t: f64| -> String {
        let mins = (t / 60.0) as u32;
        let secs = t % 60.0;
        format!("{:02}:{:05.2}", mins, secs)
    };

    // Play/Pause button
    let play_pause = if state.playback.is_playing {
        button(theme::icon_sized("⏸", 16.0))
            .style(theme::toolbar_button_style)
            .padding([8, 12])
            .on_press(Message::Pause)
    } else {
        button(theme::icon_sized("▶", 16.0))
            .style(theme::primary_button_style)
            .padding([8, 12])
            .on_press(Message::Play)
    };

    // Stop button
    let stop = button(theme::icon_sized("⏹", 16.0))
        .style(theme::toolbar_button_style)
        .padding([8, 12])
        .on_press(Message::Stop);

    // Time display
    let time_display = container(row![
        text(format_time(current))
            .size(13)
            .color(theme::colors::TEXT_PRIMARY),
        text(" / ").size(13).color(theme::colors::TEXT_MUTED),
        text(format_time(duration))
            .size(13)
            .color(theme::colors::TEXT_SECONDARY),
    ])
    .style(theme::card_style)
    .padding([6, 12]);

    // Seek slider
    let seek_slider = slider(0.0..=duration as f32, current as f32, |v| {
        Message::Seek(v as f64)
    })
    .style(theme::slider_style)
    .width(Length::Fill);

    row![
        play_pause,
        stop,
        Space::new().width(Length::Fixed(16.0)),
        time_display,
        Space::new().width(Length::Fixed(16.0)),
        seek_slider,
        Space::new().width(Length::Fixed(16.0)),
    ]
    .spacing(8)
    .padding([12, 16])
    .align_y(Alignment::Center)
    .into()
}




