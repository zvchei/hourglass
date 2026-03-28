use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;

pub const STATE_PATH: &str = "/var/lib/hourglass/state.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    #[serde(flatten)]
    pub users: HashMap<String, UserState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserState {
    pub date: NaiveDate,
    pub used_seconds: u64,
    pub extension_seconds: i64,
    #[serde(default)]
    pub warnings_sent: Vec<u64>,
}

impl UserState {
    pub fn new_today() -> Self {
        Self {
            date: chrono::Local::now().date_naive(),
            used_seconds: 0,
            extension_seconds: 0,
            warnings_sent: Vec::new(),
        }
    }

    pub fn ensure_today(&mut self) {
        let today = chrono::Local::now().date_naive();
        if self.date != today {
            *self = Self::new_today();
        }
    }

    pub fn remaining(&self, limit_seconds: u64) -> i64 {
        limit_seconds as i64 + self.extension_seconds - self.used_seconds as i64
    }
}

impl State {
    pub fn load() -> State {
        let path = Path::new(STATE_PATH);
        if !path.exists() {
            return State::default();
        }
        let text = fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Path::new(STATE_PATH);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        // Write to temp file then rename for atomicity
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn get_mut(&mut self, user: &str) -> &mut UserState {
        self.users
            .entry(user.to_string())
            .or_insert_with(UserState::new_today)
    }

    /// Load state with an advisory flock, run a closure, save, and release.
    pub fn with_lock<F, R>(f: F) -> Result<R>
    where
        F: FnOnce(&mut State) -> Result<R>,
    {
        let lock_path = Path::new(STATE_PATH).with_extension("lock");
        if let Some(dir) = lock_path.parent() {
            fs::create_dir_all(dir)?;
        }
        let lock_file = File::create(&lock_path)?;
        let _lock = nix::fcntl::Flock::lock(lock_file, nix::fcntl::FlockArg::LockExclusive)
            .map_err(|(_, e)| e)?;

        let mut state = State::load();
        let result = f(&mut state)?;
        state.save()?;

        Ok(result)
        // _lock dropped here → automatically unlocked
    }
}
