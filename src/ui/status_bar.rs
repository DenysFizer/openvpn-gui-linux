use iced::widget::{column, row, text};
use iced::Element;

use crate::app::Message;
use crate::openvpn::{ConnectionInfo, VpnState};

pub fn view<'a>(vpn_state: &VpnState, connection_info: &Option<ConnectionInfo>) -> Element<'a, Message> {
    let mut col = column![].spacing(4);

    let state_color = match vpn_state {
        VpnState::Connected => [0.3, 0.9, 0.3],
        VpnState::Error(_) => [0.9, 0.3, 0.3],
        VpnState::Disconnected => [0.5, 0.5, 0.5],
        _ => [0.9, 0.8, 0.3], // connecting states = yellow
    };

    col = col.push(
        text(format!("Status: {}", vpn_state.label()))
            .size(14)
            .color(state_color),
    );

    if let Some(info) = connection_info {
        let mut details = row![].spacing(20);

        if let Some(ip) = &info.local_ip {
            details = details.push(text(format!("IP: {ip}")).size(12).color([0.6, 0.6, 0.6]));
        }

        if let Some(remote) = &info.remote_ip {
            let port_str = info
                .remote_port
                .map(|p| format!(":{p}"))
                .unwrap_or_default();
            details = details.push(
                text(format!("Remote: {remote}{port_str}"))
                    .size(12)
                    .color([0.6, 0.6, 0.6]),
            );
        }

        if let Some(since) = &info.connected_since {
            let duration = chrono::Local::now() - since;
            let secs = duration.num_seconds();
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            details = details.push(
                text(format!("Duration: {h:02}:{m:02}:{s:02}"))
                    .size(12)
                    .color([0.6, 0.6, 0.6]),
            );
        }

        col = col.push(details);

        let traffic = format!(
            "Traffic: {} in / {} out",
            format_bytes(info.bytes_in),
            format_bytes(info.bytes_out)
        );
        col = col.push(text(traffic).size(12).color([0.6, 0.6, 0.6]));
    }

    col.into()
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
