use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::Local;
use iced::Subscription;
use iced::futures::{SinkExt, Stream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;

use crate::app::Message;
use crate::openvpn::types::{AuthRequest, ConnectionInfo, LogEntry, LogLevel, VpnState};

/// Commands that the UI can send to the management socket
#[derive(Debug, Clone)]
pub enum MgmtCommand {
    SendCredentials {
        username: String,
        password: String,
        otp: Option<String>,
    },
    SendPrivateKeyPassphrase(String),
    Signal(String),
    HoldRelease,
}

/// Shared state for the management subscription, passed via `run_with`.
#[derive(Clone, Hash)]
pub struct MgmtSubscriptionId {
    pub socket_path: PathBuf,
    pub id: u64,
}

/// Create a subscription that connects to the OpenVPN management socket.
/// Returns a Stream<Item = Message> that the subscription will consume.
pub fn management_subscription(
    socket_path: PathBuf,
    cmd_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<MgmtCommand>>>>,
    id: u64,
) -> Subscription<Message> {
    let sub_id = MgmtSubscriptionId { socket_path, id };

    struct MgmtRecipe {
        id: MgmtSubscriptionId,
        cmd_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<MgmtCommand>>>>,
    }

    impl iced::advanced::subscription::Recipe for MgmtRecipe {
        type Output = Message;

        fn hash(&self, state: &mut iced::advanced::subscription::Hasher) {
            use std::hash::Hash;
            self.id.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: iced::advanced::subscription::EventStream,
        ) -> std::pin::Pin<Box<dyn Stream<Item = Self::Output> + Send>> {
            let socket_path = self.id.socket_path;
            let cmd_rx = self.cmd_rx;

            Box::pin(iced::stream::channel(
                100,
                move |mut output: iced::futures::channel::mpsc::Sender<Self::Output>| async move {
                    // Take the receiver out of the shared slot
                    let mut cmd_receiver =
                        cmd_rx.lock().unwrap().take().expect("cmd_rx already taken");

                    // Retry connecting to the socket
                    let stream = loop {
                        match UnixStream::connect(&socket_path).await {
                            Ok(s) => break s,
                            Err(_) => {
                                tokio::time::sleep(Duration::from_millis(200)).await;
                            }
                        }
                    };

                    let _ = output.send(Message::MgmtConnected).await;

                    let (reader, mut writer) = stream.into_split();
                    let mut lines = BufReader::new(reader).lines();

                    // Send initial commands
                    let init = "state on all\nlog on all\nbytecount 5\n";
                    if let Err(e) = writer.write_all(init.as_bytes()).await {
                        let _ = output
                            .send(Message::MgmtError(format!(
                                "Failed to send init commands: {e}"
                            )))
                            .await;
                        return;
                    }

                    loop {
                        tokio::select! {
                            line_result = lines.next_line() => {
                                match line_result {
                                    Ok(Some(line)) => {
                                        if let Some(msg) = parse_management_line(&line) {
                                            let _ = output.send(msg).await;
                                        }
                                    }
                                    Ok(None) => {
                                        let _ = output.send(Message::MgmtDisconnected).await;
                                        return;
                                    }
                                    Err(e) => {
                                        let _ = output
                                            .send(Message::MgmtError(format!(
                                                "Socket read error: {e}"
                                            )))
                                            .await;
                                        let _ = output.send(Message::MgmtDisconnected).await;
                                        return;
                                    }
                                }
                            }
                            cmd = cmd_receiver.recv() => {
                                match cmd {
                                    Some(command) => {
                                        let data = format_command(&command);
                                        if let Err(e) = writer.write_all(data.as_bytes()).await {
                                            let _ = output
                                                .send(Message::MgmtError(format!(
                                                    "Failed to send command: {e}"
                                                )))
                                                .await;
                                        }
                                    }
                                    None => {
                                        // Command channel closed
                                        return;
                                    }
                                }
                            }
                        }
                    }
                },
            ))
        }
    }

    iced::advanced::subscription::from_recipe(MgmtRecipe { id: sub_id, cmd_rx })
}

/// Format a MgmtCommand into the wire protocol string(s) to send
fn format_command(cmd: &MgmtCommand) -> String {
    match cmd {
        MgmtCommand::SendCredentials {
            username,
            password,
            otp,
        } => {
            let password_str = match otp {
                Some(otp_val) => encode_scrv1(password, otp_val),
                None => password.clone(),
            };
            format!("username \"Auth\" {username}\npassword \"Auth\" {password_str}\n")
        }
        MgmtCommand::SendPrivateKeyPassphrase(passphrase) => {
            format!("password \"Private Key\" {passphrase}\n")
        }
        MgmtCommand::Signal(sig) => {
            format!("signal {sig}\n")
        }
        MgmtCommand::HoldRelease => "hold release\n".to_string(),
    }
}

/// Encode password and OTP response using the SCRV1 protocol for static-challenge.
/// Format: SCRV1:<base64(password)>:<base64(otp)>
pub fn encode_scrv1(password: &str, otp: &str) -> String {
    let pw_b64 = BASE64.encode(password.as_bytes());
    let otp_b64 = BASE64.encode(otp.as_bytes());
    format!("SCRV1:{pw_b64}:{otp_b64}")
}

/// Parse a single line from the management interface into a Message.
pub fn parse_management_line(line: &str) -> Option<Message> {
    let line = line.trim();

    if let Some(rest) = line.strip_prefix(">STATE:") {
        return Some(parse_state_line(rest));
    }

    if let Some(rest) = line.strip_prefix(">PASSWORD:") {
        return Some(parse_password_line(rest));
    }

    if let Some(rest) = line.strip_prefix(">LOG:") {
        return parse_log_line(rest);
    }

    if line.starts_with(">HOLD:") {
        return Some(Message::MgmtHoldRequest);
    }

    if let Some(rest) = line.strip_prefix(">BYTECOUNT:") {
        return parse_bytecount_line(rest);
    }

    if let Some(rest) = line.strip_prefix(">FATAL:") {
        return Some(Message::MgmtStateChanged(VpnState::Error(rest.to_string())));
    }

    None
}

fn parse_state_line(rest: &str) -> Message {
    let parts: Vec<&str> = rest.split(',').collect();

    let state_name = parts.get(1).unwrap_or(&"UNKNOWN");
    let description = parts.get(2).unwrap_or(&"");
    let local_ip = parts.get(3).and_then(|s| non_empty(s));
    let remote_ip = parts.get(4).and_then(|s| non_empty(s));

    let state = match *state_name {
        "CONNECTING" | "TCP_CONNECT" | "WAIT" | "RESOLVE" => VpnState::Connecting,
        "AUTH" => VpnState::Authenticating,
        "GET_CONFIG" => VpnState::GettingConfig,
        "ASSIGN_IP" => VpnState::AssigningIp,
        "ADD_ROUTES" => VpnState::AddingRoutes,
        "CONNECTED" => VpnState::Connected,
        "RECONNECTING" => VpnState::Reconnecting(description.to_string()),
        "EXITING" => VpnState::Exiting(description.to_string()),
        _ => VpnState::Connecting,
    };

    if state == VpnState::Connected {
        Message::MgmtStateConnected(ConnectionInfo {
            local_ip: local_ip.map(String::from),
            remote_ip: remote_ip.map(String::from),
            remote_port: None,
            connected_since: Some(Local::now()),
            bytes_in: 0,
            bytes_out: 0,
        })
    } else {
        Message::MgmtStateChanged(state)
    }
}

fn parse_password_line(rest: &str) -> Message {
    if rest.starts_with("Verification Failed") {
        return Message::MgmtError("Authentication failed. Check your credentials.".to_string());
    }

    if rest.contains("Need 'Private Key'") {
        return Message::MgmtPasswordRequest(AuthRequest::PrivateKey);
    }

    if rest.contains("Need 'Auth'") {
        if let Some(sc_pos) = rest.find("SC:") {
            let sc_data = &rest[sc_pos + 3..];
            let (echo_str, prompt) = sc_data.split_once(',').unwrap_or(("0", "Enter code"));
            let echo = echo_str == "1";
            return Message::MgmtPasswordRequest(AuthRequest::UserPassWithChallenge {
                prompt: prompt.to_string(),
                echo,
            });
        }
        return Message::MgmtPasswordRequest(AuthRequest::UserPass);
    }

    Message::MgmtError(format!("Unexpected password event: {rest}"))
}

fn parse_log_line(rest: &str) -> Option<Message> {
    let parts: Vec<&str> = rest.splitn(3, ',').collect();
    if parts.len() < 3 {
        return None;
    }

    let flags = parts[1];
    let message = parts[2].to_string();

    let level = match flags {
        "I" => LogLevel::Info,
        "W" => LogLevel::Warning,
        "N" | "E" => LogLevel::Error,
        "F" => LogLevel::Fatal,
        "D" => LogLevel::Debug,
        _ => LogLevel::Info,
    };

    Some(Message::MgmtLogLine(LogEntry {
        timestamp: Local::now(),
        level,
        message,
    }))
}

fn parse_bytecount_line(rest: &str) -> Option<Message> {
    let (in_str, out_str) = rest.split_once(',')?;
    let bytes_in = in_str.trim().parse::<u64>().ok()?;
    let bytes_out = out_str.trim().parse::<u64>().ok()?;
    Some(Message::MgmtByteCount {
        bytes_in,
        bytes_out,
    })
}

fn non_empty(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_scrv1() {
        let result = encode_scrv1("mypass", "123456");
        let pw_b64 = BASE64.encode(b"mypass");
        let otp_b64 = BASE64.encode(b"123456");
        assert_eq!(result, format!("SCRV1:{pw_b64}:{otp_b64}"));
    }

    #[test]
    fn test_parse_state_connecting() {
        let msg = parse_management_line(">STATE:1618000000,CONNECTING,,,,,,,").unwrap();
        match msg {
            Message::MgmtStateChanged(VpnState::Connecting) => {}
            other => panic!("Expected MgmtStateChanged(Connecting), got {other:?}"),
        }
    }

    #[test]
    fn test_parse_state_connected() {
        let msg =
            parse_management_line(">STATE:1618000000,CONNECTED,SUCCESS,10.8.0.6,203.0.113.1,,,")
                .unwrap();
        match msg {
            Message::MgmtStateConnected(info) => {
                assert_eq!(info.local_ip.as_deref(), Some("10.8.0.6"));
                assert_eq!(info.remote_ip.as_deref(), Some("203.0.113.1"));
            }
            other => panic!("Expected MgmtStateConnected, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_state_reconnecting() {
        let msg =
            parse_management_line(">STATE:1618000000,RECONNECTING,ping-restart,,,,,").unwrap();
        match msg {
            Message::MgmtStateChanged(VpnState::Reconnecting(reason)) => {
                assert_eq!(reason, "ping-restart");
            }
            other => panic!("Expected Reconnecting, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_state_exiting() {
        let msg = parse_management_line(">STATE:1618000000,EXITING,SIGTERM,,,,,").unwrap();
        match msg {
            Message::MgmtStateChanged(VpnState::Exiting(reason)) => {
                assert_eq!(reason, "SIGTERM");
            }
            other => panic!("Expected Exiting, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_password_request_basic() {
        let msg = parse_management_line(">PASSWORD:Need 'Auth' username/password").unwrap();
        match msg {
            Message::MgmtPasswordRequest(AuthRequest::UserPass) => {}
            other => panic!("Expected UserPass request, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_password_request_static_challenge() {
        let msg = parse_management_line(
            ">PASSWORD:Need 'Auth' username/password SC:1,Enter your OTP token",
        )
        .unwrap();
        match msg {
            Message::MgmtPasswordRequest(AuthRequest::UserPassWithChallenge { prompt, echo }) => {
                assert_eq!(prompt, "Enter your OTP token");
                assert!(echo);
            }
            other => panic!("Expected UserPassWithChallenge, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_password_request_private_key() {
        let msg = parse_management_line(">PASSWORD:Need 'Private Key' password").unwrap();
        match msg {
            Message::MgmtPasswordRequest(AuthRequest::PrivateKey) => {}
            other => panic!("Expected PrivateKey request, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_password_verification_failed() {
        let msg = parse_management_line(">PASSWORD:Verification Failed: 'Auth'").unwrap();
        match msg {
            Message::MgmtError(e) => {
                assert!(e.contains("Authentication failed"));
            }
            other => panic!("Expected MgmtError, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_log_line() {
        let msg =
            parse_management_line(">LOG:1618000000,I,Initialization Sequence Completed").unwrap();
        match msg {
            Message::MgmtLogLine(entry) => {
                assert_eq!(entry.level, LogLevel::Info);
                assert_eq!(entry.message, "Initialization Sequence Completed");
            }
            other => panic!("Expected MgmtLogLine, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_bytecount() {
        let msg = parse_management_line(">BYTECOUNT:1234,5678").unwrap();
        match msg {
            Message::MgmtByteCount {
                bytes_in,
                bytes_out,
            } => {
                assert_eq!(bytes_in, 1234);
                assert_eq!(bytes_out, 5678);
            }
            other => panic!("Expected MgmtByteCount, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_hold() {
        let msg = parse_management_line(">HOLD:Waiting for hold release:10").unwrap();
        match msg {
            Message::MgmtHoldRequest => {}
            other => panic!("Expected MgmtHoldRequest, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_fatal() {
        let msg = parse_management_line(">FATAL:Cannot open TUN/TAP device").unwrap();
        match msg {
            Message::MgmtStateChanged(VpnState::Error(e)) => {
                assert_eq!(e, "Cannot open TUN/TAP device");
            }
            other => panic!("Expected Error state, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_unknown_line() {
        let msg = parse_management_line("SUCCESS: real-time state notification set to ON");
        assert!(msg.is_none());
    }

    #[test]
    fn test_format_credentials_no_otp() {
        let cmd = MgmtCommand::SendCredentials {
            username: "user".to_string(),
            password: "pass".to_string(),
            otp: None,
        };
        let output = format_command(&cmd);
        assert!(output.contains("username \"Auth\" user"));
        assert!(output.contains("password \"Auth\" pass"));
    }

    #[test]
    fn test_format_credentials_with_otp() {
        let cmd = MgmtCommand::SendCredentials {
            username: "user".to_string(),
            password: "pass".to_string(),
            otp: Some("123456".to_string()),
        };
        let output = format_command(&cmd);
        assert!(output.contains("username \"Auth\" user"));
        assert!(output.contains("SCRV1:"));
    }

    #[test]
    fn test_format_signal() {
        let cmd = MgmtCommand::Signal("SIGTERM".to_string());
        assert_eq!(format_command(&cmd), "signal SIGTERM\n");
    }

    #[test]
    fn test_format_hold_release() {
        let cmd = MgmtCommand::HoldRelease;
        assert_eq!(format_command(&cmd), "hold release\n");
    }
}
