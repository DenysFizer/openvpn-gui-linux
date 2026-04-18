use std::collections::HashMap;
use std::path::{Path, PathBuf};

use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::app::Message;
use crate::config::OvpnConfig;
use crate::openvpn::VpnState;
use crate::settings::Profile;
use crate::ui::theme::{self, ConnectButtonKind};

pub fn view<'a>(
    profiles: &'a [Profile],
    parsed: &'a HashMap<PathBuf, OvpnConfig>,
    selected: Option<usize>,
    vpn_state: &VpnState,
    rename_state: Option<(usize, &'a str)>,
) -> Element<'a, Message> {
    if profiles.is_empty() {
        return empty_state();
    }

    let list = profile_list(profiles, parsed, selected, vpn_state);
    let import = import_button();

    let mut col = column![section_label("My profiles"), list, import]
        .spacing(f32::from(theme::SPACE_SM))
        .width(Length::Fill);

    if let Some(idx) = selected
        && let Some(profile) = profiles.get(idx)
    {
        col = col
            .push(divider())
            .push(section_label("Profile details"))
            .push(detail_card(idx, profile, parsed, vpn_state, rename_state));
    }

    scrollable(col.padding([0, 0]))
        .height(Length::Fill)
        .into()
}

fn empty_state<'a>() -> Element<'a, Message> {
    let message = text("No profiles yet")
        .size(14)
        .style(theme::text_subtle);
    let hint = text("Import an .ovpn file to get started.")
        .size(12)
        .style(theme::text_muted);

    let inner = column![message, hint, Space::new().height(Length::Fixed(8.0)), import_button()]
        .spacing(f32::from(theme::SPACE_XS))
        .align_x(iced::Alignment::Center)
        .width(Length::Fill);

    container(inner)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(theme::SPACE_LG)
        .into()
}

fn section_label<'a>(label: &str) -> Element<'a, Message> {
    container(text(label.to_uppercase()).size(10).style(theme::text_muted))
        .padding([0, 2])
        .into()
}

fn profile_list<'a>(
    profiles: &'a [Profile],
    parsed: &'a HashMap<PathBuf, OvpnConfig>,
    selected: Option<usize>,
    vpn_state: &VpnState,
) -> Element<'a, Message> {
    let mut col = column![].spacing(f32::from(theme::SPACE_XS)).width(Length::Fill);

    for (idx, profile) in profiles.iter().enumerate() {
        let is_active = Some(idx) == selected;
        let is_connected =
            is_active && matches!(vpn_state, VpnState::Connected);
        col = col.push(profile_row(idx, profile, parsed, is_active, is_connected));
    }

    col.into()
}

fn profile_row<'a>(
    idx: usize,
    profile: &'a Profile,
    parsed: &'a HashMap<PathBuf, OvpnConfig>,
    is_active: bool,
    is_connected: bool,
) -> Element<'a, Message> {
    let path = PathBuf::from(&profile.path);
    let name = display_name(profile, &path);
    let meta = parsed
        .get(&path)
        .map(server_summary)
        .unwrap_or_else(|| "—".to_string());

    let icon_style: fn(&iced::Theme) -> container::Style = if is_active {
        theme::profile_icon
    } else {
        theme::profile_icon_muted
    };
    let icon = container(text("\u{1F6E1}").size(14))
        .width(Length::Fixed(34.0))
        .height(Length::Fixed(34.0))
        .padding(iced::Padding {
            top: 6.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center)
        .style(icon_style);

    let info = column![
        text(name).size(13).style(theme::text_subtle),
        text(meta).size(11).style(theme::text_muted),
    ]
    .spacing(1)
    .width(Length::Fill);

    let badge: Element<'a, Message> = if is_connected {
        container(text("Connected").size(10))
            .padding([2, 8])
            .style(theme::stored_badge)
            .into()
    } else {
        container(text(last_used_label(profile.last_used)).size(10))
            .padding([2, 8])
            .style(theme::badge_neutral)
            .into()
    };

    let body = row![icon, info, badge]
        .spacing(f32::from(theme::SPACE_MD))
        .align_y(iced::Alignment::Center);

    button(body)
        .on_press(Message::ProfileSelected(idx))
        .padding([10, 12])
        .width(Length::Fill)
        .style(theme::profile_row_button(is_active))
        .into()
}

fn import_button<'a>() -> Element<'a, Message> {
    let label = row![
        text("\u{2B06}").size(12),
        text("Import .ovpn file").size(13),
    ]
    .spacing(f32::from(theme::SPACE_XS))
    .align_y(iced::Alignment::Center);

    button(container(label).center_x(Length::Fill))
        .on_press(Message::SelectConfig)
        .padding(9)
        .width(Length::Fill)
        .style(theme::import_button)
        .into()
}

fn divider<'a>() -> Element<'a, Message> {
    container(Space::new().height(Length::Fixed(1.0)))
        .width(Length::Fill)
        .style(theme::divider)
        .into()
}

fn detail_card<'a>(
    idx: usize,
    profile: &'a Profile,
    parsed: &'a HashMap<PathBuf, OvpnConfig>,
    vpn_state: &VpnState,
    rename_state: Option<(usize, &'a str)>,
) -> Element<'a, Message> {
    let path = PathBuf::from(&profile.path);
    let config = parsed.get(&path);

    let header_title: Element<'a, Message> = match rename_state {
        Some((rename_idx, value)) if rename_idx == idx => {
            text_input("Profile name", value)
                .on_input(move |v| Message::ProfileRenameChanged(idx, v))
                .on_submit(Message::ProfileRenameSubmitted(idx))
                .padding([4, 8])
                .size(13)
                .into()
        }
        _ => text(display_name(profile, &path))
            .size(13)
            .style(theme::text_subtle)
            .into(),
    };

    let header = row![
        container(text("\u{1F6E1}").size(16).style(theme::text_info_accent))
            .width(Length::Fixed(38.0))
            .height(Length::Fixed(38.0))
            .padding(iced::Padding {
                top: 7.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center)
            .style(theme::profile_icon),
        column![
            header_title,
            text(profile.path.clone())
                .size(11)
                .style(theme::text_muted),
        ]
        .spacing(2)
        .width(Length::Fill),
    ]
    .spacing(f32::from(theme::SPACE_SM))
    .align_y(iced::Alignment::Center);

    let (host, port, proto) = match config.and_then(|c| {
        c.remote_servers
            .first()
            .map(|s| (s.host.clone(), s.port, s.protocol.clone(), c.protocol.clone()))
    }) {
        Some((h, p, server_proto, file_proto)) => {
            let proto = file_proto
                .or(server_proto)
                .unwrap_or_else(|| "udp".to_string())
                .to_uppercase();
            (h, p.to_string(), proto)
        }
        None => ("—".into(), "—".into(), "—".into()),
    };

    let cipher = config
        .and_then(|c| c.cipher.clone())
        .unwrap_or_else(|| "—".into());
    let auth = config
        .and_then(|c| c.auth.clone())
        .map(format_auth)
        .unwrap_or_else(|| "—".into());

    let rows = column![
        detail_row("Host", host),
        detail_separator(),
        detail_row("Port", port),
        detail_separator(),
        detail_row("Protocol", proto),
        detail_separator(),
        detail_row("Cipher", cipher),
        detail_separator(),
        detail_row("Auth", auth),
    ]
    .spacing(0);

    let actions = action_row(idx, vpn_state, rename_state.is_some_and(|(i, _)| i == idx));

    let body = column![
        header,
        Space::new().height(Length::Fixed(f32::from(theme::SPACE_MD))),
        rows,
        Space::new().height(Length::Fixed(f32::from(theme::SPACE_MD))),
        actions,
    ]
    .width(Length::Fill);

    container(body)
        .padding([theme::SPACE_MD, theme::SPACE_MD + 2])
        .width(Length::Fill)
        .style(theme::card_filled)
        .into()
}

fn detail_row<'a>(key: &str, value: String) -> Element<'a, Message> {
    container(
        row![
            text(key.to_string()).size(11).style(theme::text_muted),
            Space::new().width(Length::Fill),
            text(value).size(11).font(theme::MONO).style(theme::text_subtle),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([5, 0])
    .width(Length::Fill)
    .into()
}

fn detail_separator<'a>() -> Element<'a, Message> {
    container(Space::new().height(Length::Fixed(1.0)))
        .width(Length::Fill)
        .style(theme::divider)
        .into()
}

fn action_row<'a>(
    idx: usize,
    vpn_state: &VpnState,
    is_renaming: bool,
) -> Element<'a, Message> {
    let is_connected = matches!(vpn_state, VpnState::Connected);
    let (connect_label, connect_msg) = if is_connected {
        ("Disconnect", Message::Disconnect)
    } else {
        ("Connect", Message::ProfileConnectRequested(idx))
    };

    let connect_base = button(
        container(text(connect_label).size(12).center()).center_x(Length::Fill),
    )
    .on_press(connect_msg)
    .padding([7, 10])
    .width(Length::Fill);

    let connect_btn = if is_connected {
        connect_base.style(theme::connect_button_style(ConnectButtonKind::Connected))
    } else {
        connect_base.style(theme::connect_button_style(ConnectButtonKind::Disconnected))
    };

    let rename_btn = if is_renaming {
        button(container(text("Cancel").size(12).center()).center_x(Length::Fill))
            .on_press(Message::ProfileRenameCancelled)
            .padding([7, 10])
            .width(Length::Fill)
            .style(theme::action_neutral)
    } else {
        button(container(text("Rename").size(12).center()).center_x(Length::Fill))
            .on_press(Message::ProfileRenameRequested(idx))
            .padding([7, 10])
            .width(Length::Fill)
            .style(theme::action_neutral)
    };

    let remove_btn =
        button(container(text("Remove").size(12).center()).center_x(Length::Fill))
            .on_press(Message::ProfileRemoved(idx))
            .padding([7, 10])
            .width(Length::Fill)
            .style(theme::action_danger_outline);

    row![connect_btn, rename_btn, remove_btn]
        .spacing(f32::from(theme::SPACE_SM))
        .width(Length::Fill)
        .into()
}

fn display_name(profile: &Profile, path: &Path) -> String {
    profile
        .display_name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| profile.path.clone())
        })
}

fn server_summary(config: &OvpnConfig) -> String {
    let Some(server) = config.remote_servers.first() else {
        return "—".to_string();
    };
    let proto = config
        .protocol
        .clone()
        .or_else(|| server.protocol.clone())
        .unwrap_or_else(|| "udp".to_string())
        .to_uppercase();
    format!("{} · {} {}", server.host, proto, server.port)
}

fn last_used_label(last_used: Option<i64>) -> String {
    let Some(ts) = last_used else {
        return "Never used".to_string();
    };
    let now = chrono::Local::now().timestamp();
    let diff = (now - ts).max(0);

    const MIN: i64 = 60;
    const HOUR: i64 = 60 * MIN;
    const DAY: i64 = 24 * HOUR;

    if diff < MIN {
        "Last used just now".to_string()
    } else if diff < HOUR {
        let m = diff / MIN;
        format!("Last used {m}m ago")
    } else if diff < DAY {
        let h = diff / HOUR;
        format!("Last used {h}h ago")
    } else {
        let d = diff / DAY;
        format!("Last used {d}d ago")
    }
}

fn format_auth(raw: String) -> String {
    // Normalize e.g. "SHA512" -> "SHA-512", "SHA256" -> "SHA-256". Anything
    // that already contains a dash or isn't a bare SHA family is passed through.
    let upper = raw.to_uppercase();
    if let Some(rest) = upper.strip_prefix("SHA")
        && !rest.starts_with('-')
        && rest.chars().all(|c| c.is_ascii_digit())
        && !rest.is_empty()
    {
        return format!("SHA-{rest}");
    }
    raw
}
