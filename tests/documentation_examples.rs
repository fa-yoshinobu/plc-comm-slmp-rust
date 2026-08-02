const GETTING_STARTED: &str = include_str!("../docs/GETTING_STARTED.md");
const USAGE_GUIDE: &str = include_str!("../docs/USAGE_GUIDE.md");

use plc_comm_slmp::{
    SlmpAddress, SlmpClient, SlmpPlcProfile, SlmpValue, parse_qualified_device, read_named,
    read_typed, write_bit_in_word, write_typed,
};

#[allow(dead_code)]
async fn compile_cleanup_control_flow(
    client: &SlmpClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let password_address = SlmpAddress::parse("D100", client.plc_profile().await)?;
    client.remote_password_unlock("secret").await?;
    let password_read_result = read_typed(client, password_address, "U").await;
    let lock_result = client.remote_password_lock("secret").await;
    lock_result?;
    let _ = password_read_result?;

    let address = SlmpAddress::parse("D600", SlmpPlcProfile::IqR)?;
    let original = read_typed(client, address, "U").await?;
    write_typed(client, address, "U", &SlmpValue::U16(42)).await?;
    let readback_result = read_typed(client, address, "U").await;
    let restore_result = write_typed(client, address, "U", &original).await;
    restore_result?;
    let _ = readback_result?;

    let module = parse_qualified_device(r"U3\G100", SlmpPlcProfile::IqR)?;
    let original = client.read_words_extended(module, 4).await?;
    client.write_words_extended(module, &[1, 2, 3, 4]).await?;
    let readback_result = client.read_words_extended(module, 4).await;
    let restore_result = client.write_words_extended(module, &original).await;
    restore_result?;
    let _ = readback_result?;

    let word = SlmpAddress::parse("D50", SlmpPlcProfile::IqR)?;
    let original = read_typed(client, word, "U").await?;
    write_bit_in_word(client, word, 3, true).await?;
    let addresses = vec!["D50.3".to_owned()];
    let snapshot_result = read_named(client, &addresses).await;
    let restore_result = write_typed(client, word, "U", &original).await;
    restore_result?;
    let _ = snapshot_result?;
    Ok(())
}

fn section<'a>(document: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = document.find(start).expect("section start");
    let tail = &document[start_index..];
    let end_index = tail.find(end).expect("section end");
    &tail[..end_index]
}

fn assert_ordered(text: &str, needles: &[&str]) {
    let mut offset = 0;
    for needle in needles {
        let relative = text[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("missing {needle:?}"));
        offset += relative + needle.len();
    }
}

#[test]
fn controlled_writes_restore_before_propagating_readback_errors() {
    assert_ordered(
        GETTING_STARTED,
        &[
            "let readback_result = read_typed",
            "let restore_result = write_typed",
            "restore_result?;",
            "let value = readback_result?;",
        ],
    );

    let password = section(USAGE_GUIDE, "## Remote password", "## Remote CPU control");
    assert_ordered(
        password,
        &[
            "let password_address =",
            "client.remote_password_unlock",
            "let read_result = read_typed",
            "let lock_result = client.remote_password_lock",
            "lock_result?;",
            "let value = read_result?;",
        ],
    );

    let single = section(
        USAGE_GUIDE,
        "## Write a single value",
        "## Named typed collection",
    );
    assert_ordered(
        single,
        &[
            "let readback_result = read_typed",
            "let restore_result = write_typed",
            "restore_result?;",
            "let value = readback_result?;",
        ],
    );

    let bit = section(USAGE_GUIDE, "## Bit in word", "## Polling");
    assert_ordered(
        bit,
        &[
            "let snapshot_result = read_named",
            "let restore_result = write_typed",
            "restore_result?;",
            "let snapshot = snapshot_result?;",
        ],
    );
}

#[test]
fn extended_write_and_clear_error_are_explicitly_controlled() {
    let extended = section(
        USAGE_GUIDE,
        "## Extended device access",
        "## Monitor, self-test",
    );
    assert_ordered(
        extended,
        &[
            "let module_readback_result",
            "let module_restore_result",
            "module_restore_result?;",
            "let module_readback = module_readback_result?;",
        ],
    );
    assert!(extended.contains("controlled-test example"));
    assert!(USAGE_GUIDE.contains("Clear Error is a separate state-changing maintenance action."));
    assert!(USAGE_GUIDE.contains("outcome-unknown error"));
}
