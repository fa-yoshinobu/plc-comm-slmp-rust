//! SLMP end-code helpers.
//!
//! Localized SLMP end-code message catalogs are intentionally not embedded in
//! this public communication crate.  Applications that need localized text can
//! resolve the stable key returned by [`end_code_key`] against their own catalog.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Language selector retained for source compatibility with older message
/// lookup helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlmpEndCodeLanguage {
    /// English.
    English,
    /// Japanese.
    Japanese,
}

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
/// The returned value is suitable as a resource key.  Localized message text is
/// not stored in this crate; use [`end_code_message`] only as an optional hook
/// and resolve the key in an application-owned catalog when text is needed.
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

/// Return the English error detail/cause message for an SLMP end code.
///
/// Message text is not embedded in this crate.  This function returns `None`
/// unless an application layers its own catalog outside this crate.
pub fn end_code_message_en(_end_code: u16) -> Option<&'static str> {
    None
}

/// Return the Japanese error detail/cause message for an SLMP end code.
///
/// Message text is not embedded in this crate.  This function returns `None`
/// unless an application layers its own catalog outside this crate.
pub fn end_code_message_ja(_end_code: u16) -> Option<&'static str> {
    None
}

/// Return the error detail/cause message for an SLMP end code.
///
/// Message text is not embedded in this crate.  The language parameter is kept
/// for source compatibility; callers that need text should resolve
/// [`end_code_key`] in an application-owned catalog.
pub fn end_code_message(end_code: u16, _language: SlmpEndCodeLanguage) -> Option<&'static str> {
    end_code_message_en(end_code)
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
