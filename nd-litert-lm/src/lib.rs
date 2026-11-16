mod conversation;
mod engine;
mod error;
mod version;

pub use self::conversation::Conversation;
pub use self::conversation::ConversationConfig;
pub use self::conversation::ConversationMessage;
pub use self::conversation::ConversationMessageContent;
pub use self::conversation::ConversationMessageContentType;
pub use self::conversation::ConversationMessageRole;
pub use self::conversation::ConversationOptionalArgs;
pub use self::conversation::ThinkingConfig;
pub use self::engine::Engine;
pub use self::engine::EngineSettings;
pub use self::engine::EngineSettingsBackend;
pub use self::error::Error;
pub use self::version::get_api_version;
use nd_litert_lm_sys::LiteRtLmLogSeverity;
use nd_litert_lm_sys::LiteRtLmStatusCode;
use nd_litert_lm_sys::litert_lm_set_min_log_level;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LogSeverity {
    Verbose,
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
    Silent,
}

impl LogSeverity {
    /// Get the -sys rep of this enum.
    fn as_raw(self) -> LiteRtLmLogSeverity {
        match self {
            Self::Verbose => LiteRtLmLogSeverity::kLiteRtLmLogSeverityVerbose,
            Self::Debug => LiteRtLmLogSeverity::kLiteRtLmLogSeverityDebug,
            Self::Info => LiteRtLmLogSeverity::kLiteRtLmLogSeverityInfo,
            Self::Warning => LiteRtLmLogSeverity::kLiteRtLmLogSeverityWarning,
            Self::Error => LiteRtLmLogSeverity::kLiteRtLmLogSeverityError,
            Self::Fatal => LiteRtLmLogSeverity::kLiteRtLmLogSeverityFatal,
            Self::Silent => LiteRtLmLogSeverity::kLiteRtLmLogSeveritySilent,
        }
    }
}

/// Set the min log level.
pub fn set_min_log_level(level: LogSeverity) -> Result<(), Error> {
    let code = unsafe { litert_lm_set_min_log_level(level.as_raw()) };
    if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
        return Err(Error::last_error_clear());
    }

    Ok(())
}
