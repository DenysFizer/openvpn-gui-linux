use iced::widget::{Space, button, checkbox, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::app::Message;
use crate::config::OvpnConfig;
use crate::openvpn::VpnState;
use crate::ui::theme;

#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    config_path: &Option<std::path::PathBuf>,
    config: &Option<OvpnConfig>,
    username: &str,
    password: &str,
    otp_response: &str,
    remember_credentials: bool,
    spinner_frame: u8,
    vpn_state: &VpnState,
    error_message: &'a Option<String>,
) -> Element<'a, Message> {
    let inputs_enabled = !vpn_state.is_active();
    let can_submit = config.is_some()
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

    let mut col = column![config_card(config_path, config, inputs_enabled)]
        .spacing(f32::from(theme::SPACE_MD))
        .width(Length::Fill);

    if let Some(cfg) = config
        && cfg.needs_auth_user_pass
    {
        col = col.push(credentials_card(
            cfg,
            username,
            password,
            otp_response,
            remember_credentials,
            inputs_enabled,
            can_submit,
        ));
    }

    if let Some(err) = error_message {
        col = col.push(error_alert(err));
    }

    col = col.push(connect_button(vpn_state, can_submit, spinner_frame));

    col.into()
}

fn config_card<'a>(
    config_path: &Option<std::path::PathBuf>,
    config: &Option<OvpnConfig>,
    inputs_enabled: bool,
) -> Element<'a, Message> {
    let browse_label = if config_path.is_some() { "Change" } else { "Browse…" };

    let mut browse = button(text(browse_label).size(13)).padding([6, 12]);
    if inputs_enabled {
        browse = browse.on_press(Message::SelectConfig).style(button::secondary);
    } else {
        browse = browse.style(button::secondary);
    }

    let body: Element<'a, Message> = match config_path {
        Some(path) => {
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            let full = path.display().to_string();

            let mut info = column![
                text(filename).size(16),
                text(full).size(11).color(theme::MUTED),
            ]
            .spacing(2)
            .width(Length::Fill);

            if let Some(cfg) = config
                && let Some(server) = cfg.remote_servers.first()
            {
                let proto = cfg
                    .protocol
                    .as_deref()
                    .or(server.protocol.as_deref())
                    .unwrap_or("udp")
                    .to_uppercase();
                info = info.push(
                    text(format!("{}:{} · {}", server.host, server.port, proto))
                        .size(12)
                        .color(theme::SUBTLE),
                );
            }

            row![info, browse]
                .spacing(f32::from(theme::SPACE_MD))
                .align_y(iced::Alignment::Center)
                .into()
        }
        None => row![
            column![
                text("No configuration selected").size(15),
                text("Pick a .ovpn file to get started")
                    .size(12)
                    .color(theme::MUTED),
            ]
            .spacing(2)
            .width(Length::Fill),
            browse,
        ]
        .spacing(f32::from(theme::SPACE_MD))
        .align_y(iced::Alignment::Center)
        .into(),
    };

    container(body)
        .padding(theme::SPACE_MD)
        .width(Length::Fill)
        .style(theme::card)
        .into()
}

#[allow(clippy::too_many_arguments)]
fn credentials_card<'a>(
    config: &OvpnConfig,
    username: &str,
    password: &str,
    otp_response: &str,
    remember_credentials: bool,
    inputs_enabled: bool,
    can_submit: bool,
) -> Element<'a, Message> {
    let mut col = column![].spacing(f32::from(theme::SPACE_SM)).width(Length::Fill);

    col = col.push(
        text("Username")
            .size(12)
            .color(theme::MUTED),
    );
    let mut username_input = text_input("yourname", username).padding(10);
    if inputs_enabled {
        username_input = username_input.on_input(Message::UsernameChanged);
        if can_submit {
            username_input = username_input.on_submit(Message::Connect);
        }
    }
    col = col.push(username_input);

    col = col.push(
        text("Password")
            .size(12)
            .color(theme::MUTED),
    );
    let mut password_input = text_input("••••••••", password)
        .secure(true)
        .padding(10);
    if inputs_enabled {
        password_input = password_input.on_input(Message::PasswordChanged);
        if can_submit {
            password_input = password_input.on_submit(Message::Connect);
        }
    }
    col = col.push(password_input);

    if let Some(challenge) = &config.static_challenge {
        col = col.push(
            text(challenge.text.clone())
                .size(12)
                .color(theme::MUTED),
        );
        let mut otp_input = text_input("Authenticator code", otp_response)
            .secure(!challenge.echo)
            .padding(10);
        if inputs_enabled {
            otp_input = otp_input.on_input(Message::OtpChanged);
            if can_submit {
                otp_input = otp_input.on_submit(Message::Connect);
            }
        }
        col = col.push(otp_input);
    }

    let mut remember = checkbox(remember_credentials);
    if inputs_enabled {
        remember = remember.on_toggle(Message::RememberCredentialsToggled);
    }
    col = col.push(
        row![
            remember,
            text("Remember me").size(13),
            Space::new().width(Length::Fill),
            text("Stored locally").size(11).color(theme::MUTED),
        ]
        .spacing(f32::from(theme::SPACE_SM))
        .align_y(iced::Alignment::Center),
    );

    container(col)
        .padding(theme::SPACE_MD)
        .width(Length::Fill)
        .style(theme::card)
        .into()
}

fn error_alert<'a>(err: &'a str) -> Element<'a, Message> {
    container(
        row![
            text("⚠").size(14).color(theme::DANGER),
            text(err).size(13).color(theme::DANGER).width(Length::Fill),
            button(text("×").size(16))
                .on_press(Message::DismissError)
                .padding([0, 8])
                .style(button::text),
        ]
        .spacing(f32::from(theme::SPACE_SM))
        .align_y(iced::Alignment::Center),
    )
    .padding([theme::SPACE_SM, theme::SPACE_MD])
    .width(Length::Fill)
    .style(theme::alert_error)
    .into()
}

fn connect_button<'a>(
    vpn_state: &VpnState,
    can_submit: bool,
    spinner_frame: u8,
) -> Element<'a, Message> {
    let label = match vpn_state {
        VpnState::Disconnected | VpnState::Error(_) => "Connect",
        VpnState::Connected => "Disconnect",
        VpnState::Disconnecting => "Disconnecting…",
        VpnState::Spawning => "Starting OpenVPN…",
        VpnState::Connecting => "Connecting to server…",
        VpnState::Authenticating => "Authenticating…",
        VpnState::GettingConfig => "Fetching configuration…",
        VpnState::AssigningIp => "Assigning IP…",
        VpnState::AddingRoutes => "Applying routes…",
        VpnState::Reconnecting(_) => "Reconnecting…",
        VpnState::Exiting(_) => "Exiting…",
    };

    let show_spinner = matches!(
        vpn_state,
        VpnState::Spawning
            | VpnState::Connecting
            | VpnState::Authenticating
            | VpnState::GettingConfig
            | VpnState::AssigningIp
            | VpnState::AddingRoutes
            | VpnState::Reconnecting(_)
            | VpnState::Disconnecting
            | VpnState::Exiting(_)
    );

    let content: Element<'a, Message> = if show_spinner {
        row![
            text(theme::spinner_glyph(spinner_frame)).size(16),
            text(label).size(16),
        ]
        .spacing(f32::from(theme::SPACE_SM))
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        text(label).size(16).center().into()
    };

    let base = button(container(content).center_x(Length::Fill))
        .width(Length::Fill)
        .padding(12);

    match vpn_state {
        VpnState::Disconnected | VpnState::Error(_) => {
            if can_submit {
                base.on_press(Message::Connect).style(button::success).into()
            } else {
                base.style(button::secondary).into()
            }
        }
        VpnState::Connected => base
            .on_press(Message::Disconnect)
            .style(button::danger)
            .into(),
        _ => base.style(button::secondary).into(),
    }
}
