use std::collections::{HashMap, HashSet};
use std::process::Command;

/// Session status for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    NotLoggedIn,
    Active,
    Idle,
}

/// Parse one line of `loginctl list-sessions --no-legend`.
/// Returns (session_id, username) if the line has enough columns.
pub fn parse_session_line(line: &str) -> Option<(&str, &str)> {
    // Format: SESSION UID USER SEAT TTY
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() >= 3 {
        Some((cols[0], cols[2]))
    } else {
        None
    }
}

/// Parse Type=, Class=, State=, and IdleHint= properties from `loginctl show-session` output.
/// Returns (is_graphical, is_user_class, is_active, is_idle).
pub fn parse_session_properties(output: &str) -> (bool, bool, bool, bool) {
    let mut is_gui = false;
    let mut is_user = false;
    let mut is_active = false;
    let mut is_idle = false;
    for line in output.lines() {
        match line.trim() {
            "Type=x11" | "Type=wayland" | "Type=mir" => is_gui = true,
            "Class=user" => is_user = true,
            "State=active" => is_active = true,
            "IdleHint=yes" => is_idle = true,
            _ => {}
        }
    }
    (is_gui, is_user, is_active, is_idle)
}

/// Parse Display= property from loginctl show-session output.
pub fn parse_display_property(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(val) = line.strip_prefix("Display=") {
            let val = val.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Returns the set of usernames that have active graphical sessions (x11 or wayland).
pub fn active_graphical_users() -> HashSet<String> {
    let mut users = HashSet::new();

    let output = match Command::new("loginctl")
        .args(["list-sessions", "--no-legend", "--no-pager"])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            log::warn!("failed to run loginctl: {e}");
            return users;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some((session_id, username)) = parse_session_line(line) {
            if is_graphical_session(session_id) {
                users.insert(username.to_string());
            }
        }
    }
    users
}

fn is_graphical_session(session_id: &str) -> bool {
    let output = match Command::new("loginctl")
        .args(["show-session", session_id, "--no-pager",
               "--property=Type", "--property=Class",
               "--property=State", "--property=IdleHint"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return false,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let (is_gui, is_user, is_active, is_idle) = parse_session_properties(&stdout);
    is_gui && is_user && is_active && !is_idle
}

/// Returns the session status for each managed user that has a graphical session.
pub fn graphical_user_statuses(usernames: &[&str]) -> HashMap<String, SessionStatus> {
    let mut result: HashMap<String, SessionStatus> = HashMap::new();

    let output = match Command::new("loginctl")
        .args(["list-sessions", "--no-legend", "--no-pager"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return result,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some((session_id, username)) = parse_session_line(line) {
            if !usernames.contains(&username) {
                continue;
            }
            let show = match Command::new("loginctl")
                .args(["show-session", session_id, "--no-pager",
                       "--property=Type", "--property=Class",
                       "--property=State", "--property=IdleHint"])
                .output()
            {
                Ok(o) => o,
                Err(_) => continue,
            };
            let out = String::from_utf8_lossy(&show.stdout);
            let (is_gui, is_user, is_active, is_idle) = parse_session_properties(&out);
            if is_gui && is_user {
                let status = if !is_active {
                    SessionStatus::Idle // online but not foreground
                } else if is_idle {
                    SessionStatus::Idle
                } else {
                    SessionStatus::Active
                };
                // Active wins over Idle if multiple sessions
                let entry = result.entry(username.to_string()).or_insert(status);
                if status == SessionStatus::Active {
                    *entry = SessionStatus::Active;
                }
            }
        }
    }
    result
}

/// Get the DISPLAY for a user's session (needed for zenity).
pub fn get_session_display(username: &str) -> Option<String> {
    let output = Command::new("loginctl")
        .args(["list-sessions", "--no-legend", "--no-pager"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some((session_id, user)) = parse_session_line(line) {
            if user == username {
                let show = Command::new("loginctl")
                    .args(["show-session", session_id, "--no-pager", "--property=Display"])
                    .output()
                    .ok()?;
                let out = String::from_utf8_lossy(&show.stdout);
                if let Some(display) = parse_display_property(&out) {
                    return Some(display);
                }
            }
        }
    }
    None
}
