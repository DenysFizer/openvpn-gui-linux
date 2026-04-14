use iced::widget::{button, checkbox, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::app::Message;
use crate::config::OvpnConfig;
use crate::openvpn::VpnState;

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
    let mut col = column![].spacing(12).width(Length::Fill);

    col = col.push(text("OpenVPN Client").size(22));

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

    col = col.push(text("Configuration File").size(14));
    col = col.push(file_row);

    // Server info
    if let Some(config) = config {
        if let Some(server) = config.remote_servers.first() {
            let proto = config
                .protocol
                .as_deref()
                .or(server.protocol.as_deref())
                .unwrap_or("udp");
            let server_info = format!("{}:{} ({})", server.host, server.port, proto);
            col = col.push(
                text(format!("Server: {server_info}"))
                    .size(13)
                    .color([0.6, 0.6, 0.6]),
            );
        }
    }

    // Credential fields (only when config requires auth)
    if let Some(config) = config {
        if config.needs_auth_user_pass {
            col = col.push(text("Credentials").size(14));

            let inputs_enabled = !vpn_state.is_active();

            let mut username_input =
                text_input("Username", username).padding(8);
            if inputs_enabled {
                username_input = username_input.on_input(Message::UsernameChanged);
            }
            col = col.push(username_input);

            let mut password_input = text_input("Password", password)
                .secure(true)
                .padding(8);
            if inputs_enabled {
                password_input = password_input.on_input(Message::PasswordChanged);
            }
            col = col.push(password_input);

            // OTP field (only when static-challenge is present)
            if let Some(challenge) = &config.static_challenge {
                let placeholder = challenge.text.clone();
                let mut otp_input =
                    text_input(&placeholder, otp_response)
                        .secure(!challenge.echo)
                        .padding(8);
                if inputs_enabled {
                    otp_input = otp_input.on_input(Message::OtpChanged);
                }
                col = col.push(otp_input);
            }

            let mut remember = checkbox(remember_credentials);
            if inputs_enabled {
                remember = remember.on_toggle(Message::RememberCredentialsToggled);
            }
            col = col.push(
                row![remember, text("Remember me").size(14)]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
            );
        }
    }

    // Error message
    if let Some(err) = error_message {
        col = col.push(
            container(text(err.as_str()).color([0.9, 0.3, 0.3]).size(13))
                .padding(8),
        );
    }

    // Connect / Disconnect button
    let connect_btn = match vpn_state {
        VpnState::Disconnected | VpnState::Error(_) => {
            let can_connect = config.is_some()
                && config
                    .as_ref()
                    .is_none_or(|c| !c.needs_auth_user_pass || !username.is_empty());
            if can_connect {
                button("Connect")
                    .on_press(Message::Connect)
                    .width(Length::Fill)
                    .padding(12)
            } else {
                button("Connect").width(Length::Fill).padding(12)
            }
        }
        VpnState::Connected => button("Disconnect")
            .on_press(Message::Disconnect)
            .width(Length::Fill)
            .padding(12),
        VpnState::Disconnecting => {
            button("Disconnecting...").width(Length::Fill).padding(12)
        }
        _ => {
            button("Connecting...")
                .width(Length::Fill)
                .padding(12)
        }
    };

    col = col.push(connect_btn);

    col.into()
}
