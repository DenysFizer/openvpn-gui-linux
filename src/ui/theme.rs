use iced::widget::container;
use iced::{Background, Border, Color, Theme};

pub const SPACE_XS: u16 = 4;
pub const SPACE_SM: u16 = 8;
pub const SPACE_MD: u16 = 12;
pub const SPACE_LG: u16 = 16;

pub const MUTED: [f32; 3] = [0.62, 0.62, 0.66];
pub const SUBTLE: [f32; 3] = [0.82, 0.82, 0.85];
pub const DANGER: [f32; 3] = [0.95, 0.42, 0.42];
pub const SUCCESS: [f32; 3] = [0.40, 0.85, 0.50];
pub const WARNING: [f32; 3] = [0.95, 0.78, 0.35];

pub fn card(_theme: &Theme) -> container::Style {
    container::Style {
        background: None,
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn alert_error(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.95, 0.35, 0.35, 0.12))),
        text_color: Some(Color::from_rgb(DANGER[0], DANGER[1], DANGER[2])),
        border: Border {
            color: Color::from_rgba(0.95, 0.35, 0.35, 0.55),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn status_pill(color: [f32; 3]) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(
            color[0], color[1], color[2], 0.12,
        ))),
        border: Border {
            color: Color::from_rgba(color[0], color[1], color[2], 0.45),
            width: 1.0,
            radius: 999.0.into(),
        },
        ..container::Style::default()
    }
}

pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn spinner_glyph(frame: u8) -> &'static str {
    SPINNER_FRAMES[(frame as usize) % SPINNER_FRAMES.len()]
}
