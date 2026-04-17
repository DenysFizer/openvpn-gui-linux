use iced::widget::{button, container};
use iced::{Background, Border, Color, Font, Theme};

pub const SPACE_XS: u16 = 4;
pub const SPACE_SM: u16 = 8;
pub const SPACE_MD: u16 = 12;
pub const SPACE_LG: u16 = 16;

pub const MUTED: [f32; 3] = [0.62, 0.62, 0.66];
pub const SUBTLE: [f32; 3] = [0.82, 0.82, 0.85];
pub const DANGER: [f32; 3] = [0.95, 0.42, 0.42];
pub const SUCCESS: [f32; 3] = [0.40, 0.85, 0.50];
pub const WARNING: [f32; 3] = [0.95, 0.78, 0.35];

pub const INFO_BG: [f32; 3] = [0.047, 0.267, 0.486]; // #0C447C
pub const INFO_FG: [f32; 3] = [0.710, 0.831, 0.957]; // #B5D4F4

pub const BTN_CONNECTED_BG: [f32; 3] = [0.639, 0.176, 0.176]; // #A32D2D
pub const BTN_CONNECTED_FG: [f32; 3] = [0.969, 0.757, 0.757]; // #F7C1C1
pub const BTN_DISCONNECTED_BG: [f32; 3] = [0.047, 0.267, 0.486]; // #0C447C
pub const BTN_DISCONNECTED_FG: [f32; 3] = [0.710, 0.831, 0.957]; // #B5D4F4

pub const BG_SECONDARY: [f32; 3] = [0.110, 0.110, 0.135];
pub const BG_LIFTED: [f32; 3] = [0.200, 0.200, 0.235];

pub const MONO: Font = Font::MONOSPACE;

fn rgb(c: [f32; 3]) -> Color {
    Color::from_rgb(c[0], c[1], c[2])
}

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

pub fn card_filled(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(rgb(BG_SECONDARY))),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn profile_icon(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            INFO_BG[0], INFO_BG[1], INFO_BG[2], 0.45,
        ))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn tab_row(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(rgb(BG_SECONDARY))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn tab_active(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(rgb(BG_LIFTED))),
        text_color: Color::WHITE,
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn tab_inactive(_theme: &Theme, status: button::Status) -> button::Style {
    let text = match status {
        button::Status::Hovered => rgb(SUBTLE),
        _ => rgb(MUTED),
    };
    button::Style {
        background: None,
        text_color: text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
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

pub fn stored_badge(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            SUCCESS[0], SUCCESS[1], SUCCESS[2], 0.14,
        ))),
        text_color: Some(rgb(SUCCESS)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 20.0.into(),
        },
        ..container::Style::default()
    }
}

#[derive(Copy, Clone)]
pub enum ConnectButtonKind {
    Connected,
    Disconnected,
}

pub fn connect_button_style(
    kind: ConnectButtonKind,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let (bg, fg) = match kind {
        ConnectButtonKind::Connected => (BTN_CONNECTED_BG, BTN_CONNECTED_FG),
        ConnectButtonKind::Disconnected => (BTN_DISCONNECTED_BG, BTN_DISCONNECTED_FG),
    };
    move |_theme, status| {
        let alpha = match status {
            button::Status::Hovered => 1.0,
            button::Status::Pressed => 0.88,
            button::Status::Disabled => 0.55,
            button::Status::Active => 0.95,
        };
        button::Style {
            background: Some(Background::Color(Color::from_rgba(bg[0], bg[1], bg[2], alpha))),
            text_color: rgb(fg),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            ..button::Style::default()
        }
    }
}

pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn spinner_glyph(frame: u8) -> &'static str {
    SPINNER_FRAMES[(frame as usize) % SPINNER_FRAMES.len()]
}
