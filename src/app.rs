use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use iced::widget::{column, container, text_editor};
use iced::{Element, Length, Subscription, Task, Theme, window};

use crate::config::{OvpnConfig, parse_ovpn};
use crate::openvpn::management::MgmtCommand;
use crate::openvpn::{AuthRequest, ConnectionInfo, LogEntry, VpnState};
use crate::settings;
use crate::ui;

const ICON_BYTES: &[u8] = include_bytes!("../assets/logo.png");

pub fn run() -> iced::Result {
    let icon = match window::icon::from_file_data(ICON_BYTES, None) {
        Ok(icon) => Some(icon),
        Err(e) => {
            log::warn!("Failed to load window icon: {e}");
            None
        }
    };

    let mut window_settings = window::Settings::default();
    window_settings.icon = icon;
    window_settings.size = iced::Size::new(600.0, 700.0);

    iced::application(App::new, App::update, App::view)
        .title("OpenVPN Client")
        .theme(App::theme)
        .subscription(App::subscription)
        .window(window_settings)
        .run()
}

pub struct App {
    config_path: Option<PathBuf>,
    config: Option<OvpnConfig>,
    username: String,
    password: String,
    otp_response: String,
    vpn_state: VpnState,
    connection_info: Option<ConnectionInfo>,
    log_lines: Vec<LogEntry>,
    log_content: text_editor::Content,
    show_logs: bool,
    error_message: Option<String>,

    // Management socket state
    mgmt_socket_path: Option<PathBuf>,
    mgmt_cmd_tx: Option<tokio::sync::mpsc::UnboundedSender<MgmtCommand>>,
    mgmt_cmd_rx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<MgmtCommand>>>>,
    openvpn_pid: Option<u32>,
    subscription_id: u64,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        // Load saved settings
        let saved = settings::load();

        let password = saved.password();
        let config_path = saved.config_path.map(PathBuf::from);
        let username = saved.username.unwrap_or_default();

        let mut app = Self {
            config_path: config_path.clone(),
            config: None,
            username,
            password,
            otp_response: String::new(),
            vpn_state: VpnState::Disconnected,
            connection_info: None,
            log_lines: Vec::new(),
            log_content: text_editor::Content::new(),
            show_logs: false,
            error_message: None,
            mgmt_socket_path: None,
            mgmt_cmd_tx: None,
            mgmt_cmd_rx: Arc::new(Mutex::new(None)),
            openvpn_pid: None,
            subscription_id: 0,
        };

        // Build startup tasks
        let mut tasks = vec![Task::perform(
            crate::openvpn::manager::check_openvpn_installed(),
            Message::OpenvpnChecked,
        )];

        // If we have a saved config path, parse it
        if let Some(path) = config_path {
            if path.exists() {
                tasks.push(Task::perform(
                    load_and_parse_config(path),
                    Message::ConfigParsed,
                ));
            } else {
                app.config_path = None;
                app.error_message = Some(format!("Saved config file no longer exists"));
            }
        }

        (app, Task::batch(tasks))
    }

    fn save_settings(&self) {
        let mut s = settings::Settings::default();
        s.config_path = self.config_path.as_ref().map(|p| p.display().to_string());
        s.username = if self.username.is_empty() {
            None
        } else {
            Some(self.username.clone())
        };
        s.set_password(&self.password);
        settings::save(&s);
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // Startup
    OpenvpnChecked(bool),

    // File selection
    SelectConfig,
    ConfigSelected(Option<PathBuf>),
    ConfigParsed(Result<OvpnConfig, String>),

    // Credential input
    UsernameChanged(String),
    PasswordChanged(String),
    OtpChanged(String),

    // Connection lifecycle
    Connect,
    Disconnect,
    ProcessSpawned(Result<u32, String>),
    ProcessExited(Option<i32>),

    // Management interface events
    MgmtConnected,
    MgmtStateChanged(VpnState),
    MgmtStateConnected(ConnectionInfo),
    MgmtPasswordRequest(AuthRequest),
    MgmtLogLine(LogEntry),
    MgmtByteCount { bytes_in: u64, bytes_out: u64 },
    MgmtError(String),
    MgmtDisconnected,
    MgmtHoldRequest,

    // UI
    ToggleLogs,
    LogEditorAction(text_editor::Action),

    // Timer
    Tick,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenvpnChecked(installed) => {
                if !installed {
                    self.error_message = Some(
                        "OpenVPN is not installed. Please install it with: sudo apt install openvpn"
                            .to_string(),
                    );
                }
                Task::none()
            }
            Message::SelectConfig => {
                Task::perform(pick_config_file(), Message::ConfigSelected)
            }
            Message::ConfigSelected(path) => {
                if let Some(path) = path {
                    self.config_path = Some(path.clone());
                    self.error_message = None;
                    self.save_settings();
                    Task::perform(load_and_parse_config(path), Message::ConfigParsed)
                } else {
                    Task::none()
                }
            }
            Message::ConfigParsed(result) => {
                match result {
                    Ok(config) => {
                        self.config = Some(config);
                        self.error_message = None;
                    }
                    Err(err) => {
                        self.config = None;
                        self.error_message = Some(err);
                    }
                }
                Task::none()
            }
            Message::UsernameChanged(val) => {
                self.username = val;
                self.save_settings();
                Task::none()
            }
            Message::PasswordChanged(val) => {
                self.password = val;
                self.save_settings();
                Task::none()
            }
            Message::OtpChanged(val) => {
                self.otp_response = val;
                Task::none()
            }
            Message::Connect => {
                let config_path = match &self.config_path {
                    Some(p) => p.clone(),
                    None => return Task::none(),
                };

                self.error_message = None;
                self.vpn_state = VpnState::Spawning;
                self.log_lines.clear();
                self.log_content = text_editor::Content::new();
                self.show_logs = true;
                self.connection_info = None;

                let socket_path = crate::openvpn::manager::generate_socket_path();
                self.mgmt_socket_path = Some(socket_path.clone());

                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                self.mgmt_cmd_tx = Some(tx);
                *self.mgmt_cmd_rx.lock().unwrap() = Some(rx);

                self.subscription_id += 1;

                Task::perform(
                    async move {
                        crate::openvpn::manager::spawn_openvpn(&config_path, &socket_path)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::ProcessSpawned,
                )
            }
            Message::Disconnect => {
                self.vpn_state = VpnState::Disconnecting;

                if let Some(tx) = &self.mgmt_cmd_tx {
                    let _ = tx.send(MgmtCommand::Signal("SIGTERM".to_string()));
                }

                if let Some(pid) = self.openvpn_pid {
                    let socket_path = self.mgmt_socket_path.clone();
                    return Task::perform(
                        async move {
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            crate::openvpn::manager::force_stop(pid).await;
                            if let Some(sp) = socket_path {
                                crate::openvpn::manager::cleanup_socket(&sp).await;
                            }
                            None
                        },
                        Message::ProcessExited,
                    );
                }

                Task::none()
            }
            Message::ProcessSpawned(result) => {
                match result {
                    Ok(pid) => {
                        self.openvpn_pid = Some(pid);
                        self.vpn_state = VpnState::Connecting;
                    }
                    Err(err) => {
                        self.vpn_state = VpnState::Error(err.clone());
                        let friendly = if err.contains("126")
                            || err.contains("127")
                            || err.contains("Authorization")
                        {
                            "Authorization was cancelled. Root access is required to manage VPN connections.".to_string()
                        } else {
                            err
                        };
                        self.error_message = Some(friendly);
                        self.cleanup_connection_state();
                    }
                }
                Task::none()
            }
            Message::ProcessExited(code) => {
                if self.vpn_state != VpnState::Disconnected {
                    self.vpn_state = VpnState::Disconnected;
                    self.connection_info = None;
                    if let Some(code) = code {
                        if code != 0 {
                            self.error_message =
                                Some(format!("OpenVPN process exited with code {code}"));
                        }
                    }
                }
                self.cleanup_connection_state();
                Task::none()
            }
            Message::MgmtConnected => Task::none(),
            Message::MgmtStateChanged(state) => {
                match &state {
                    VpnState::Error(e) => {
                        // Only show error once we've settled into error state
                        self.error_message = Some(e.clone());
                        self.connection_info = None;
                        self.cleanup_connection_state();
                    }
                    VpnState::Exiting(_) => {
                        self.connection_info = None;
                    }
                    VpnState::Disconnected => {
                        self.connection_info = None;
                        self.cleanup_connection_state();
                    }
                    _ => {
                        // Clear errors during transitional states (connecting, auth, etc.)
                        self.error_message = None;
                    }
                }
                self.vpn_state = state;
                Task::none()
            }
            Message::MgmtStateConnected(info) => {
                self.vpn_state = VpnState::Connected;
                self.connection_info = Some(info);
                self.error_message = None;
                Task::none()
            }
            Message::MgmtPasswordRequest(auth_request) => {
                if let Some(tx) = &self.mgmt_cmd_tx {
                    match auth_request {
                        AuthRequest::UserPass | AuthRequest::UserPassWithChallenge { .. } => {
                            let otp = if self
                                .config
                                .as_ref()
                                .is_some_and(|c| c.static_challenge.is_some())
                            {
                                Some(self.otp_response.clone())
                            } else {
                                None
                            };
                            let _ = tx.send(MgmtCommand::SendCredentials {
                                username: self.username.clone(),
                                password: self.password.clone(),
                                otp,
                            });
                        }
                        AuthRequest::PrivateKey => {
                            let _ = tx.send(MgmtCommand::SendPrivateKeyPassphrase(
                                self.password.clone(),
                            ));
                        }
                    }
                }
                Task::none()
            }
            Message::MgmtHoldRequest => {
                if let Some(tx) = &self.mgmt_cmd_tx {
                    let _ = tx.send(MgmtCommand::HoldRelease);
                }
                Task::none()
            }
            Message::MgmtLogLine(entry) => {
                let line = format!(
                    "{} [{}] {}\n",
                    entry.timestamp.format("%H:%M:%S"),
                    entry.level.prefix(),
                    entry.message
                );
                // Append to text_editor content
                self.log_content.perform(text_editor::Action::Move(
                    text_editor::Motion::DocumentEnd,
                ));
                self.log_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        line.into(),
                    )));
                self.log_lines.push(entry);
                if self.log_lines.len() > 1000 {
                    self.log_lines.remove(0);
                }
                Task::none()
            }
            Message::MgmtByteCount { bytes_in, bytes_out } => {
                if let Some(info) = &mut self.connection_info {
                    info.bytes_in = bytes_in;
                    info.bytes_out = bytes_out;
                }
                Task::none()
            }
            Message::MgmtError(err) => {
                if err.contains("Authentication failed") {
                    // Auth failure is terminal — show error and stop
                    self.vpn_state = VpnState::Error(err.clone());
                    self.error_message = Some(err);
                    self.cleanup_connection_state();
                }
                // Other errors are logged but not shown during connection
                // (they appear in the log panel)
                Task::none()
            }
            Message::MgmtDisconnected => {
                if self.vpn_state.is_active() {
                    self.vpn_state = VpnState::Disconnected;
                    self.connection_info = None;
                }
                self.cleanup_connection_state();
                Task::none()
            }
            Message::ToggleLogs => {
                self.show_logs = !self.show_logs;
                Task::none()
            }
            Message::LogEditorAction(action) => {
                // Only allow selection (read-only), not editing
                if action.is_edit() {
                    return Task::none();
                }
                self.log_content.perform(action);
                Task::none()
            }
            Message::Tick => {
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            ui::connect_view::view(
                &self.config_path,
                &self.config,
                &self.username,
                &self.password,
                &self.otp_response,
                &self.vpn_state,
                &self.error_message,
            ),
            ui::status_bar::view(&self.vpn_state, &self.connection_info),
            ui::log_view::view(&self.log_content, self.show_logs, self.log_lines.len()),
        ]
        .spacing(10)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![];

        if let Some(socket_path) = &self.mgmt_socket_path {
            if self.vpn_state.is_active() {
                subs.push(crate::openvpn::management::management_subscription(
                    socket_path.clone(),
                    self.mgmt_cmd_rx.clone(),
                    self.subscription_id,
                ));
            }
        }

        if self.vpn_state == VpnState::Connected {
            subs.push(iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick));
        }

        Subscription::batch(subs)
    }

    fn cleanup_connection_state(&mut self) {
        self.mgmt_cmd_tx = None;
        self.openvpn_pid = None;
    }
}

async fn pick_config_file() -> Option<PathBuf> {
    let handle = rfd::AsyncFileDialog::new()
        .add_filter("OpenVPN Config", &["ovpn", "conf"])
        .set_title("Select OpenVPN Configuration File")
        .pick_file()
        .await?;

    Some(handle.path().to_path_buf())
}

async fn load_and_parse_config(path: PathBuf) -> Result<OvpnConfig, String> {
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Could not read config file: {e}"))?;

    parse_ovpn(&content).map_err(|e| e.to_string())
}
