use plc_comm_slmp::{
    SlmpEndCodeLanguage, SlmpError, end_code_message, end_code_message_en, end_code_message_ja,
    end_code_name, is_remote_password_end_code,
};

#[test]
fn end_code_names_and_messages() {
    assert_eq!(end_code_name(0x1080), "slmp_end_code_1080");
    assert_eq!(
        end_code_message_en(0x1080),
        Some("The number of writes to the flash ROM has exceeded 100000.")
    );
    assert_eq!(
        end_code_message_ja(0x1080),
        Some("フラッシュROMへの書込み回数が10万回を超えた。")
    );

    assert_eq!(end_code_name(0xC201), "slmp_end_code_c201");
    assert_eq!(
        end_code_message(0xC201, SlmpEndCodeLanguage::English),
        Some("The remote password status of the port used for communications is in the lock status.")
    );

    assert_eq!(end_code_name(0xC810), "slmp_end_code_c810");
    assert_eq!(
        end_code_message_en(0xC810),
        Some("Remote password authentication has failed when required. Set a correct password and retry.")
    );
    assert_eq!(
        end_code_message_en(0xC811),
        Some("Remote password authentication has failed when required. Set a correct password and retry after 1 minute.")
    );
    assert_eq!(
        end_code_message_en(0xC814),
        Some("Remote password authentication has failed when required. Set a correct password and retry after 60 minutes.")
    );
    assert_eq!(
        end_code_message_ja(0xC810),
        Some("リモートパスワード認証が必要なアクセス時に，リモートパスワードのパスワード認証に失敗した。正しいパスワードを設定して再度実行してください。")
    );

    assert_eq!(end_code_name(0xCFBF), "slmp_end_code_cfbf");
    assert_eq!(
        end_code_message_en(0xCFBF),
        Some("The simple CPU communication cannot be executed.")
    );

    assert_eq!(end_code_name(0xE504), "slmp_end_code_e504");
    assert_eq!(
        end_code_message_ja(0xE504),
        Some("自局がバトンパスを行っていない状態で，トランジェント伝送が実行された。")
    );
}

#[test]
fn unknown_and_remote_password_codes() {
    assert_eq!(end_code_name(0xDEAD), "unknown_plc_end_code");
    assert_eq!(end_code_message_en(0xDEAD), None);
    assert!(is_remote_password_end_code(0xC201));
    assert!(is_remote_password_end_code(0xC810));
    assert!(!is_remote_password_end_code(0x1080));
}

#[test]
fn slmp_error_end_code_helpers() {
    let error = SlmpError::with_context("SLMP error", Some(0xC201), None, None);
    assert_eq!(error.end_code_name(), Some("slmp_end_code_c201"));
    assert_eq!(
        error.end_code_message(),
        Some("The remote password status of the port used for communications is in the lock status.")
    );
    assert!(error.is_remote_password_error());

    let without_code = SlmpError::new("no end code");
    assert_eq!(without_code.end_code_name(), None);
    assert_eq!(without_code.end_code_message(), None);
    assert!(!without_code.is_remote_password_error());
}
