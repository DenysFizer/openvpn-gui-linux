use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::error::AppError;

/// Spawn the OpenVPN process via pkexec with management socket.
/// Returns (child PID, socket path).
pub async fn spawn_openvpn(
    config_path: &Path,
    socket_path: &Path,
) -> Result<u32, AppError> {
    // Ensure the socket file does not already exist
    let _ = tokio::fs::remove_file(socket_path).await;

    let child = Command::new("pkexec")
        .arg("openvpn")
        .arg("--config")
        .arg(config_path)
        .arg("--management")
        .arg(socket_path)
        .arg("unix")
        .arg("--management-hold")
        .arg("--management-query-passwords")
        .arg("--management-log-cache")
        .arg("200")
        .arg("--verb")
        .arg("3")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| AppError::ProcessSpawn(format!("Failed to launch pkexec: {e}")))?;

    let pid = child.id().ok_or_else(|| {
        AppError::ProcessSpawn("Failed to get process ID".to_string())
    })?;

    // Spawn a background task to wait for the child process to exit
    tokio::spawn(async move {
        let _ = child;
        // The child will be dropped when this task scope ends,
        // but we don't wait here — we manage via the management socket
    });

    Ok(pid)
}

/// Generate a unique management socket path in /tmp.
pub fn generate_socket_path() -> PathBuf {
    let id = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    PathBuf::from(format!("/tmp/ovpn-mgmt-{id}-{ts}.sock"))
}

/// Clean up the management socket file.
pub async fn cleanup_socket(socket_path: &Path) {
    let _ = tokio::fs::remove_file(socket_path).await;
}

/// Stop openvpn by sending SIGTERM to the process via kill.
/// This is a fallback if the management socket is unresponsive.
pub async fn force_stop(pid: u32) {
    // Try SIGTERM first
    let _ = Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .output()
        .await;

    // Wait a bit, then SIGKILL if still alive
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let _ = Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .await
        .and_then(|output| {
            if output.status.success() {
                // Process still alive, force kill
                std::process::Command::new("kill")
                    .arg("-KILL")
                    .arg(pid.to_string())
                    .output()
            } else {
                Ok(output)
            }
        });
}

/// Check if openvpn binary is available on the system.
pub async fn check_openvpn_installed() -> bool {
    Command::new("which")
        .arg("openvpn")
        .output()
        .await
        .is_ok_and(|o| o.status.success())
}
