use iced::{
    widget::{button, checkbox, column, container, row, scrollable, text, Space},
    Alignment, Element, Length,
};

use crate::message::Message;
use crate::state::AppState;
use crate::theme;

pub fn view(state: &AppState) -> Element<'_, Message> {
    let title = container(
        row![
            text("Settings").size(20),
            Space::new().width(Length::Fill),
            button(text("✕").size(16))
                .style(theme::toolbar_button_style)
                .on_press(Message::ToggleSettings),
        ]
        .align_y(Alignment::Center),
    )
    .padding(16)
    .style(theme::section_header_style);

    let chinese_filter = checkbox(state.config.show_chinese_only)
        .label("Show Chinese compatible fonts only")
        .on_toggle(Message::ToggleShowChineseOnly);

    let current_font_name = state.config.ui_font.as_deref().unwrap_or("Default");

    let font_list = state
        .available_fonts
        .iter()
        .filter(|f| !state.config.show_chinese_only || f.is_chinese)
        .map(|f| {
            let is_selected = state.config.ui_font.as_ref() == Some(&f.name);

            button(row![
                text(&f.name).size(14),
                Space::new().width(Length::Fill),
                if is_selected {
                    text("✓").size(14)
                } else {
                    text("")
                }
            ])
            .style(if is_selected {
                theme::primary_button_style
            } else {
                theme::list_item_style
            })
            .width(Length::Fill)
            .padding(10)
            .on_press(Message::SelectUiFont(f.clone()))
            .into()
        })
        .collect::<Vec<Element<Message>>>();

    let content = column![
        chinese_filter,
        Space::new().height(16),
        text("UI Font:").size(14),
        Space::new().height(8),
        container(
            scrollable(column(font_list).spacing(2))
                .height(Length::Fill)
                .style(theme::scrollable_style)
        )
        // .style(theme::scrollable_style) // Removed from container
        .height(Length::Fixed(300.0))
        .width(Length::Fill)
        .padding(1),
        Space::new().height(16),
        text("Preview:").size(14).color(theme::colors::TEXT_SECONDARY),
        container(
            text("The quick brown fox jumps over the lazy dog.\nCreate 1234567890\n测试中文显示效果\n日本語のテスト")
                .size(16)
                 // Note: We can't apply the font immediately to this text unless we loaded it into Iced's font system
                 // and have a handle/name for it. 
                 // If the font is selected, we assume it's loaded.
                 // We need to leak the string because Iced requires static lifetime for font names
                 .font(iced::Font::with_name(Box::leak(current_font_name.to_string().into_boxed_str())))
        )
        .padding(12)
        .style(theme::panel_style)
        .width(Length::Fill),
    ]
    .padding(16);

    container(
        container(column![title, content,])
            .width(Length::Fixed(500.0))
            .style(theme::card_style),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_: &_| container::Style {
        background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7).into()),
        ..Default::default()
    })
    .into()
}
