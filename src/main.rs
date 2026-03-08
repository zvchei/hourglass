use clap::{Parser, Subcommand};
use hourglass::config::{Config, UserConfig};
use hourglass::state::State;
use hourglass::session::{self, SessionStatus};
use hourglass::{daemon, pam};

#[derive(Parser)]
#[command(name = "hourglass", about = "Linux screen time limiter")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the enforcement daemon (requires root)
    Daemon,
    /// Set daily time limit for a user
    SetLimit {
        /// Username
        user: String,
        /// Daily limit in minutes
        minutes: u32,
    },
    /// Remove time limit for a user
    RemoveLimit {
        /// Username
        user: String,
    },
    /// Grant extra minutes for today
    Extend {
        /// Username
        user: String,
        /// Extra minutes to grant
        minutes: u32,
    },
    /// Show status for one or all managed users
    Status {
        /// Username (omit for all users)
        user: Option<String>,
    },
    /// PAM check (called by pam_exec, not for manual use)
    PamCheck,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Daemon => daemon::run(),
        Commands::SetLimit { user, minutes } => cmd_set_limit(&user, minutes),
        Commands::RemoveLimit { user } => cmd_remove_limit(&user),
        Commands::Extend { user, minutes } => cmd_extend(&user, minutes),
        Commands::Status { user } => {
            cmd_status(user.as_deref());
            Ok(())
        }
        Commands::PamCheck => {
            pam::pam_check();
            Ok(()) // unreachable, pam_check calls process::exit
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn cmd_set_limit(user: &str, minutes: u32) -> anyhow::Result<()> {
    let mut config = Config::load();
    config.users.insert(
        user.to_string(),
        UserConfig {
            daily_limit_minutes: minutes,
        },
    );
    config.save()?;
    println!("Set daily limit for {user} to {minutes} minutes.");
    Ok(())
}

fn cmd_remove_limit(user: &str) -> anyhow::Result<()> {
    let mut config = Config::load();
    if config.users.remove(user).is_some() {
        config.save()?;
        println!("Removed limit for {user}.");
    } else {
        println!("{user} has no limit configured.");
    }
    Ok(())
}

fn cmd_extend(user: &str, minutes: u32) -> anyhow::Result<()> {
    let extra = minutes as u64 * 60;
    State::with_lock(|state| {
        let us = state.get_mut(user);
        us.ensure_today();
        us.extension_seconds += extra;
        println!("Granted {minutes} extra minutes to {user} (total extension: {}m).",
            us.extension_seconds / 60);
        Ok(())
    })
}

fn cmd_status(user: Option<&str>) {
    let config = Config::load();
    let state = State::load();
    let today = chrono::Local::now().date_naive();

    let users: Vec<&str> = match user {
        Some(u) => vec![u],
        None => config.users.keys().map(|s| s.as_str()).collect(),
    };

    if users.is_empty() {
        println!("No users configured.");
        return;
    }

    let statuses = session::graphical_user_statuses(&users);

    for username in users {
        let limit = config
            .users
            .get(username)
            .map(|c| c.daily_limit_minutes)
            .unwrap_or(0);
        let limit_s = limit as u64 * 60;

        let (used, ext, remaining) = match state.users.get(username) {
            Some(us) if us.date == today => {
                let r = us.remaining(limit_s);
                (us.used_seconds, us.extension_seconds, r)
            }
            _ => (0, 0, limit_s as i64),
        };

        let status = if remaining <= 0 {
            "LOCKED OUT"
        } else {
            match statuses.get(username) {
                Some(SessionStatus::Active) => "ACTIVE",
                Some(SessionStatus::Idle) => "IDLE",
                _ => "NOT LOGGED IN",
            }
        };

        println!("{username}:");
        println!("  limit:     {limit}m");
        println!("  used:      {}m {}s", used / 60, used % 60);
        println!("  extension: {}m", ext / 60);
        println!("  remaining: {}m {}s", remaining / 60, remaining.unsigned_abs() % 60);
        println!("  status:    {status}");
    }
}
