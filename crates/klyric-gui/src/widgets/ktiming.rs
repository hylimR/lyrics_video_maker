//! K-Timing Editor Widget - Keyboard-driven syllable timing
//! Redesigned with custom dark theme styling

use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Element, Length,
};

use crate::message::Message;
use crate::state::AppState;
use crate::theme;

/// View for the K-Timing panel
pub fn view(state: &AppState) -> Element<'_, Message> {
    // Header with instructions
    let header = container(
        row![
            theme::icon_sized("⌨️ K-Timing Editor", 14.0),
            Space::new().width(Length::Fixed(24.0)),
            container(row![
                text("Space").size(10).color(theme::colors::ACCENT),
                text(" mark").size(10).color(theme::colors::TEXT_SECONDARY),
                Space::new().width(Length::Fixed(12.0)),
                theme::icon_sized("←→", 10.0).color(theme::colors::ACCENT),
                text(" navigate")
                    .size(10)
                    .color(theme::colors::TEXT_SECONDARY),
            ])
            .style(theme::card_style)
            .padding([4, 8]),
            Space::new().width(Length::Fill),
            button(theme::icon_sized("↺ Reset", 11.0))
                .style(theme::secondary_button_style)
                .padding([4, 10])
                .on_press(Message::ResetLineTiming),
        ]
        .align_y(Alignment::Center),
    )
    .style(theme::section_header_style)
    .padding([8, 12])
    .width(Length::Fill);

    // Content
    let content: Element<Message> = if let Some(doc) = &state.document {
        let selected_line = state.selected_line;
        let selected_char = state.selected_char;

        {
            let line = selected_line.and_then(|idx| doc.lines.get(idx));

                if let Some(line) = line {
                    // Character boxes with timing info
                    let char_boxes: Vec<Element<Message>> = line
                        .chars
                        .iter()
                        .enumerate()
                        .map(|(idx, ch)| {
                            let is_selected = selected_char == Some(idx);
                            let char_text = ch.char.clone();
                            let duration = ch.end - ch.start;

                            let box_content = column![
                                text(char_text).size(20).color(if is_selected {
                                    theme::colors::ACCENT
                                } else {
                                    theme::colors::TEXT_PRIMARY
                                }),
                                text(format!("{:.2}s", duration))
                                    .size(9)
                                    .color(theme::colors::TEXT_MUTED),
                            ]
                            .align_x(Alignment::Center)
                            .spacing(2);

                            let btn = button(box_content)
                                .style(if is_selected {
                                    |theme: &iced::Theme, status| {
                                        let mut style = theme::char_box_style(theme, status);
                                        style.background = Some(iced::Background::Color(
                                            theme::colors::SELECTED,
                                        ));
                                        style.border.color = theme::colors::ACCENT;
                                        style.border.width = 2.0;
                                        style
                                    }
                                } else {
                                    theme::char_box_style
                                })
                                .padding([10, 14])
                                .on_press(Message::SelectChar(idx));

                            container(btn).into()
                        })
                        .collect();

                    column![
                        // Line info
                        container(row![
                            text(line.text.as_deref().unwrap_or("Untitled"))
                                .size(13)
                                .color(theme::colors::TEXT_PRIMARY),
                            Space::new().width(Length::Fill),
                            text(format!("{:.2}s → {:.2}s", line.start, line.end))
                                .size(11)
                                .color(theme::colors::TEXT_SECONDARY),
                        ])
                        .padding([8, 12]),
                        // Character boxes
                        scrollable(row(char_boxes).spacing(6))
                            .direction(scrollable::Direction::Horizontal(
                                scrollable::Scrollbar::default()
                            ))
                            .style(theme::scrollable_style)
                            .width(Length::Fill),
                    ]
                    .spacing(4)
                    .into()
                } else {
                    Element::from(container(
                        column![
                            theme::icon_sized("📝", 32.0).color(theme::colors::TEXT_MUTED),
                            Space::new().height(Length::Fixed(8.0)),
                            text("Select a line to edit timing")
                                .size(13)
                                .color(theme::colors::TEXT_SECONDARY),
                        ]
                        .align_x(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill))
                }
        }
        .into()
    } else {
        container(
            column![
                theme::icon_sized("📝", 32.0).color(theme::colors::TEXT_MUTED),
                Space::new().height(Length::Fixed(8.0)),
                text("Select a line to edit timing")
                    .size(13)
                    .color(theme::colors::TEXT_SECONDARY),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };

    column![
        header,
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
