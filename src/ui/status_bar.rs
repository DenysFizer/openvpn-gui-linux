use iced::widget::{Space, column, container, row, text};
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

    let trailing: Element<'a, Message> = match (vpn_state, connection_info) {
        (VpnState::Connected, Some(info)) => {
            let duration = info
                .connected_since
                .map(|since| {
                    let d = chrono::Local::now() - since;
                    let s = d.num_seconds().max(0);
                    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
                })
                .unwrap_or_else(|| "--:--:--".into());
            text(duration)
                .size(12)
                .color(theme::MUTED)
                .font(theme::MONO)
                .into()
        }
        (VpnState::Disconnected, _) => {
            let msg = if has_config {
                "Ready to connect"
            } else {
                "Select a profile to begin"
            };
            text(msg).size(12).color(theme::MUTED).into()
        }
        _ => Space::new().width(Length::Shrink).into(),
    };

    let status_row = row![
        text("●").size(12).color(state_color),
        text(vpn_state.label().to_string())
            .size(14)
            .color(theme::SUBTLE),
        Space::new().width(Length::Fill),
        trailing,
    ]
    .spacing(f32::from(theme::SPACE_SM))
    .align_y(iced::Alignment::Center);

    let mut col = column![status_row].spacing(f32::from(theme::SPACE_SM));

    if let Some(info) = connection_info {
        col = col.push(details_grid(info));
    }

    container(col)
        .padding([theme::SPACE_SM, 0])
        .width(Length::Fill)
        .into()
}

fn details_grid<'a>(info: &ConnectionInfo) -> Element<'a, Message> {
    let local_ip = info.local_ip.clone().unwrap_or_else(|| "—".into());
    let remote_ip = info.remote_ip.clone().unwrap_or_else(|| "—".into());
    let download = format!("{} ↓", format_bytes(info.bytes_in));
    let upload = format!("{} ↑", format_bytes(info.bytes_out));

    column![
        row![stat_card("Local IP", local_ip), stat_card("Remote IP", remote_ip),]
            .spacing(f32::from(theme::SPACE_SM)),
        row![stat_card("Download", download), stat_card("Upload", upload),]
            .spacing(f32::from(theme::SPACE_SM)),
    ]
    .spacing(f32::from(theme::SPACE_SM))
    .width(Length::Fill)
    .into()
}

fn stat_card<'a>(label: &str, value: String) -> Element<'a, Message> {
    container(
        column![
            text(label.to_uppercase()).size(11).color(theme::MUTED),
            text(value).size(14).color(theme::SUBTLE).font(theme::MONO),
        ]
        .spacing(4),
    )
    .padding([11, 14])
    .width(Length::Fill)
    .style(theme::card_filled)
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
