//! SLMP end-code helpers.
//!
//! Localized SLMP end-code message catalogs are intentionally not embedded in
//! this public communication crate.  Applications that need localized text can
//! resolve the stable key returned by [`end_code_key`] against their own catalog.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static END_CODE_NAME_CACHE: OnceLock<Mutex<HashMap<u16, &'static str>>> = OnceLock::new();

/// Return the stable catalog/resource key for an SLMP end code.
///
/// This function does not need a catalog; it derives the key directly from the
/// numeric end code, for example `0xC810` becomes `slmp_end_code_c810`.
pub fn end_code_key(end_code: u16) -> String {
    format!("slmp_end_code_{end_code:04x}")
}

/// Return a compact code-derived diagnostic label for an SLMP end code.
///
/// The returned value is suitable as a resource key. Localized message text is
/// intentionally outside this communication crate.
pub fn end_code_name(end_code: u16) -> &'static str {
    let cache = END_CODE_NAME_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(&value) = guard.get(&end_code) {
            return value;
        }
    }

    let generated = Box::leak(end_code_key(end_code).into_boxed_str());
    if let Ok(mut guard) = cache.lock() {
        guard.entry(end_code).or_insert(generated)
    } else {
        generated
    }
}

/// Return whether the SLMP end code is related to remote password protection.
pub fn is_remote_password_end_code(end_code: u16) -> bool {
    matches!(
        end_code,
        0xC200
            | 0xC201
            | 0xC202
            | 0xC203
            | 0xC204
            | 0xC205
            | 0xC810
            | 0xC811
            | 0xC812
            | 0xC813
            | 0xC814
            | 0xC815
            | 0xC816
    )
}
