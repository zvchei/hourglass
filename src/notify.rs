use nix::unistd::User;
use std::process::Command;

use crate::session;

/// Send a notification to a user's graphical session.
/// Tries zenity (modal splash) first, then notify-send (toast).
/// Best-effort: logs failures, never panics.
pub fn notify_user(username: &str, title: &str, message: &str) {
    let uid = match User::from_name(username) {
        Ok(Some(u)) => u.uid,
        _ => {
            log::warn!("cannot resolve uid for {username}");
            return;
        }
    };

    let display = session::get_session_display(username).unwrap_or_else(|| ":0".into());
    let dbus_addr = format!("unix:path=/run/user/{}/bus", uid);

    // Try zenity first (modal, most visible)
    if try_zenity(username, &display, &dbus_addr, title, message) {
        return;
    }
    // Fallback: notify-send (toast)
    if try_notify_send(username, &display, &dbus_addr, title, message) {
        return;
    }
    log::warn!("all notification methods failed for {username}");
}

fn try_zenity(user: &str, display: &str, dbus: &str, title: &str, msg: &str) -> bool {
    let status = Command::new("runuser")
        .args(["-u", user, "--"])
        .args(["zenity", "--info", "--title", title, "--text", msg, "--timeout", "30"])
        .env("DISPLAY", display)
        .env("DBUS_SESSION_BUS_ADDRESS", dbus)
        .status();
    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

fn try_notify_send(user: &str, display: &str, dbus: &str, title: &str, msg: &str) -> bool {
    let status = Command::new("runuser")
        .args(["-u", user, "--"])
        .args(["notify-send", "--urgency=critical", title, msg])
        .env("DISPLAY", display)
        .env("DBUS_SESSION_BUS_ADDRESS", dbus)
        .status();
    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}
