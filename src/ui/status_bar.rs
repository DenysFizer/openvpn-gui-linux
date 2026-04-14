use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::app::Message;
use crate::openvpn::{ConnectionInfo, VpnState};

pub fn view<'a>(
    vpn_state: &VpnState,
    connection_info: &Option<ConnectionInfo>,
) -> Element<'a, Message> {
    let state_color = match vpn_state {
        VpnState::Connected => [0.3, 0.9, 0.3],
        VpnState::Error(_) => [0.9, 0.3, 0.3],
        VpnState::Disconnected => [0.5, 0.5, 0.5],
        _ => [0.9, 0.8, 0.3],
    };

    let status_row = row![
        text("●").size(16).color(state_color),
        text(vpn_state.label().to_string()).size(15).color(state_color),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let mut col = column![status_row].spacing(8);

    if let Some(info) = connection_info {
        let label_color = [0.55, 0.55, 0.55];
        let value_color = [0.85, 0.85, 0.85];

        let mut details = row![].spacing(16);

        if let Some(ip) = &info.local_ip {
            details = details.push(
                row![
                    text("IP").size(12).color(label_color),
                    text(ip.clone()).size(12).color(value_color),
                ]
                .spacing(4),
            );
        }

        if let Some(remote) = &info.remote_ip {
            let port_str = info
                .remote_port
                .map(|p| format!(":{p}"))
                .unwrap_or_default();
            details = details.push(
                row![
                    text("Remote").size(12).color(label_color),
                    text(format!("{remote}{port_str}")).size(12).color(value_color),
                ]
                .spacing(4),
            );
        }

        if let Some(since) = &info.connected_since {
            let duration = chrono::Local::now() - *since;
            let secs = duration.num_seconds();
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            details = details.push(
                row![
                    text("Duration").size(12).color(label_color),
                    text(format!("{h:02}:{m:02}:{s:02}"))
                        .size(12)
                        .color(value_color),
                ]
                .spacing(4),
            );
        }

        col = col.push(details);

        col = col.push(
            row![
                text("Traffic").size(12).color(label_color),
                text(format!(
                    "{} in / {} out",
                    format_bytes(info.bytes_in),
                    format_bytes(info.bytes_out),
                ))
                .size(12)
                .color(value_color),
            ]
            .spacing(4),
        );
    }

    container(col).padding([6, 4]).width(Length::Fill).into()
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
