use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::app::Message;
use crate::openvpn::{ConnectionInfo, VpnState};
use crate::ui::theme;

pub fn view<'a>(
    vpn_state: &VpnState,
    connection_info: &Option<ConnectionInfo>,
    has_config: bool,
) -> Element<'a, Message> {
    let state_color = match vpn_state {
        VpnState::Connected => theme::SUCCESS,
        VpnState::Error(_) => theme::DANGER,
        VpnState::Disconnected => [0.55, 0.55, 0.60],
        _ => theme::WARNING,
    };

    let pill = container(
        row![
            text("●").size(14).color(state_color),
            text(vpn_state.label().to_string())
                .size(13)
                .color(state_color),
        ]
        .spacing(f32::from(theme::SPACE_SM))
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 12])
    .style(theme::status_pill(state_color));

    let subtext: Option<iced::widget::Text> = match vpn_state {
        VpnState::Disconnected if !has_config => {
            Some(text("Select a configuration file to begin"))
        }
        VpnState::Disconnected => Some(text("Ready to connect")),
        VpnState::Error(_) => None,
        _ if connection_info.is_none() => None,
        _ => None,
    };

    let mut header = row![pill]
        .spacing(f32::from(theme::SPACE_MD))
        .align_y(iced::Alignment::Center);
    if let Some(sub) = subtext {
        header = header.push(sub.size(12).color(theme::MUTED));
    }

    let mut col = column![header].spacing(f32::from(theme::SPACE_SM));

    if let Some(info) = connection_info {
        col = col.push(details_grid(info));
    }

    container(col)
        .padding([theme::SPACE_SM, 0])
        .width(Length::Fill)
        .into()
}

fn details_grid<'a>(info: &ConnectionInfo) -> Element<'a, Message> {
    let ip_val = info.local_ip.clone().unwrap_or_else(|| "—".into());
    let remote_val = match &info.remote_ip {
        Some(ip) => {
            let port = info
                .remote_port
                .map(|p| format!(":{p}"))
                .unwrap_or_default();
            format!("{ip}{port}")
        }
        None => "—".into(),
    };

    let duration_val = info
        .connected_since
        .map(|since| {
            let d = chrono::Local::now() - since;
            let s = d.num_seconds().max(0);
            format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
        })
        .unwrap_or_else(|| "—".into());

    let traffic_val = format!(
        "{} ↓ / {} ↑",
        format_bytes(info.bytes_in),
        format_bytes(info.bytes_out),
    );

    container(
        column![
            row![
                kv("IP ADDRESS", ip_val),
                kv("REMOTE", remote_val),
            ]
            .spacing(f32::from(theme::SPACE_LG)),
            row![
                kv("DURATION", duration_val),
                kv("TRAFFIC", traffic_val),
            ]
            .spacing(f32::from(theme::SPACE_LG)),
        ]
        .spacing(f32::from(theme::SPACE_SM)),
    )
    .padding(theme::SPACE_MD)
    .width(Length::Fill)
    .style(theme::card)
    .into()
}

fn kv<'a>(label: &'a str, value: String) -> Element<'a, Message> {
    column![
        text(label).size(10).color(theme::MUTED),
        text(value).size(13).color(theme::SUBTLE),
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
