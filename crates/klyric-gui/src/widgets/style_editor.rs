#![allow(dead_code)]
//! Style Editor Widget - Global, line, and character style controls
//! Redesigned with custom dark theme styling

use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};

use crate::message::Message;
use crate::state::AppState;
use crate::theme;

/// Available effects
const EFFECTS: [&str; 6] = ["None", "fadeIn", "slideUp", "scaleIn", "glow", "bounce"];

/// View for the style editor panel
pub fn view(state: &AppState) -> Element<'_, Message> {
    let doc = state.document.as_ref();

    // Header
    let header = container(theme::icon_sized("🎨 Style Editor", 14.0))
        .style(theme::section_header_style)
        .padding([8, 12])
        .width(Length::Fill);

    // Global styles section
    let global_section = if let Some(doc) = doc {
        let base_style = doc.styles.get("base");
        let font_family = base_style
            .and_then(|s| s.font.as_ref())
            .map(|f| f.family_or_default())
            .unwrap_or_else(|| "Default".to_string());
        let font_size = base_style
            .and_then(|s| s.font.as_ref())
            .map(|f| f.size_or_default())
            .unwrap_or(72.0);
        let inactive_color = base_style
            .and_then(|s| s.colors.as_ref())
            .and_then(|c| c.inactive.as_ref())
            .and_then(|i| i.fill.clone())
            .unwrap_or_else(|| "#FFFFFF".to_string());
        let active_color = base_style
            .and_then(|s| s.colors.as_ref())
            .and_then(|c| c.active.as_ref())
            .and_then(|a| a.fill.clone())
            .unwrap_or_else(|| "#FFD700".to_string());
        let complete_color = base_style
            .and_then(|s| s.colors.as_ref())
            .and_then(|c| c.complete.as_ref())
            .and_then(|c| c.fill.clone())
            .unwrap_or_else(|| "#00FF00".to_string());

        container(
            column![
                text("Global Styles")
                    .size(12)
                    .color(theme::colors::TEXT_SECONDARY),
                Space::with_height(Length::Fixed(8.0)),
                // Font settings
                row![
                    text("Font:").size(11).width(Length::Fixed(50.0)),
                    text_input("Family", &font_family)
                        .style(theme::text_input_style)
                        .on_input(Message::SetGlobalFont)
                        .size(11)
                        .padding([6, 8])
                        .width(Length::Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text("Size:").size(11).width(Length::Fixed(50.0)),
                    text_input("72", &format!("{}", font_size))
                        .style(theme::text_input_style)
                        .on_input(Message::SetGlobalFontSize)
                        .size(11)
                        .padding([6, 8])
                        .width(Length::Fixed(60.0)),
                    text("px").size(11).color(theme::colors::TEXT_MUTED),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                Space::with_height(Length::Fixed(12.0)),
                text("Colors").size(12).color(theme::colors::TEXT_SECONDARY),
                Space::with_height(Length::Fixed(4.0)),
                // Color settings
                row![
                    text("Inactive:").size(11).width(Length::Fixed(60.0)),
                    text_input("#FFFFFF", &inactive_color)
                        .style(theme::text_input_style)
                        .on_input(Message::SetInactiveColor)
                        .size(11)
                        .padding([6, 8])
                        .width(Length::Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text("Active:").size(11).width(Length::Fixed(60.0)),
                    text_input("#FFD700", &active_color)
                        .style(theme::text_input_style)
                        .on_input(Message::SetActiveColor)
                        .size(11)
                        .padding([6, 8])
                        .width(Length::Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text("Complete:").size(11).width(Length::Fixed(60.0)),
                    text_input("#00FF00", &complete_color)
                        .style(theme::text_input_style)
                        .on_input(Message::SetCompleteColor)
                        .size(11)
                        .padding([6, 8])
                        .width(Length::Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ]
            .spacing(4),
        )
        .style(theme::card_style)
        .padding(12)
        .width(Length::Fill)
    } else {
        container(
            text("Load a document to edit styles")
                .size(12)
                .color(theme::colors::TEXT_MUTED),
        )
        .style(theme::card_style)
        .padding(16)
        .width(Length::Fill)
    };

    // Effect picker section
    let effects: Vec<String> = EFFECTS.iter().map(|s| s.to_string()).collect();
    let selected_effect = state
        .selected_effect
        .clone()
        .unwrap_or_else(|| "None".to_string());

    let effect_section = container(
        column![
            text("Effect").size(12).color(theme::colors::TEXT_SECONDARY),
            Space::with_height(Length::Fixed(8.0)),
            pick_list(effects, Some(selected_effect), Message::SetEffect,).padding([8, 12]),
            Space::with_height(Length::Fixed(8.0)),
            text("Sample Effects")
                .size(12)
                .color(theme::colors::TEXT_SECONDARY),
            row![
                button(text("Typewriter").size(10))
                    .on_press(Message::AddSampleEffect("Typewriter".to_string()))
                    .style(theme::secondary_button_style)
                    .padding([4, 8]),
                button(text("Stroke Reveal").size(10))
                    .on_press(Message::AddSampleEffect("StrokeReveal".to_string()))
                    .style(theme::secondary_button_style)
                    .padding([4, 8]),
            ]
            .spacing(4),
            row![
                button(text("Particle Expr").size(10))
                    .on_press(Message::AddSampleEffect("ParticleOverride".to_string()))
                    .style(theme::secondary_button_style)
                    .padding([4, 8]),
                button(text("Clear Line Effects").size(10))
                    .on_press(Message::UnsetEffect)
                    .style(theme::secondary_button_style)
                    .padding([4, 8]),
            ]
            .spacing(4),
        ]
        .spacing(4),
    )
    .style(theme::card_style)
    .padding(12)
    .width(Length::Fill);

    // Line-level styles
    let line_section = if let Some(line) = state.current_line() {
        let stroke_width = line
            .stroke
            .as_ref()
            .map(|s| s.width_or_default())
            .unwrap_or(0.0);
        let stroke_color = line
            .stroke
            .as_ref()
            .map(|s| s.color_or_default())
            .unwrap_or_default();

        container(
            column![
                row![
                    text("Line Styles")
                        .size(12)
                        .color(theme::colors::TEXT_SECONDARY),
                    Space::with_width(Length::Fill),
                    text(format!(
                        "Line {}",
                        state.selected_line.map(|i| i + 1).unwrap_or(0)
                    ))
                    .size(10)
                    .color(theme::colors::TEXT_MUTED),
                ],
                Space::with_height(Length::Fixed(8.0)),
                row![
                    text("Stroke:").size(11).width(Length::Fixed(50.0)),
                    text_input("0", &format!("{}", stroke_width))
                        .style(theme::text_input_style)
                        .on_input(Message::SetLineStrokeWidth)
                        .size(11)
                        .padding([6, 8])
                        .width(Length::Fixed(50.0)),
                    text("px").size(11).color(theme::colors::TEXT_MUTED),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text("Color:").size(11).width(Length::Fixed(50.0)),
                    text_input("#000000", &stroke_color)
                        .style(theme::text_input_style)
                        .on_input(Message::SetLineStrokeColor)
                        .size(11)
                        .padding([6, 8])
                        .width(Length::Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ]
            .spacing(4),
        )
        .style(theme::card_style)
        .padding(12)
        .width(Length::Fill)
    } else {
        container(
            text("Select a line to edit its styles")
                .size(12)
                .color(theme::colors::TEXT_MUTED),
        )
        .style(theme::card_style)
        .padding(12)
        .width(Length::Fill)
    };

    // Main layout
    scrollable(column![
        header,
        container(column![
            global_section,
            Space::with_height(Length::Fixed(8.0)),
            effect_section,
            Space::with_height(Length::Fixed(8.0)),
            line_section,
        ])
        .padding(12),
    ])
    .style(theme::scrollable_style)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
