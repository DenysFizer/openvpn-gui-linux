use iced::widget::{button, container, text};
use iced::{Background, Border, Color, Font, Theme};

pub const SPACE_XS: u16 = 4;
pub const SPACE_SM: u16 = 8;
pub const SPACE_MD: u16 = 12;
pub const SPACE_LG: u16 = 16;

pub const MONO: Font = Font::MONOSPACE;

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgb(
        a.r * (1.0 - t) + b.r * t,
        a.g * (1.0 - t) + b.g * t,
        a.b * (1.0 - t) + b.b * t,
    )
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

pub fn muted(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    mix(p.background.base.text, p.background.base.color, 0.45)
}

pub fn subtle(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    mix(p.background.base.text, p.background.base.color, 0.15)
}

pub fn danger(theme: &Theme) -> Color {
    theme.extended_palette().danger.base.color
}

pub fn success(theme: &Theme) -> Color {
    theme.extended_palette().success.base.color
}

pub fn warning(_theme: &Theme) -> Color {
    Color::from_rgb(0.95, 0.78, 0.35)
}

pub fn info_accent(theme: &Theme) -> Color {
    theme.extended_palette().primary.strong.color
}

pub fn disconnected_dot(theme: &Theme) -> Color {
    mix(
        theme.extended_palette().background.base.text,
        theme.extended_palette().background.base.color,
        0.55,
    )
}

pub fn text_muted(theme: &Theme) -> text::Style {
    text::Style { color: Some(muted(theme)) }
}

pub fn text_subtle(theme: &Theme) -> text::Style {
    text::Style { color: Some(subtle(theme)) }
}

pub fn text_danger(theme: &Theme) -> text::Style {
    text::Style { color: Some(danger(theme)) }
}

pub fn text_info_accent(theme: &Theme) -> text::Style {
    text::Style { color: Some(info_accent(theme)) }
}

pub fn text_on_primary(theme: &Theme) -> text::Style {
    text::Style { color: Some(theme.extended_palette().primary.base.text) }
}

pub fn card(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: None,
        border: Border {
            color: with_alpha(p.background.strong.color, 0.6),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn card_filled(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(p.background.weak.color)),
        border: Border {
            color: with_alpha(p.background.strong.color, 0.4),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn profile_icon(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(with_alpha(p.primary.base.color, 0.45))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn tab_row(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(p.background.weak.color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn tab_active(theme: &Theme, _status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    button::Style {
        background: Some(Background::Color(p.background.strong.color)),
        text_color: p.background.strong.text,
        border: Border {
            color: with_alpha(p.background.strong.color, 0.8),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn tab_inactive(theme: &Theme, status: button::Status) -> button::Style {
    let color = match status {
        button::Status::Hovered => subtle(theme),
        _ => muted(theme),
    };
    button::Style {
        background: None,
        text_color: color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn alert_error(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(with_alpha(p.danger.base.color, 0.12))),
        text_color: Some(p.danger.strong.color),
        border: Border {
            color: with_alpha(p.danger.base.color, 0.55),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn stored_badge(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(with_alpha(p.success.base.color, 0.14))),
        text_color: Some(p.success.base.color),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 20.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn badge_neutral(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(p.background.weak.color)),
        text_color: Some(muted(theme)),
        border: Border {
            color: with_alpha(p.background.strong.color, 0.4),
            width: 1.0,
            radius: 20.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn profile_icon_muted(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(p.background.weak.color)),
        text_color: Some(muted(theme)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn divider(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(with_alpha(p.background.strong.color, 0.4))),
        ..container::Style::default()
    }
}

pub fn import_button(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let (bg, fg) = match status {
        button::Status::Hovered => (
            Some(Background::Color(p.background.weak.color)),
            p.background.base.text,
        ),
        _ => (None, muted(theme)),
    };
    button::Style {
        background: bg,
        text_color: fg,
        border: Border {
            color: with_alpha(p.background.strong.color, 0.6),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn profile_row_button(is_active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = theme.extended_palette();
        if is_active {
            button::Style {
                background: Some(Background::Color(with_alpha(p.primary.base.color, 0.18))),
                text_color: p.background.base.text,
                border: Border {
                    color: with_alpha(p.primary.base.color, 0.45),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..button::Style::default()
            }
        } else {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(p.background.weak.color))
                }
                _ => None,
            };
            button::Style {
                background: bg,
                text_color: p.background.base.text,
                border: Border {
                    color: match status {
                        button::Status::Hovered => with_alpha(p.background.strong.color, 0.4),
                        _ => Color::TRANSPARENT,
                    },
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..button::Style::default()
            }
        }
    }
}

pub fn action_danger_outline(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => {
            Some(Background::Color(with_alpha(p.danger.base.color, 0.12)))
        }
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: p.danger.base.color,
        border: Border {
            color: with_alpha(p.danger.base.color, 0.55),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn action_neutral(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => {
            Some(Background::Color(p.background.weak.color))
        }
        _ => Some(Background::Color(p.background.base.color)),
    };
    button::Style {
        background: bg,
        text_color: p.background.base.text,
        border: Border {
            color: with_alpha(p.background.strong.color, 0.6),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
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
    move |theme, status| {
        let p = theme.extended_palette();
        let (bg, fg) = match kind {
            ConnectButtonKind::Connected => (p.danger.strong.color, p.danger.strong.text),
            ConnectButtonKind::Disconnected => (p.primary.strong.color, p.primary.strong.text),
        };
        let alpha = match status {
            button::Status::Hovered => 1.0,
            button::Status::Pressed => 0.88,
            button::Status::Disabled => 0.55,
            button::Status::Active => 0.95,
        };
        button::Style {
            background: Some(Background::Color(with_alpha(bg, alpha))),
            text_color: fg,
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
