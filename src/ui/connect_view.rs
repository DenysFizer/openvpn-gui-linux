use iced::widget::{button, checkbox, column, container, image, row, text, text_input};
use iced::{ContentFit, Element, Length};

use crate::app::Message;
use crate::config::OvpnConfig;
use crate::openvpn::VpnState;

const LOGO_BYTES: &[u8] = include_bytes!("../../assets/logo.png");

pub fn view<'a>(
    config_path: &Option<std::path::PathBuf>,
    config: &Option<OvpnConfig>,
    username: &str,
    password: &str,
    otp_response: &str,
    remember_credentials: bool,
    vpn_state: &VpnState,
    error_message: &'a Option<String>,
) -> Element<'a, Message> {
    let header = row![
        image(image::Handle::from_bytes(LOGO_BYTES.to_vec()))
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .content_fit(ContentFit::Contain),
        text("OpenVPN Client").size(24),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);

    // Config file selector
    let config_label: String = match config_path {
        Some(path) => path.display().to_string(),
        None => "No config file selected".to_string(),
    };

    let browse_enabled = !vpn_state.is_active();
    let file_row = row![
        text(config_label).width(Length::Fill),
        if browse_enabled {
            button("Browse...").on_press(Message::SelectConfig)
        } else {
            button("Browse...")
        },
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let mut config_col = column![
        text("Configuration File").size(14),
        file_row,
    ]
    .spacing(8);

    // Server info
    if let Some(config) = config {
        if let Some(server) = config.remote_servers.first() {
            let proto = config
                .protocol
                .as_deref()
                .or(server.protocol.as_deref())
                .unwrap_or("udp");
            let server_info = format!("{}:{} ({})", server.host, server.port, proto);
            config_col = config_col.push(
                text(format!("Server: {server_info}"))
                    .size(13)
                    .color([0.6, 0.6, 0.6]),
            );
        }
    }

    let config_card = container(config_col).padding(4).width(Length::Fill);

    // Credential fields (only when config requires auth)
    let can_submit = config.is_some()
        && !vpn_state.is_active()
        && matches!(vpn_state, VpnState::Disconnected | VpnState::Error(_))
        && config.as_ref().is_none_or(|c| {
            if !c.needs_auth_user_pass {
                return true;
            }
            !username.is_empty()
                && !password.is_empty()
                && c.static_challenge
                    .as_ref()
                    .is_none_or(|_| !otp_response.is_empty())
        });

    let mut col = column![header, config_card]
        .spacing(14)
        .width(Length::Fill);

    if let Some(config) = config {
        if config.needs_auth_user_pass {
            let mut cred_col = column![text("Credentials").size(14)].spacing(8);

            let inputs_enabled = !vpn_state.is_active();

            let mut username_input = text_input("Username", username).padding(8);
            if inputs_enabled {
                username_input = username_input.on_input(Message::UsernameChanged);
                if can_submit {
                    username_input = username_input.on_submit(Message::Connect);
                }
            }
            cred_col = cred_col.push(username_input);

            let mut password_input = text_input("Password", password).secure(true).padding(8);
            if inputs_enabled {
                password_input = password_input.on_input(Message::PasswordChanged);
                if can_submit {
                    password_input = password_input.on_submit(Message::Connect);
                }
            }
            cred_col = cred_col.push(password_input);

            if let Some(challenge) = &config.static_challenge {
                let placeholder = challenge.text.clone();
                let mut otp_input = text_input(&placeholder, otp_response)
                    .secure(!challenge.echo)
                    .padding(8);
                if inputs_enabled {
                    otp_input = otp_input.on_input(Message::OtpChanged);
                    if can_submit {
                        otp_input = otp_input.on_submit(Message::Connect);
                    }
                }
                cred_col = cred_col.push(otp_input);
            }

            let mut remember = checkbox(remember_credentials);
            if inputs_enabled {
                remember = remember.on_toggle(Message::RememberCredentialsToggled);
            }
            cred_col = cred_col.push(
                row![remember, text("Remember me").size(14)]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
            );

            col = col.push(container(cred_col).padding(4).width(Length::Fill));
        }
    }

    // Error message
    if let Some(err) = error_message {
        col = col.push(
            container(text(err.as_str()).color([0.9, 0.3, 0.3]).size(13)).padding(8),
        );
    }

    // Connect / Disconnect button
    let connect_btn = match vpn_state {
        VpnState::Disconnected | VpnState::Error(_) => {
            let base = button(text("Connect").size(16).center())
                .width(Length::Fill)
                .padding(14);
            if can_submit {
                base.on_press(Message::Connect).style(button::success)
            } else {
                base
            }
        }
        VpnState::Connected => button(text("Disconnect").size(16).center())
            .on_press(Message::Disconnect)
            .style(button::danger)
            .width(Length::Fill)
            .padding(14),
        VpnState::Disconnecting => button(text("Disconnecting...").size(16).center())
            .width(Length::Fill)
            .padding(14),
        _ => button(text("Connecting...").size(16).center())
            .width(Length::Fill)
            .padding(14),
    };

    col = col.push(connect_btn);

    col.into()
}
