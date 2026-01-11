pub mod cross_dissolve;
pub mod flash;
pub mod blur;
pub mod glitch;
pub mod slide;
pub mod zoom;
pub mod shake;

use crate::model::{Effect, Direction};

pub fn get_transition(name: &str) -> Option<Effect> {
    match name {
        "fade" | "crossDissolve" => Some(cross_dissolve::cross_dissolve(1.0)),
        "flash" | "dipToColor" => Some(flash::flash_in(0.5)),
        "blur" | "blurDissolve" => Some(blur::blur_dissolve(1.0)),
        "glitch" => Some(glitch::glitch(1.0)),
        "slideLeft" | "pushLeft" => Some(slide::slide(1.0, Direction::Rtl)), // Enter from Right to Left
        "slideRight" | "pushRight" => Some(slide::slide(1.0, Direction::Ltr)), // Enter from Left to Right
        "slideUp" | "pushUp" => Some(slide::slide(1.0, Direction::Btt)), // Enter from Bottom to Top
        "slideDown" | "pushDown" => Some(slide::slide(1.0, Direction::Ttb)), // Enter from Top to Bottom
        "zoomIn" | "crashZoom" => Some(zoom::zoom(0.8, true)),
        "zoomOut" => Some(zoom::zoom(0.8, false)),
        "shake" => Some(shake::shake(0.8)),
        _ => None,
    }
}
