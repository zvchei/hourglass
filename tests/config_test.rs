use hourglass::config::{Config, UserConfig};

#[test]
fn parse_valid_config() {
    let toml = r#"
[users.alice]
daily_limit_minutes = 120

[users.bob]
daily_limit_minutes = 60
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.users.len(), 2);
    assert_eq!(config.users["alice"].daily_limit_minutes, 120);
    assert_eq!(config.users["bob"].daily_limit_minutes, 60);
}

#[test]
fn parse_empty_config() {
    let config: Config = toml::from_str("").unwrap();
    assert!(config.users.is_empty());
}

#[test]
fn parse_config_no_users_section() {
    let toml = r#"
[something_else]
key = "value"
"#;
    let config: Config = toml::from_str(toml).unwrap_or_default();
    assert!(config.users.is_empty());
}

#[test]
fn config_roundtrip() {
    let mut config = Config::default();
    config.users.insert(
        "alice".into(),
        UserConfig {
            daily_limit_minutes: 90,
        },
    );
    config.users.insert(
        "bob".into(),
        UserConfig {
            daily_limit_minutes: 45,
        },
    );

    let serialized = toml::to_string_pretty(&config).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();

    assert_eq!(deserialized.users.len(), 2);
    assert_eq!(deserialized.users["alice"].daily_limit_minutes, 90);
    assert_eq!(deserialized.users["bob"].daily_limit_minutes, 45);
}

#[test]
fn config_single_user() {
    let toml = r#"
[users.charlie]
daily_limit_minutes = 30
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.users.len(), 1);
    assert_eq!(config.users["charlie"].daily_limit_minutes, 30);
}

#[test]
fn config_zero_limit() {
    let toml = r#"
[users.locked]
daily_limit_minutes = 0
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.users["locked"].daily_limit_minutes, 0);
}

#[test]
fn config_large_limit() {
    let toml = r#"
[users.generous]
daily_limit_minutes = 1440
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.users["generous"].daily_limit_minutes, 1440);
}

#[test]
fn config_default_is_empty() {
    let config = Config::default();
    assert!(config.users.is_empty());
}
