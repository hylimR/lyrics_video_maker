//! Editor Panel - Line list, character list, and property editors
//! Redesigned with custom dark theme styling

use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Element, Length,
};
use crate::message::Message;
use crate::state::AppState;
use crate::theme;

/// View for the editor panel
/// View for the editor panel (now just Line Selector)
pub fn view(state: &AppState) -> Element<'_, Message> {
    if let Some(doc) = &state.document {
        let selected_line = state.selected_line;

        // Line list with styled items
        let line_list = scrollable({
            let line_count = doc.lines.len();
            column(
                (0..line_count)
                    .map(|idx| {
                        let is_selected = selected_line == Some(idx);
                        let line = &doc.lines[idx];

                        let fallback = format!("Line {}", idx + 1);
                        let label_str = line.text.as_deref().unwrap_or(&fallback);
                        let label_owned = label_str.to_string();

                        let timing = format!("{:.1}s - {:.1}s", line.start, line.end);

                        let content = row![
                            text(format!("{:02}", idx + 1))
                                .size(11)
                                .color(theme::colors::TEXT_MUTED)
                                .width(Length::Fixed(24.0)),
                            column![
                                text(label_owned).size(13),
                                text(timing)
                                    .size(10)
                                    .color(theme::colors::TEXT_SECONDARY),
                            ]
                            .spacing(2),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center);

                        let btn = button(content)
                            .style(if is_selected {
                                |t: &iced::Theme, status| {
                                    let mut style = theme::list_item_style(t, status);
                                    style.background = Some(iced::Background::Color(
                                        theme::colors::SELECTED,
                                    ));
                                    style.border.color = theme::colors::ACCENT;
                                    style.border.width = 1.0;
                                    style
                                }
                            } else {
                                theme::list_item_style
                            })
                            .width(Length::Fill)
                            .padding([4, 8]) // Reduced padding
                            .on_press(Message::SelectLine(idx));

                        container(btn).width(Length::Fill).into()
                    })
                    .collect::<Vec<Element<Message>>>(),
            )
            .spacing(2)
        })
        .style(theme::scrollable_style)
        .height(Length::Fill); // Allow it to fill the container

        // Line actions
        let line_actions = row![
            text(format!("{} lines", doc.lines.len()))
                .size(12)
                .color(theme::colors::TEXT_SECONDARY),
            Space::new().width(Length::Fill),
            button(text("+ Add").size(12))
                .style(theme::secondary_button_style)
                .padding([4, 8])
                .on_press(Message::AddLine),
            if let Some(idx) = selected_line {
                button(theme::icon_sized("🗑", 12.0)) // Compact delete
                    .style(theme::secondary_button_style)
                    .padding([4, 8])
                    .on_press(Message::DeleteLine(idx))
            } else {
                button(theme::icon_sized("🗑", 12.0))
                    .style(theme::secondary_button_style)
                    .padding([4, 8])
            },
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // Main layout
        container(
            column![
                container(line_actions)
                    .padding([4, 8])
                    .style(theme::section_header_style)
                    .width(Length::Fill),
                container(line_list).padding(4).height(Length::Fill),
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        container(text("No document")).padding(16).into()
    }
}
