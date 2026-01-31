//! Editor Panel - Line list, character list, and property editors
//! Redesigned with custom dark theme styling

use iced::{
    widget::{button, column, container, lazy, row, scrollable, text, Space},
    Alignment, Element, Length,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::message::Message;
use crate::state::AppState;
use crate::theme;
use crate::utils::refs::DocumentRef;

/// View for the editor panel
/// View for the editor panel (now just Line Selector)
pub fn view(state: &AppState) -> Element<'_, Message> {
    if let Some(doc) = &state.document {
        let selected_line = state.selected_line;

        // Line list with styled items
        // OPTIMIZATION: Use lazy with DocumentRef (pointer comparison) to avoid rebuilding
        // the potentially large list of widgets on every Tick (60fps), unless document or selection changes.
        let line_list = scrollable(lazy(
            (DocumentRef(doc.clone()), selected_line),
            move |(doc_ref, selected_line)| {
                let line_count = doc_ref.0.lines.len();
                column(
                    (0..line_count)
                        .map(|idx| {
                            let is_selected = selected_line == Some(idx);

                            // Get line reference temporarily to compute hash
                            let line = &doc_ref.0.lines[idx];

                            // Capture only necessary data for the view to optimize caching
                            // Use hashing to avoid cloning string while preserving content-based invalidation
                            let mut hasher = DefaultHasher::new();
                            line.text.hash(&mut hasher);
                            let text_hash = hasher.finish();

                            let start_bits = line.start.to_bits();
                            let end_bits = line.end.to_bits();

                            // Capture doc_ref for the closure
                            let captured_doc = doc_ref.clone();

                            lazy(
                                (idx, is_selected, text_hash, start_bits, end_bits),
                                move |(idx, is_selected, _hash, start_bits, end_bits)| {
                                    let start = f64::from_bits(start_bits);
                                    let end = f64::from_bits(end_bits);

                                    // Retrieve text from captured doc
                                    let line = &captured_doc.0.lines[idx];

                                    let fallback = format!("Line {}", idx + 1);
                                    let label_str = line.text.as_deref().unwrap_or(&fallback);
                                    let label_owned = label_str.to_string();

                                    let timing = format!("{:.1}s - {:.1}s", start, end);

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
                                        .style(if *is_selected {
                                            |t: &iced::Theme, status| {
                                                let mut style = theme::list_item_style(t, status);
                                                style.background =
                                                    Some(iced::Background::Color(theme::colors::SELECTED));
                                                style.border.color = theme::colors::ACCENT;
                                                style.border.width = 1.0;
                                                style
                                            }
                                        } else {
                                            theme::list_item_style
                                        })
                                        .width(Length::Fill)
                                        .padding([4, 8]) // Reduced padding
                                        .on_press(Message::SelectLine(*idx));

                                    container(btn).width(Length::Fill).into()
                                },
                            )
                        })
                        .collect::<Vec<Element<Message>>>(),
                )
                .spacing(2)
                .into()
            },
        ))
        .style(theme::scrollable_style)
        .height(Length::Fill); // Allow it to fill the container

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
