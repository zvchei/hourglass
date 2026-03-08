use hourglass::session::{parse_display_property, parse_session_line, parse_session_properties};

// --- parse_session_line ---

#[test]
fn parse_valid_session_line() {
    let line = "    42 1000 alice seat0 tty2";
    let (id, user) = parse_session_line(line).unwrap();
    assert_eq!(id, "42");
    assert_eq!(user, "alice");
}

#[test]
fn parse_session_line_minimal_columns() {
    let line = "1 1000 bob";
    let (id, user) = parse_session_line(line).unwrap();
    assert_eq!(id, "1");
    assert_eq!(user, "bob");
}

#[test]
fn parse_session_line_extra_columns() {
    let line = "  100  1001  charlie  seat0  tty7  extra";
    let (id, user) = parse_session_line(line).unwrap();
    assert_eq!(id, "100");
    assert_eq!(user, "charlie");
}

#[test]
fn parse_session_line_too_few_columns() {
    assert!(parse_session_line("42 1000").is_none());
}

#[test]
fn parse_session_line_empty() {
    assert!(parse_session_line("").is_none());
}

#[test]
fn parse_session_line_whitespace_only() {
    assert!(parse_session_line("   ").is_none());
}

// --- parse_session_properties ---

#[test]
fn parse_x11_user_session() {
    let output = "Type=x11\nClass=user\nState=active\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_wayland_user_session() {
    let output = "Type=wayland\nClass=user\nState=active\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_mir_user_session() {
    let output = "Type=mir\nClass=user\nState=active\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_tty_session() {
    let output = "Type=tty\nClass=user\nState=active\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(!gui);
    assert!(user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_greeter_session() {
    let output = "Type=x11\nClass=greeter\nState=active\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(!user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_unspecified_type() {
    let output = "Type=unspecified\nClass=user\nState=active\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(!gui);
    assert!(user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_empty_properties() {
    let (gui, user, active, idle) = parse_session_properties("");
    assert!(!gui);
    assert!(!user);
    assert!(!active);
    assert!(!idle);
}

#[test]
fn parse_properties_with_extra_whitespace() {
    let output = "  Type=x11  \n  Class=user  \n  State=active  \n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_properties_mixed_order() {
    let output = "Class=user\nType=wayland\nState=active\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_properties_extra_lines() {
    let output = "State=active\nType=x11\nClass=user\nSeat=seat0\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_idle_session() {
    let output = "Type=wayland\nClass=user\nState=active\nIdleHint=yes\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(user);
    assert!(active);
    assert!(idle);
}

#[test]
fn parse_not_idle_session() {
    let output = "Type=x11\nClass=user\nState=active\nIdleHint=no\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(user);
    assert!(active);
    assert!(!idle);
}

#[test]
fn parse_online_not_active_session() {
    let output = "Type=wayland\nClass=user\nState=online\nIdleHint=no\n";
    let (gui, user, active, idle) = parse_session_properties(output);
    assert!(gui);
    assert!(user);
    assert!(!active);
    assert!(!idle);
}

// --- parse_display_property ---

#[test]
fn parse_display_present() {
    let output = "Display=:0\n";
    assert_eq!(parse_display_property(output), Some(":0".into()));
}

#[test]
fn parse_display_with_number() {
    let output = "Display=:1\n";
    assert_eq!(parse_display_property(output), Some(":1".into()));
}

#[test]
fn parse_display_empty() {
    let output = "Display=\n";
    assert_eq!(parse_display_property(output), None);
}

#[test]
fn parse_display_missing() {
    let output = "Type=x11\nClass=user\n";
    assert_eq!(parse_display_property(output), None);
}

#[test]
fn parse_display_among_other_props() {
    let output = "Type=x11\nDisplay=:0\nClass=user\n";
    assert_eq!(parse_display_property(output), Some(":0".into()));
}

#[test]
fn parse_display_whitespace() {
    let output = "Display= :0 \n";
    assert_eq!(parse_display_property(output), Some(":0".into()));
}

// --- Multi-line loginctl output simulation ---

#[test]
fn parse_multiple_session_lines() {
    let output = "\
        42 1000 alice seat0 tty2\n\
        43 1001 bob   seat0 tty7\n\
        44 1002 carol seat0 -\n";

    let results: Vec<_> = output
        .lines()
        .filter_map(parse_session_line)
        .collect();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], ("42", "alice"));
    assert_eq!(results[1], ("43", "bob"));
    assert_eq!(results[2], ("44", "carol"));
}

#[test]
fn parse_session_lines_with_blanks() {
    let output = "\n42 1000 alice seat0\n\n43 1001 bob seat0\n\n";
    let results: Vec<_> = output
        .lines()
        .filter_map(parse_session_line)
        .collect();
    assert_eq!(results.len(), 2);
}
