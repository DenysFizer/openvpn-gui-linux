mod app;
mod config;
mod error;
mod openvpn;
mod settings;
mod ui;

fn main() -> iced::Result {
    env_logger::init();
    inherit_system_cursor_theme();
    app::run()
}

// winit falls back to a built-in cursor when XCURSOR_THEME/XCURSOR_SIZE are unset,
// which causes the cursor to visibly change when moving into the window on GNOME
// Wayland. Query GSettings and export the values before the event loop starts.
fn inherit_system_cursor_theme() {
    use std::process::Command;

    fn gsettings_get(key: &str) -> Option<String> {
        let output = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", key])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let raw = String::from_utf8(output.stdout).ok()?;
        Some(raw.trim().trim_matches('\'').to_string())
    }

    if std::env::var_os("XCURSOR_THEME").is_none()
        && let Some(theme) = gsettings_get("cursor-theme").filter(|s| !s.is_empty())
    {
        // SAFETY: single-threaded at startup, before iced spawns any threads.
        unsafe { std::env::set_var("XCURSOR_THEME", theme) };
    }

    if std::env::var_os("XCURSOR_SIZE").is_none()
        && let Some(size) = gsettings_get("cursor-size").filter(|s| !s.is_empty())
    {
        unsafe { std::env::set_var("XCURSOR_SIZE", size) };
    }
}
