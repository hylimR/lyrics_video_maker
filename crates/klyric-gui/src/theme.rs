//! Custom Dark Theme for KLyric GUI
//! Inspired by bl3_save_edit's polished dark aesthetic

use iced::widget::{button, container, scrollable, slider, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

// ============================================================================
// Color Palette
// ============================================================================

/// Font used for icons (Segoe UI Emoji)
pub const ICON_FONT: iced::Font = iced::Font::with_name("Segoe UI Emoji");

/// Icons for the application
#[allow(dead_code)]
pub mod icons {
    pub const FILE_OPEN: &str = "📁";
    pub const FILE_SAVE: &str = "💾";
    pub const EXPORT: &str = "📤";
    pub const DEBUG: &str = "🐞";
    pub const SETTINGS: &str = "⚙️";
    pub const VISIBLE: &str = "👁";
    pub const PREVIEW: &str = "📽️";
    pub const INFO: &str = "ℹ";
    pub const CHECK: &str = "✔";
    pub const CROSS: &str = "✖";
    pub const REFRESH: &str = "⟳";
}

/// Background colors (darkest to lightest)
pub mod colors {
    use iced::Color;

    /// Main application background - very dark
    pub const SURFACE_DARKEST: Color = Color::from_rgb(0.067, 0.067, 0.067); // #111111
    /// Panel backgrounds
    pub const SURFACE_DARK: Color = Color::from_rgb(0.086, 0.086, 0.086); // #161616
    /// Elevated surfaces (cards, modals)
    pub const SURFACE_MID: Color = Color::from_rgb(0.098, 0.098, 0.098); // #191919
    /// Hover states
    pub const SURFACE_LIGHT: Color = Color::from_rgb(0.118, 0.118, 0.118); // #1E1E1E

    /// Border colors
    pub const BORDER: Color = Color::from_rgb(0.137, 0.137, 0.137); // #232323
    pub const BORDER_HOVER: Color = Color::from_rgb(0.176, 0.176, 0.176); // #2D2D2D
    #[allow(dead_code)]
    pub const BORDER_FOCUS: Color = Color::from_rgb(0.4, 0.6, 0.9); // Blue accent

    /// Text colors
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.863, 0.863, 0.863); // #DCDCDC
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.6, 0.6, 0.6); // #999999
    pub const TEXT_MUTED: Color = Color::from_rgb(0.4, 0.4, 0.4); // #666666

    /// Accent colors
    pub const ACCENT: Color = Color::from_rgb(0.4, 0.6, 0.9); // Blue
    pub const ACCENT_HOVER: Color = Color::from_rgb(0.5, 0.7, 1.0); // Lighter blue
    pub const ACCENT_PRESSED: Color = Color::from_rgb(0.3, 0.5, 0.8); // Darker blue

    /// Semantic colors
    pub const SUCCESS: Color = Color::from_rgb(0.4, 0.75, 0.5); // Green

    pub const ERROR: Color = Color::from_rgb(0.9, 0.4, 0.4); // Red

    /// Selection/Active state
    pub const SELECTED: Color = Color::from_rgb(0.2, 0.35, 0.5); // Dark blue
}

// ============================================================================
// Custom Theme
// ============================================================================

/// Create the custom dark theme
pub fn dark_theme() -> Theme {
    Theme::custom(
        "KLyric Dark".to_string(),
        iced::theme::Palette {
            background: colors::SURFACE_DARKEST,
            text: colors::TEXT_PRIMARY,
            primary: colors::ACCENT,
            success: colors::SUCCESS,
            danger: colors::ERROR,
            warning: colors::ERROR,
        },
    )
}

// ============================================================================
// Text & Icon Helpers
// ============================================================================

/// Helper to create a row with an icon and text
pub fn icon_text<'a>(
    icon: &'static str,
    label: &'static str,
) -> iced::widget::Row<'a, crate::message::Message> {
    iced::widget::row![
        iced::widget::text(icon).font(ICON_FONT),
        iced::widget::text(label)
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
}

/// Create icon-only text widget with correct font
#[allow(dead_code)]
pub fn icon<'a>(emoji: &'static str) -> iced::widget::Text<'a> {
    iced::widget::text(emoji).font(ICON_FONT)
}

/// Create icon text with custom size
pub fn icon_sized<'a>(emoji: &'static str, size: f32) -> iced::widget::Text<'a> {
    iced::widget::text(emoji).font(ICON_FONT).size(size)
}

/// Monospace font for debug/code
pub fn mono_font() -> iced::Font {
    iced::Font::MONOSPACE
}

// ============================================================================
// Container Styles
// ============================================================================

/// Main panel container style
pub fn panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE_DARK)),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(colors::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Canvas container style (for debug/preview areas)
pub fn canvas_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE_DARKEST)),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        text_color: Some(colors::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Toolbar container style
pub fn toolbar_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE_MID)),
        border: Border {
            color: colors::BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        text_color: Some(colors::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Section header style
pub fn section_header_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE_MID)),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 2.0.into(),
        },
        text_color: Some(colors::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Card/elevated container style  
pub fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE_MID)),
        border: Border {
            color: colors::BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(colors::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Selected item container style
#[allow(dead_code)]
pub fn selected_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SELECTED)),
        border: Border {
            color: colors::ACCENT,
            width: 1.0,
            radius: 2.0.into(),
        },
        text_color: Some(colors::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Dark background container (for content areas)
pub fn content_area_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SURFACE_DARKEST)),
        border: Border::default(),
        text_color: Some(colors::TEXT_PRIMARY),
        ..Default::default()
    }
}

// ============================================================================
// Button Styles
// ============================================================================

/// Primary button style (for main actions)
pub fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(colors::ACCENT)),
            text_color: Color::WHITE,
            border: Border {
                color: colors::ACCENT,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::ACCENT_HOVER)),
            text_color: Color::WHITE,
            border: Border {
                color: colors::ACCENT_HOVER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::ACCENT_PRESSED)),
            text_color: Color::WHITE,
            border: Border {
                color: colors::ACCENT_PRESSED,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(colors::SURFACE_LIGHT)),
            text_color: colors::TEXT_MUTED,
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
    }
}

/// Secondary button style (for less prominent actions)
pub fn secondary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(colors::SURFACE_MID)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::SURFACE_LIGHT)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::BORDER_HOVER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::SURFACE_DARK)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(colors::SURFACE_DARK)),
            text_color: colors::TEXT_MUTED,
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
    }
}

/// Toolbar button style (minimal)
pub fn toolbar_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: Color::TRANSPARENT,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::SURFACE_LIGHT)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::BORDER_HOVER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::SURFACE_MID)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: colors::TEXT_MUTED,
            border: Border::default(),
            ..Default::default()
        },
    }
}

/// List item button style
pub fn list_item_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::SURFACE_LIGHT)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::BORDER_HOVER,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::SELECTED)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::ACCENT,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: colors::TEXT_MUTED,
            border: Border::default(),
            ..Default::default()
        },
    }
}

/// Character box button style
pub fn char_box_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(colors::SURFACE_MID)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::SURFACE_LIGHT)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::ACCENT,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::SELECTED)),
            text_color: Color::WHITE,
            border: Border {
                color: colors::ACCENT,
                width: 2.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(colors::SURFACE_DARK)),
            text_color: colors::TEXT_MUTED,
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        },
    }
}

// ============================================================================
// Text Input Styles
// ============================================================================

/// Standard text input style
pub fn text_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    match status {
        text_input::Status::Active => text_input::Style {
            background: Background::Color(colors::SURFACE_MID),
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            icon: colors::TEXT_MUTED,
            placeholder: colors::TEXT_MUTED,
            value: colors::TEXT_PRIMARY,
            selection: colors::ACCENT,
        },
        text_input::Status::Hovered => text_input::Style {
            background: Background::Color(colors::SURFACE_MID),
            border: Border {
                color: colors::BORDER_HOVER,
                width: 1.0,
                radius: 4.0.into(),
            },
            icon: colors::TEXT_SECONDARY,
            placeholder: colors::TEXT_MUTED,
            value: colors::TEXT_PRIMARY,
            selection: colors::ACCENT,
        },
        text_input::Status::Focused { .. } => text_input::Style {
            background: Background::Color(colors::SURFACE_LIGHT),
            border: Border {
                color: colors::ACCENT,
                width: 2.0,
                radius: 4.0.into(),
            },
            icon: colors::TEXT_PRIMARY,
            placeholder: colors::TEXT_MUTED,
            value: colors::TEXT_PRIMARY,
            selection: colors::ACCENT,
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(colors::SURFACE_DARK),
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            icon: colors::TEXT_MUTED,
            placeholder: colors::TEXT_MUTED,
            value: colors::TEXT_MUTED,
            selection: colors::BORDER,
        },
    }
}

// ============================================================================
// Scrollable Styles
// ============================================================================

/// Custom scrollbar style
pub fn scrollable_style(_theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let scrollbar = scrollable::Rail {
        background: Some(Background::Color(colors::SURFACE_DARK)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        scroller: scrollable::Scroller {
            background: Background::Color(colors::BORDER_HOVER),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
        },
    };

    let auto_scroll = scrollable::AutoScroll {
        background: Background::Color(Color {
            a: 0.9,
            ..colors::SURFACE_MID
        }),
        border: Border {
            color: Color {
                a: 0.5,
                ..colors::TEXT_PRIMARY
            },
            width: 1.0,
            radius: 50.0.into(),
        },
        shadow: Shadow {
            color: Color {
                a: 0.5,
                ..colors::SURFACE_DARKEST
            },
            offset: Vector::ZERO,
            blur_radius: 5.0,
        },
        icon: colors::TEXT_PRIMARY,
    };

    match status {
        scrollable::Status::Active { .. } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: scrollbar.clone(),
            horizontal_rail: scrollbar,
            gap: None,
            auto_scroll,
        },
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => {
            let hovered_scrollbar = scrollable::Rail {
                background: Some(Background::Color(colors::SURFACE_MID)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                scroller: scrollable::Scroller {
                    background: Background::Color(
                        if is_vertical_scrollbar_hovered || is_horizontal_scrollbar_hovered {
                            colors::ACCENT
                        } else {
                            colors::BORDER_HOVER
                        },
                    ),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                },
            };
            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: hovered_scrollbar.clone(),
                horizontal_rail: hovered_scrollbar,
                gap: None,
                auto_scroll,
            }
        }
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => {
            let dragged_scrollbar = scrollable::Rail {
                background: Some(Background::Color(colors::SURFACE_MID)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                scroller: scrollable::Scroller {
                    background: Background::Color(
                        if is_vertical_scrollbar_dragged || is_horizontal_scrollbar_dragged {
                            colors::ACCENT_PRESSED
                        } else {
                            colors::ACCENT
                        },
                    ),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                },
            };
            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: dragged_scrollbar.clone(),
                horizontal_rail: dragged_scrollbar,
                gap: None,
                auto_scroll,
            }
        }
    }
}

// ============================================================================
// Slider Styles
// ============================================================================

/// Timeline slider style
pub fn slider_style(_theme: &Theme, status: slider::Status) -> slider::Style {
    let base = slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(colors::ACCENT),
                Background::Color(colors::SURFACE_LIGHT),
            ),
            width: 4.0,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: Background::Color(colors::ACCENT),
            border_width: 2.0,
            border_color: colors::SURFACE_DARKEST,
        },
    };

    match status {
        slider::Status::Active => base,
        slider::Status::Hovered => slider::Style {
            handle: slider::Handle {
                background: Background::Color(colors::ACCENT_HOVER),
                ..base.handle
            },
            ..base
        },
        slider::Status::Dragged => slider::Style {
            handle: slider::Handle {
                background: Background::Color(colors::ACCENT_PRESSED),
                border_width: 3.0,
                ..base.handle
            },
            ..base
        },
    }
}

/// Property slider style (smaller)
pub fn property_slider_style(_theme: &Theme, status: slider::Status) -> slider::Style {
    let base = slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(colors::ACCENT),
                Background::Color(colors::BORDER),
            ),
            width: 3.0,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 1.5.into(),
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 6.0 },
            background: Background::Color(colors::TEXT_PRIMARY),
            border_width: 1.0,
            border_color: colors::SURFACE_DARKEST,
        },
    };

    match status {
        slider::Status::Active => base,
        slider::Status::Hovered => slider::Style {
            handle: slider::Handle {
                background: Background::Color(colors::ACCENT),
                ..base.handle
            },
            ..base
        },
        slider::Status::Dragged => slider::Style {
            handle: slider::Handle {
                background: Background::Color(colors::ACCENT_HOVER),
                ..base.handle
            },
            ..base
        },
    }
}

/// Bold font helper
#[allow(dead_code)]
pub fn font_bold() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    }
}

/// Minimal button style for icons
pub fn button_icon_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: colors::TEXT_SECONDARY,
            border: Border::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::SURFACE_LIGHT)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                color: colors::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::SURFACE_MID)),
            text_color: colors::ACCENT,
            border: Border::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: None,
            text_color: colors::TEXT_MUTED,
            border: Border::default(),
            ..Default::default()
        },
    }
}

/// Inherited text input style (dimmed, transparent)
pub fn text_input_inherit_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input_style(_theme, status);
    style.value = colors::TEXT_MUTED;
    if !matches!(status, text_input::Status::Focused { .. }) {
        style.border.color = Color::TRANSPARENT;
        style.background = Background::Color(Color::TRANSPARENT);
    }
    style
}
