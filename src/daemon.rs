use anyhow::Result;
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::notify;
use crate::session;
use crate::state::State;

const TICK_SECONDS: u64 = 10;
const WARNING_THRESHOLDS: &[u64] = &[900, 300, 60]; // 15min, 5min, 1min

pub fn run() -> Result<()> {
    log::info!("hourglass daemon starting, tick={}s", TICK_SECONDS);

    loop {
        if let Err(e) = tick() {
            log::error!("tick error: {e}");
        }
        thread::sleep(Duration::from_secs(TICK_SECONDS));
    }
}

fn tick() -> Result<()> {
    let config = Config::load();
    let active_users = session::active_graphical_users();

    State::with_lock(|state| {
        for (username, user_config) in &config.users {
            if !active_users.contains(username) {
                continue;
            }

            let limit_seconds = user_config.daily_limit_minutes as u64 * 60;
            let us = state.get_mut(username);
            us.ensure_today();
            us.used_seconds += TICK_SECONDS;

            let remaining = us.remaining(limit_seconds);
            log::debug!(
                "{username}: used={}s limit={}s ext={}s remaining={}s",
                us.used_seconds, limit_seconds, us.extension_seconds, remaining
            );

            // Check warning thresholds
            for &threshold in WARNING_THRESHOLDS {
                if remaining <= threshold as i64 && remaining > 0 && !us.warnings_sent.contains(&threshold) {
                    let mins = threshold / 60;
                    let msg = if mins > 0 {
                        format!("You have {mins} minute(s) of screen time remaining.")
                    } else {
                        format!("You have {threshold} seconds of screen time remaining.")
                    };
                    log::info!("{username}: warning at {threshold}s remaining");
                    notify::notify_user(username, "⏳ Hourglass", &msg);
                    us.warnings_sent.push(threshold);
                }
            }

            // Time's up
            if remaining <= 0 {
                log::info!("{username}: time expired, terminating session");
                notify::notify_user(
                    username,
                    "⏳ Hourglass",
                    "Your screen time is up. Logging you out now.",
                );
                // Small delay so notification can be seen
                thread::sleep(Duration::from_secs(3));
                terminate_user(username);
            }
        }
        Ok(())
    })
}

fn terminate_user(username: &str) {
    let status = Command::new("loginctl")
        .args(["terminate-user", username])
        .status();
    match status {
        Ok(s) if s.success() => log::info!("terminated sessions for {username}"),
        Ok(s) => log::warn!("loginctl terminate-user {username} exited with {s}"),
        Err(e) => log::error!("failed to run loginctl: {e}"),
    }
}
