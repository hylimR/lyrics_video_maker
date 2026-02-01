//! Preview Panel - Video preview display
//! Redesigned with custom dark theme styling

use iced::{
    widget::{column, container, image, text, Space},
    Alignment, ContentFit, Element, Length,
};

use crate::message::Message;
use crate::state::AppState;
use crate::theme;

/// View for the preview panel
pub fn view(state: &AppState) -> Element<'_, Message> {
    // Header
    let header = container(
        row![
            theme::icon_sized("👁️ Preview", 14.0),
            Space::new().width(Length::Fill),
            if let Some(doc) = &state.document {
                text(format!(
                    "{}×{}",
                    doc.project.resolution.width, doc.project.resolution.height
                ))
                .size(12)
                .color(theme::colors::TEXT_SECONDARY)
            } else {
                text("")
            },
        ]
        .align_y(Alignment::Center),
    )
    .style(theme::section_header_style)
    .padding([8, 12])
    .width(Length::Fill);

    // Preview content
    let preview_content: Element<Message> = if let Some(handle) = &state.preview_handle {
        // Render the preview image
        container(image(handle.clone()).content_fit(ContentFit::Contain))
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::BLACK)),
                border: iced::Border {
                    color: theme::colors::BORDER,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    } else {
        // No preview available
        container(
            column![
                theme::icon_sized("🎥", 48.0).color(theme::colors::TEXT_MUTED),
                Space::new().height(Length::Fixed(12.0)),
                text("No preview available")
                    .size(14)
                    .color(theme::colors::TEXT_SECONDARY),
                Space::new().height(Length::Fixed(4.0)),
                text("Load a document to see the preview")
                    .size(11)
                    .color(theme::colors::TEXT_MUTED),
            ]
            .align_x(Alignment::Center),
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(theme::colors::SURFACE_DARKEST)),
            border: iced::Border {
                color: theme::colors::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };

    column![
        header,
        container(preview_content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

use iced::widget::row;




