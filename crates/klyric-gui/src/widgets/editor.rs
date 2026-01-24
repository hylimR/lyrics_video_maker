//! Editor Panel - Line list, character list, and property editors
//! Redesigned with custom dark theme styling

use iced::{
    widget::{button, column, container, row, scrollable, text, Space, lazy},
    Element, Length, Alignment,
};

use crate::message::Message;
use crate::state::AppState;
use crate::theme;
use crate::utils::refs::DocumentRef;

/// View for the editor panel
/// View for the editor panel (now just Line Selector)
pub fn view(state: &AppState) -> Element<'_, Message> {
    if let Some(doc) = &state.document {
        let doc_ref = DocumentRef(doc.clone());
        let selected_line = state.selected_line;

        lazy(
            (doc_ref, selected_line),
            move |(doc_ref, selected_line)| {
                let doc = &doc_ref.0;

                // Line list with styled items
                let line_list = scrollable(
                    column(
                        doc.lines
                            .iter()
                            .enumerate()
                            .map(|(idx, line)| {
                                let is_selected = selected_line == Some(idx);
                                let fallback = format!("Line {}", idx + 1);
                                let label = line.text.as_deref().unwrap_or(&fallback);
                                let label_owned = label.to_string();

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
                                        |theme: &iced::Theme, status| {
                                            let mut style = theme::list_item_style(theme, status);
                                            style.background = Some(iced::Background::Color(theme::colors::SELECTED));
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

                                container(btn)
                                    .width(Length::Fill)
                                    .into()
                            })
                            .collect::<Vec<Element<Message>>>()
                    )
                    .spacing(2)
                )
                .style(theme::scrollable_style)
                .height(Length::Fill); // Allow it to fill the container
                let line_list: Element<'static, Message> = line_list.into();

                // Line actions
                let line_actions = row![
                    text(format!("{} lines", doc.lines.len()))
                         .size(12)
                         .color(theme::colors::TEXT_SECONDARY),
                    Space::with_width(Length::Fill),
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
                        container(line_actions).padding([4, 8]).style(theme::section_header_style).width(Length::Fill),
                        container(line_list).padding(4).height(Length::Fill),
                    ]
                    .spacing(0)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        ).into()
    } else {
        container(text("No document"))
            .padding(16)
            .into()
    }
}
