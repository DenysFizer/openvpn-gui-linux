use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use iced::widget::{column, container, text, text_editor};
use iced::{Element, Length, Size, Subscription, Task, Theme, window};

use crate::config::{OvpnConfig, parse_ovpn};
use crate::openvpn::management::MgmtCommand;
use crate::openvpn::{AuthRequest, ConnectionInfo, LogEntry, VpnState};
use crate::settings::{self, Profile};
use crate::ui;
use crate::ui::tab_bar::Tab;

const ICON_BYTES: &[u8] = include_bytes!("../assets/logo.png");

pub fn run() -> iced::Result {
    let icon = match window::icon::from_file_data(ICON_BYTES, None) {
        Ok(icon) => Some(icon),
        Err(e) => {
            log::warn!("Failed to load window icon: {e}");
            None
        }
    };

    let window_settings = window::Settings {
        icon,
        size: Size::new(460.0, 720.0),
        min_size: Some(Size::new(420.0, 560.0)),
        platform_specific: window::settings::PlatformSpecific {
            application_id: "openvpn-gui-linux".to_string(),
            ..Default::default()
        },
        ..window::Settings::default()
    };

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
    remember_credentials: bool,
    otp_response: String,
    vpn_state: VpnState,
    connection_info: Option<ConnectionInfo>,
    log_lines: Vec<LogEntry>,
    log_content: text_editor::Content,
    enable_logs: bool,
    error_message: Option<String>,

    // Management socket state
    mgmt_socket_path: Option<PathBuf>,
    mgmt_cmd_tx: Option<tokio::sync::mpsc::UnboundedSender<MgmtCommand>>,
    mgmt_cmd_rx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<MgmtCommand>>>>,
    openvpn_pid: Option<u32>,
    subscription_id: u64,

    // UI state
    spinner_frame: u8,
    current_tab: Tab,
    theme: Theme,

    // Profile management
    profiles: Vec<Profile>,
    selected_profile_idx: Option<usize>,
    parsed_profiles: HashMap<PathBuf, OvpnConfig>,
    rename_state: Option<(usize, String)>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        // Load saved settings
        let saved = settings::load();

        let remember_credentials = saved.remember_credentials;
        let enable_logs = saved.enable_logs;
        let password = if remember_credentials {
            saved.password()
        } else {
            String::new()
        };
        let config_path = saved.config_path.map(PathBuf::from);
        let profiles = saved.profiles;
        let selected_profile_idx = config_path.as_ref().and_then(|path| {
            profiles
                .iter()
                .position(|p| Path::new(&p.path) == path.as_path())
        });
        let username = if remember_credentials {
            saved.username.unwrap_or_default()
        } else {
            String::new()
        };
        let theme = match saved.theme.as_str() {
            "Light" => Theme::Light,
            _ => Theme::Dark,
        };

        let mut app = Self {
            config_path: config_path.clone(),
            config: None,
            username,
            password,
            remember_credentials,
            otp_response: String::new(),
            vpn_state: VpnState::Disconnected,
            connection_info: None,
            log_lines: Vec::new(),
            log_content: text_editor::Content::new(),
            enable_logs,
            error_message: None,
            mgmt_socket_path: None,
            mgmt_cmd_tx: None,
            mgmt_cmd_rx: Arc::new(Mutex::new(None)),
            openvpn_pid: None,
            subscription_id: 0,
            spinner_frame: 0,
            current_tab: Tab::Connect,
            theme,
            profiles,
            selected_profile_idx,
            parsed_profiles: HashMap::new(),
            rename_state: None,
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
                app.error_message = Some("Saved config file no longer exists".to_string());
            }
        }

        // Pre-parse every known profile so the list can show host/proto/cipher.
        for profile in &app.profiles {
            let path = PathBuf::from(&profile.path);
            if path.exists() {
                tasks.push(Task::perform(load_profile_meta(path), |(p, r)| {
                    Message::ProfileMetaLoaded(p, r)
                }));
            }
        }

        (app, Task::batch(tasks))
    }

    fn save_settings(&self) {
        let (username, password_b64) = if self.remember_credentials {
            let username = (!self.username.is_empty()).then(|| self.username.clone());
            let mut tmp = settings::Settings::default();
            tmp.set_password(&self.password);
            (username, tmp.password_b64)
        } else {
            (None, None)
        };

        let theme_name = match self.theme {
            Theme::Light => "Light",
            _ => "Dark",
        };
        let s = settings::Settings {
            config_path: self.config_path.as_ref().map(|p| p.display().to_string()),
            username,
            password_b64,
            remember_credentials: self.remember_credentials,
            enable_logs: self.enable_logs,
            theme: theme_name.to_string(),
            profiles: self.profiles.clone(),
        };
        settings::save(&s);
    }

    fn upsert_profile(&mut self, path: PathBuf) -> usize {
        if let Some(idx) = self
            .profiles
            .iter()
            .position(|p| Path::new(&p.path) == path.as_path())
        {
            return idx;
        }
        self.profiles.push(Profile {
            path: path.display().to_string(),
            display_name: None,
            last_used: None,
        });
        self.profiles.len() - 1
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
    RememberCredentialsToggled(bool),
    EnableLogsToggled(bool),
    ThemeChanged(Theme),

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
    LogEditorAction(text_editor::Action),
    DismissError,
    CopyLogs,
    ClearLogs,
    TabChanged(Tab),

    // Profile management
    ProfileSelected(usize),
    ProfileConnectRequested(usize),
    ProfileRenameRequested(usize),
    ProfileRenameChanged(usize, String),
    ProfileRenameSubmitted(usize),
    ProfileRenameCancelled,
    ProfileRemoved(usize),
    ProfileMetaLoaded(PathBuf, Result<OvpnConfig, String>),

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
            Message::SelectConfig => Task::perform(pick_config_file(), Message::ConfigSelected),
            Message::ConfigSelected(path) => {
                if let Some(path) = path {
                    let idx = self.upsert_profile(path.clone());
                    self.selected_profile_idx = Some(idx);
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
                        if let Some(path) = &self.config_path {
                            self.parsed_profiles.insert(path.clone(), config.clone());
                        }
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
            Message::RememberCredentialsToggled(val) => {
                self.remember_credentials = val;
                self.save_settings();
                Task::none()
            }
            Message::EnableLogsToggled(val) => {
                self.enable_logs = val;
                if !val && self.current_tab == Tab::Logs {
                    self.current_tab = Tab::Connect;
                }
                self.save_settings();
                Task::none()
            }
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                self.save_settings();
                Task::none()
            }
            Message::Connect => {
                let config_path = match &self.config_path {
                    Some(p) => p.clone(),
                    None => return Task::none(),
                };

                if let Some(idx) = self.selected_profile_idx
                    && let Some(profile) = self.profiles.get_mut(idx)
                {
                    profile.last_used = Some(chrono::Local::now().timestamp());
                    self.save_settings();
                }

                self.error_message = None;
                self.vpn_state = VpnState::Spawning;
                self.log_lines.clear();
                self.log_content = text_editor::Content::new();
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
                    if let Some(code) = code
                        && code != 0
                    {
                        self.error_message =
                            Some(format!("OpenVPN process exited with code {code}"));
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
                            let _ = tx
                                .send(MgmtCommand::SendPrivateKeyPassphrase(self.password.clone()));
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
                self.log_content
                    .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
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
            Message::MgmtByteCount {
                bytes_in,
                bytes_out,
            } => {
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
            Message::LogEditorAction(action) => {
                // Only allow selection (read-only), not editing
                if action.is_edit() {
                    return Task::none();
                }
                self.log_content.perform(action);
                Task::none()
            }
            Message::Tick => {
                self.spinner_frame = self.spinner_frame.wrapping_add(1);
                Task::none()
            }
            Message::DismissError => {
                self.error_message = None;
                Task::none()
            }
            Message::CopyLogs => iced::clipboard::write(self.log_content.text()),
            Message::ClearLogs => {
                self.log_lines.clear();
                self.log_content = text_editor::Content::new();
                Task::none()
            }
            Message::TabChanged(tab) => {
                self.current_tab = tab;
                Task::none()
            }
            Message::ProfileSelected(idx) => {
                let Some(profile) = self.profiles.get(idx) else {
                    return Task::none();
                };
                let path = PathBuf::from(&profile.path);
                self.selected_profile_idx = Some(idx);
                self.config_path = Some(path.clone());
                self.rename_state = None;
                self.config = self.parsed_profiles.get(&path).cloned();
                self.save_settings();

                if self.config.is_none() && path.exists() {
                    Task::perform(load_and_parse_config(path), Message::ConfigParsed)
                } else {
                    Task::none()
                }
            }
            Message::ProfileConnectRequested(idx) => {
                let select_task = self.update(Message::ProfileSelected(idx));
                self.current_tab = Tab::Connect;
                select_task
            }
            Message::ProfileRenameRequested(idx) => {
                if let Some(profile) = self.profiles.get(idx) {
                    let initial = profile
                        .display_name
                        .clone()
                        .filter(|n| !n.is_empty())
                        .unwrap_or_else(|| {
                            PathBuf::from(&profile.path)
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| profile.path.clone())
                        });
                    self.rename_state = Some((idx, initial));
                }
                Task::none()
            }
            Message::ProfileRenameChanged(idx, value) => {
                if matches!(self.rename_state, Some((i, _)) if i == idx) {
                    self.rename_state = Some((idx, value));
                }
                Task::none()
            }
            Message::ProfileRenameSubmitted(idx) => {
                if let Some((state_idx, value)) = self.rename_state.take()
                    && state_idx == idx
                    && let Some(profile) = self.profiles.get_mut(idx)
                {
                    let trimmed = value.trim();
                    profile.display_name = if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    };
                    self.save_settings();
                }
                Task::none()
            }
            Message::ProfileRenameCancelled => {
                self.rename_state = None;
                Task::none()
            }
            Message::ProfileRemoved(idx) => {
                if idx >= self.profiles.len() {
                    return Task::none();
                }
                let removed = self.profiles.remove(idx);
                let removed_path = PathBuf::from(&removed.path);
                self.parsed_profiles.remove(&removed_path);
                self.rename_state = None;

                let was_selected = self.selected_profile_idx == Some(idx);
                self.selected_profile_idx = match self.selected_profile_idx {
                    Some(sel) if sel == idx => None,
                    Some(sel) if sel > idx => Some(sel - 1),
                    other => other,
                };

                if was_selected {
                    self.config_path = None;
                    self.config = None;
                    if self.vpn_state.is_active() {
                        // Disconnect before clearing — keep UI consistent.
                        // Spawn disconnect via re-dispatch.
                        self.save_settings();
                        return self.update(Message::Disconnect);
                    }
                }
                self.save_settings();
                Task::none()
            }
            Message::ProfileMetaLoaded(path, result) => {
                if let Ok(config) = result {
                    self.parsed_profiles.insert(path, config);
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let inputs_enabled = !self.vpn_state.is_active();

        let body: Element<'_, Message> = match self.current_tab {
            Tab::Connect => column![
                ui::connect_view::connect_body(
                    &self.config,
                    &self.username,
                    &self.password,
                    &self.otp_response,
                    self.remember_credentials,
                    self.spinner_frame,
                    &self.vpn_state,
                    &self.error_message,
                ),
                ui::status_bar::view(
                    &self.vpn_state,
                    &self.connection_info,
                    self.config.is_some(),
                ),
            ]
            .spacing(f32::from(ui::theme::SPACE_LG))
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            Tab::Profiles => ui::profiles_view::view(
                &self.profiles,
                &self.parsed_profiles,
                self.selected_profile_idx,
                &self.vpn_state,
                self.rename_state.as_ref().map(|(i, s)| (*i, s.as_str())),
            ),
            Tab::Logs => ui::log_view::view(&self.log_content, self.log_lines.len()),
            Tab::Settings => settings_tab(self.enable_logs, &self.theme),
        };

        let content = column![
            ui::connect_view::profile_card(&self.config_path, &self.config, inputs_enabled),
            ui::tab_bar::view(self.current_tab, self.enable_logs),
            body,
        ]
        .spacing(f32::from(ui::theme::SPACE_LG))
        .padding([ui::theme::SPACE_LG + 4, ui::theme::SPACE_LG + 8])
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![];

        if let Some(socket_path) = &self.mgmt_socket_path
            && self.vpn_state.is_active()
        {
            subs.push(crate::openvpn::management::management_subscription(
                socket_path.clone(),
                self.mgmt_cmd_rx.clone(),
                self.subscription_id,
            ));
        }

        // Tick drives both the connected-duration counter and the spinner glyph
        // during transitional states.
        if self.vpn_state == VpnState::Connected {
            subs.push(iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick));
        } else if self.vpn_state.is_active() {
            subs.push(iced::time::every(Duration::from_millis(120)).map(|_| Message::Tick));
        }

        Subscription::batch(subs)
    }

    fn cleanup_connection_state(&mut self) {
        self.mgmt_cmd_tx = None;
        self.openvpn_pid = None;
    }
}

fn settings_tab<'a>(enable_logs: bool, current_theme: &Theme) -> Element<'a, Message> {
    use iced::widget::toggler;

    let light_active = matches!(current_theme, Theme::Light);

    let theme_toggle = toggler(light_active)
        .label("Light theme")
        .on_toggle(|enabled| {
            Message::ThemeChanged(if enabled { Theme::Light } else { Theme::Dark })
        })
        .size(22)
        .text_size(14);
    let theme_help = text("Switch between dark and light color schemes.")
        .size(12)
        .style(ui::theme::text_muted);

    let appearance_card = container(
        column![theme_toggle, theme_help]
            .spacing(f32::from(ui::theme::SPACE_XS))
            .width(Length::Fill),
    )
    .padding(ui::theme::SPACE_MD)
    .width(Length::Fill)
    .style(ui::theme::card);

    let logs_toggle = toggler(enable_logs)
        .label("Enable log output")
        .on_toggle(Message::EnableLogsToggled)
        .size(22)
        .text_size(14);
    let logs_help = text("Show a Logs tab for viewing OpenVPN output.")
        .size(12)
        .style(ui::theme::text_muted);

    let logs_card = container(
        column![logs_toggle, logs_help]
            .spacing(f32::from(ui::theme::SPACE_XS))
            .width(Length::Fill),
    )
    .padding(ui::theme::SPACE_MD)
    .width(Length::Fill)
    .style(ui::theme::card);

    column![appearance_card, logs_card]
        .spacing(f32::from(ui::theme::SPACE_MD))
        .width(Length::Fill)
        .into()
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

async fn load_profile_meta(path: PathBuf) -> (PathBuf, Result<OvpnConfig, String>) {
    let result = load_and_parse_config(path.clone()).await;
    (path, result)
}
