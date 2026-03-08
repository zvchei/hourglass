use crate::config::Config;
use crate::state::State;
use std::process;

/// Called by pam_exec during login. Checks if the user has exhausted their daily limit.
/// Exit 0 = allow login, exit 1 = deny.
pub fn pam_check() {
    let username = match std::env::var("PAM_USER") {
        Ok(u) => u,
        Err(_) => process::exit(0), // no PAM_USER → not our concern
    };

    let config = Config::load();
    let user_config = match config.users.get(&username) {
        Some(c) => c,
        None => process::exit(0), // user not managed by hourglass
    };

    let state = State::load();
    let limit_seconds = user_config.daily_limit_minutes as u64 * 60;

    if let Some(user_state) = state.users.get(&username) {
        let today = chrono::Local::now().date_naive();
        if user_state.date == today && user_state.remaining(limit_seconds) <= 0 {
            eprintln!("hourglass: daily screen time limit reached. Try again tomorrow.");
            process::exit(1);
        }
    }

    process::exit(0);
}
