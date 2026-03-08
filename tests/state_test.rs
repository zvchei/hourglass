use chrono::NaiveDate;
use hourglass::state::{State, UserState};

// --- UserState::new_today ---

#[test]
fn new_today_has_todays_date() {
    let us = UserState::new_today();
    let today = chrono::Local::now().date_naive();
    assert_eq!(us.date, today);
}

#[test]
fn new_today_starts_at_zero() {
    let us = UserState::new_today();
    assert_eq!(us.used_seconds, 0);
    assert_eq!(us.extension_seconds, 0);
    assert!(us.warnings_sent.is_empty());
}

// --- UserState::remaining ---

#[test]
fn remaining_full_limit() {
    let us = UserState {
        date: chrono::Local::now().date_naive(),
        used_seconds: 0,
        extension_seconds: 0,
        warnings_sent: vec![],
    };
    assert_eq!(us.remaining(7200), 7200); // 2 hours
}

#[test]
fn remaining_partial_use() {
    let us = UserState {
        date: chrono::Local::now().date_naive(),
        used_seconds: 3600,
        extension_seconds: 0,
        warnings_sent: vec![],
    };
    assert_eq!(us.remaining(7200), 3600);
}

#[test]
fn remaining_with_extension() {
    let us = UserState {
        date: chrono::Local::now().date_naive(),
        used_seconds: 7200,
        extension_seconds: 1800,
        warnings_sent: vec![],
    };
    // limit=7200, ext=1800, used=7200 → remaining = 9000-7200 = 1800
    assert_eq!(us.remaining(7200), 1800);
}

#[test]
fn remaining_exactly_at_limit() {
    let us = UserState {
        date: chrono::Local::now().date_naive(),
        used_seconds: 7200,
        extension_seconds: 0,
        warnings_sent: vec![],
    };
    assert_eq!(us.remaining(7200), 0);
}

#[test]
fn remaining_over_limit_is_negative() {
    let us = UserState {
        date: chrono::Local::now().date_naive(),
        used_seconds: 7260,
        extension_seconds: 0,
        warnings_sent: vec![],
    };
    assert_eq!(us.remaining(7200), -60);
}

#[test]
fn remaining_over_limit_with_extension_still_negative() {
    let us = UserState {
        date: chrono::Local::now().date_naive(),
        used_seconds: 10000,
        extension_seconds: 1800,
        warnings_sent: vec![],
    };
    // limit=7200, ext=1800 → total=9000, used=10000 → remaining=-1000
    assert_eq!(us.remaining(7200), -1000);
}

#[test]
fn remaining_zero_limit() {
    let us = UserState {
        date: chrono::Local::now().date_naive(),
        used_seconds: 0,
        extension_seconds: 0,
        warnings_sent: vec![],
    };
    assert_eq!(us.remaining(0), 0);
}

// --- UserState::ensure_today ---

#[test]
fn ensure_today_resets_on_old_date() {
    let mut us = UserState {
        date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        used_seconds: 5000,
        extension_seconds: 600,
        warnings_sent: vec![900, 300],
    };
    us.ensure_today();

    let today = chrono::Local::now().date_naive();
    assert_eq!(us.date, today);
    assert_eq!(us.used_seconds, 0);
    assert_eq!(us.extension_seconds, 0);
    assert!(us.warnings_sent.is_empty());
}

#[test]
fn ensure_today_keeps_data_if_same_day() {
    let today = chrono::Local::now().date_naive();
    let mut us = UserState {
        date: today,
        used_seconds: 1234,
        extension_seconds: 600,
        warnings_sent: vec![900],
    };
    us.ensure_today();

    assert_eq!(us.date, today);
    assert_eq!(us.used_seconds, 1234);
    assert_eq!(us.extension_seconds, 600);
    assert_eq!(us.warnings_sent, vec![900]);
}

// --- State serialization ---

#[test]
fn state_json_roundtrip() {
    let mut state = State::default();
    state.users.insert(
        "alice".into(),
        UserState {
            date: NaiveDate::from_ymd_opt(2026, 3, 7).unwrap(),
            used_seconds: 3600,
            extension_seconds: 900,
            warnings_sent: vec![900, 300],
        },
    );

    let json = serde_json::to_string_pretty(&state).unwrap();
    let restored: State = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.users.len(), 1);
    let alice = &restored.users["alice"];
    assert_eq!(alice.date, NaiveDate::from_ymd_opt(2026, 3, 7).unwrap());
    assert_eq!(alice.used_seconds, 3600);
    assert_eq!(alice.extension_seconds, 900);
    assert_eq!(alice.warnings_sent, vec![900, 300]);
}

#[test]
fn state_empty_roundtrip() {
    let state = State::default();
    let json = serde_json::to_string(&state).unwrap();
    let restored: State = serde_json::from_str(&json).unwrap();
    assert!(restored.users.is_empty());
}

#[test]
fn state_parse_example_json() {
    let json = r#"{
        "alice": {
            "date": "2026-03-07",
            "used_seconds": 3600,
            "extension_seconds": 900,
            "warnings_sent": [900, 300]
        },
        "bob": {
            "date": "2026-03-07",
            "used_seconds": 0,
            "extension_seconds": 0,
            "warnings_sent": []
        }
    }"#;
    let state: State = serde_json::from_str(json).unwrap();
    assert_eq!(state.users.len(), 2);
    assert_eq!(state.users["alice"].used_seconds, 3600);
    assert_eq!(state.users["bob"].used_seconds, 0);
}

// --- State::get_mut ---

#[test]
fn get_mut_creates_new_entry() {
    let mut state = State::default();
    let us = state.get_mut("newuser");
    assert_eq!(us.used_seconds, 0);
    assert_eq!(us.date, chrono::Local::now().date_naive());
    assert!(state.users.contains_key("newuser"));
}

#[test]
fn get_mut_returns_existing() {
    let mut state = State::default();
    state.users.insert(
        "alice".into(),
        UserState {
            date: chrono::Local::now().date_naive(),
            used_seconds: 500,
            extension_seconds: 100,
            warnings_sent: vec![],
        },
    );

    let us = state.get_mut("alice");
    assert_eq!(us.used_seconds, 500);
    assert_eq!(us.extension_seconds, 100);
}

#[test]
fn get_mut_second_call_preserves_changes() {
    let mut state = State::default();
    state.get_mut("alice").used_seconds = 1000;
    assert_eq!(state.get_mut("alice").used_seconds, 1000);
}

// --- Warning threshold logic ---

#[test]
fn warnings_sent_tracking() {
    let mut us = UserState::new_today();
    us.used_seconds = 6000; // limit=7200 → remaining=1200 → no 900s warning yet

    // Simulate daemon logic: check if 900 threshold should fire
    let limit_s = 7200u64;
    let remaining = us.remaining(limit_s);
    assert_eq!(remaining, 1200);

    // Not yet at 900
    assert!(remaining > 900);

    // Advance time
    us.used_seconds = 6350; // remaining=850 → crossed 900 threshold
    let remaining = us.remaining(limit_s);
    assert!(remaining <= 900);
    assert!(!us.warnings_sent.contains(&900));
    us.warnings_sent.push(900);

    // Now 900 is recorded
    assert!(us.warnings_sent.contains(&900));

    // Advance more
    us.used_seconds = 6950; // remaining=250 → crossed 300 threshold
    let remaining = us.remaining(limit_s);
    assert!(remaining <= 300);
    assert!(!us.warnings_sent.contains(&300));
    us.warnings_sent.push(300);

    assert_eq!(us.warnings_sent, vec![900, 300]);
}

// --- Edge cases ---

#[test]
fn state_multiple_users_independent() {
    let mut state = State::default();
    state.get_mut("alice").used_seconds = 3600;
    state.get_mut("bob").used_seconds = 1800;

    assert_eq!(state.users["alice"].used_seconds, 3600);
    assert_eq!(state.users["bob"].used_seconds, 1800);
}

#[test]
fn remaining_large_extension() {
    let us = UserState {
        date: chrono::Local::now().date_naive(),
        used_seconds: 100,
        extension_seconds: 86400, // 24 hours bonus
        warnings_sent: vec![],
    };
    assert_eq!(us.remaining(3600), 3600 + 86400 - 100);
}
