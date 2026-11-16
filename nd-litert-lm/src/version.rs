use crate::Error;
use nd_litert_lm_sys::LiteRtLmStatusCode;
use nd_litert_lm_sys::litert_lm_get_api_version;

#[derive(Debug, Copy, Clone)]
pub struct ApiVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Get the api version.
///
/// # Returns
/// Returns the major, minor, and patch versions as a tuple.
/// This should never return an error.
///
/// # Panics
/// Panics if the version is negative. This shouldn't happen.
pub fn get_api_version() -> Result<ApiVersion, Error> {
    let mut major = 0;
    let mut minor = 0;
    let mut patch = 0;

    let code = unsafe { litert_lm_get_api_version(&mut major, &mut minor, &mut patch) };
    if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
        return Err(Error::last_error_clear());
    }

    Ok(ApiVersion {
        major: u32::try_from(major).expect("major is negative"),
        minor: u32::try_from(minor).expect("minor is negative"),
        patch: u32::try_from(patch).expect("patch is negative"),
    })
}
