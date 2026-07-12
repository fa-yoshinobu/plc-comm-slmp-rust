use plc_comm_slmp::{SlmpError, end_code_key, end_code_name, is_remote_password_end_code};

#[test]
fn end_code_keys_and_names_are_code_derived() {
    assert_eq!(end_code_key(0x1080), "slmp_end_code_1080");
    assert_eq!(end_code_name(0x1080), "slmp_end_code_1080");
    assert_eq!(end_code_key(0xC201), "slmp_end_code_c201");
    assert_eq!(end_code_name(0xC201), "slmp_end_code_c201");
    assert_eq!(end_code_key(0xDEAD), "slmp_end_code_dead");
    assert_eq!(end_code_name(0xDEAD), "slmp_end_code_dead");
}

#[test]
fn remote_password_codes_are_classified() {
    assert_eq!(end_code_name(0xC810), "slmp_end_code_c810");
    assert!(is_remote_password_end_code(0xC201));
    assert!(is_remote_password_end_code(0xC810));
    assert!(!is_remote_password_end_code(0x1080));
}

#[test]
fn slmp_error_end_code_helpers() {
    let error = SlmpError::with_context("SLMP error", Some(0xC201), None, None);
    assert_eq!(error.end_code_name(), Some("slmp_end_code_c201"));
    assert!(error.is_remote_password_error());

    let without_code = SlmpError::new("no end code");
    assert_eq!(without_code.end_code_name(), None);
    assert!(!without_code.is_remote_password_error());
}
