use iced::widget::{Space, button, checkbox, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::app::Message;
use crate::config::OvpnConfig;
use crate::openvpn::VpnState;
use crate::ui::theme::{self, ConnectButtonKind};

pub fn profile_card<'a>(
    config_path: &Option<std::path::PathBuf>,
    config: &Option<OvpnConfig>,
    inputs_enabled: bool,
) -> Element<'a, Message> {
    let icon = container(
        text("\u{1F6E1}").size(20).color(theme::INFO_FG),
    )
    .width(Length::Fixed(40.0))
    .height(Length::Fixed(40.0))
    .padding(iced::Padding {
        top: 6.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    })
    .align_x(iced::Alignment::Center)
    .align_y(iced::Alignment::Center)
    .style(theme::profile_icon);

    let change_label = if config_path.is_some() {
        "Change"
    } else {
        "Browse…"
    };
    let mut change_btn = button(text(change_label).size(13).color(theme::INFO_FG))
        .padding([4, 8])
        .style(button::text);
    if inputs_enabled {
        change_btn = change_btn.on_press(Message::SelectConfig);
    }

    let info: Element<'a, Message> = match config_path {
        Some(path) => {
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());

            let mut info_col = column![text(filename).size(15).color(theme::SUBTLE),]
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
                info_col = info_col.push(
                    text(format!("{} · {} {}", server.host, proto, server.port))
                        .size(12)
                        .color(theme::MUTED),
                );
            }

            info_col.into()
        }
        None => column![
            text("No profile selected").size(15).color(theme::SUBTLE),
            text("Pick a .ovpn file to get started")
                .size(12)
                .color(theme::MUTED),
        ]
        .spacing(2)
        .width(Length::Fill)
        .into(),
    };

    let body = row![icon, info, change_btn]
        .spacing(f32::from(theme::SPACE_MD))
        .align_y(iced::Alignment::Center);

    container(body)
        .padding([theme::SPACE_MD, theme::SPACE_MD + 2])
        .width(Length::Fill)
        .style(theme::card_filled)
        .into()
}

#[allow(clippy::too_many_arguments)]
pub fn connect_body<'a>(
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

    let mut col = column![]
        .spacing(f32::from(theme::SPACE_MD))
        .width(Length::Fill);

    if let Some(cfg) = config
        && cfg.needs_auth_user_pass
    {
        col = col.push(credentials_block(
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

fn small_label<'a>(s: &str) -> Element<'a, Message> {
    text(s.to_uppercase()).size(12).color(theme::MUTED).into()
}

#[allow(clippy::too_many_arguments)]
fn credentials_block<'a>(
    config: &OvpnConfig,
    username: &str,
    password: &str,
    otp_response: &str,
    remember_credentials: bool,
    inputs_enabled: bool,
    can_submit: bool,
) -> Element<'a, Message> {
    let mut col = column![]
        .spacing(f32::from(theme::SPACE_MD))
        .width(Length::Fill);

    col = col.push(field(
        "Username",
        {
            let mut input = text_input("yourname", username).padding([10, 12]).size(15);
            if inputs_enabled {
                input = input.on_input(Message::UsernameChanged);
                if can_submit {
                    input = input.on_submit(Message::Connect);
                }
            }
            input.into()
        },
    ));

    col = col.push(field(
        "Password",
        {
            let mut input = text_input("\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}", password)
                .secure(true)
                .padding([10, 12])
                .size(15);
            if inputs_enabled {
                input = input.on_input(Message::PasswordChanged);
                if can_submit {
                    input = input.on_submit(Message::Connect);
                }
            }
            input.into()
        },
    ));

    if let Some(challenge) = &config.static_challenge {
        let label = if challenge.text.is_empty() {
            "Authenticator code".to_string()
        } else {
            challenge
                .text
                .trim()
                .strip_prefix("Enter ")
                .or_else(|| challenge.text.trim().strip_prefix("enter "))
                .unwrap_or(challenge.text.trim())
                .to_string()
        };
        let mut otp_input = text_input("000 000", otp_response)
            .secure(!challenge.echo)
            .padding([10, 12])
            .size(17)
            .font(theme::MONO);
        if inputs_enabled {
            otp_input = otp_input.on_input(Message::OtpChanged);
            if can_submit {
                otp_input = otp_input.on_submit(Message::Connect);
            }
        }
        col = col.push(field(&label, otp_input.into()));
    }

    let mut remember = checkbox(remember_credentials);
    if inputs_enabled {
        remember = remember.on_toggle(Message::RememberCredentialsToggled);
    }
    col = col.push(
        row![
            remember,
            text("Remember credentials").size(14).color(theme::SUBTLE),
            Space::new().width(Length::Fill),
            container(text("Stored locally").size(11))
                .padding([3, 8])
                .style(theme::stored_badge),
        ]
        .spacing(f32::from(theme::SPACE_SM))
        .align_y(iced::Alignment::Center),
    );

    col.into()
}

fn field<'a>(label: &str, input: Element<'a, Message>) -> Element<'a, Message> {
    column![small_label(label), input]
        .spacing(f32::from(theme::SPACE_XS))
        .width(Length::Fill)
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
                base.on_press(Message::Connect)
                    .style(theme::connect_button_style(ConnectButtonKind::Disconnected))
                    .into()
            } else {
                base.style(button::secondary).into()
            }
        }
        VpnState::Connected => base
            .on_press(Message::Disconnect)
            .style(theme::connect_button_style(ConnectButtonKind::Connected))
            .into(),
        _ => base.style(button::secondary).into(),
    }
}
