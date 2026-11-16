use nd_litert_lm_sys::litert_lm_clear_last_error;
use nd_litert_lm_sys::litert_lm_get_last_error_message;
use std::ffi::CStr;
use std::ffi::CString;

#[derive(Debug)]
pub struct Error {
    message: CString,
}

impl Error {
    /// Get the last error.
    pub(crate) fn last_error() -> Self {
        let message_ptr = unsafe { litert_lm_get_last_error_message() };
        let message = if message_ptr.is_null() {
            CString::new("No error").unwrap()
        } else {
            let message_cstr = unsafe { CStr::from_ptr(message_ptr) };
            CString::from(message_cstr)
        };

        Self { message }
    }

    /// Get the last error, and clear it.
    pub(crate) fn last_error_clear() -> Self {
        let error = Self::last_error();
        unsafe { litert_lm_clear_last_error() }
        error
    }

    /// Create an error from a string.
    ///
    /// # Panics
    /// Panics if message has an interior nul.
    pub(crate) fn custom(message: &str) -> Self {
        let message = CString::new(message).expect("message has an interior NUL");

        Self { message }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\"")?;
        for chunk in self.message.as_bytes().utf8_chunks() {
            for ch in chunk.valid().chars() {
                write!(f, "{}", ch.escape_debug())?;
            }
            for byte in chunk.invalid() {
                write!(f, "\\x{:02X}", byte)?;
            }
        }
        write!(f, "\"")?;

        Ok(())
    }
}

impl std::error::Error for Error {}
