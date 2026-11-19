mod message;
mod tool;

pub use self::message::ConversationMessageContent;
pub use self::message::ConversationMessageRole;
pub use self::message::ToolCall as ConversationToolCall;
pub use self::tool::FunctionParameters;
use crate::Engine;
use crate::Error;
use nd_litert_lm_sys::LiteRtLmConversation;
use nd_litert_lm_sys::LiteRtLmConversationConfig;
use nd_litert_lm_sys::LiteRtLmConversationOptionalArgs;
use nd_litert_lm_sys::LiteRtLmJsonResponse;
use nd_litert_lm_sys::LiteRtLmStatusCode;
use nd_litert_lm_sys::LiteRtLmThinkingConfig;
use nd_litert_lm_sys::litert_lm_conversation_config_create;
use nd_litert_lm_sys::litert_lm_conversation_config_delete;
use nd_litert_lm_sys::litert_lm_conversation_config_set_prompt_template;
use nd_litert_lm_sys::litert_lm_conversation_config_set_system_message;
use nd_litert_lm_sys::litert_lm_conversation_config_set_thinking_config;
use nd_litert_lm_sys::litert_lm_conversation_config_set_tools;
use nd_litert_lm_sys::litert_lm_conversation_create;
use nd_litert_lm_sys::litert_lm_conversation_delete;
use nd_litert_lm_sys::litert_lm_conversation_optional_args_create;
use nd_litert_lm_sys::litert_lm_conversation_optional_args_delete;
use nd_litert_lm_sys::litert_lm_conversation_optional_args_set_thinking_config;
use nd_litert_lm_sys::litert_lm_conversation_send_message;
use nd_litert_lm_sys::litert_lm_json_response_delete;
use nd_litert_lm_sys::litert_lm_json_response_get_string;
use nd_litert_lm_sys::litert_lm_thinking_config_create;
use nd_litert_lm_sys::litert_lm_thinking_config_delete;
use nd_litert_lm_sys::litert_lm_thinking_config_set_enable_thinking;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct ThinkingConfig {
    ptr: *mut LiteRtLmThinkingConfig,
}

impl ThinkingConfig {
    /// Create a new thinking config.
    pub fn new() -> Result<Self, Error> {
        let mut config: MaybeUninit<*mut LiteRtLmThinkingConfig> = MaybeUninit::zeroed();
        let code = unsafe { litert_lm_thinking_config_create(config.as_mut_ptr()) };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        let ptr = unsafe { config.assume_init() };
        Ok(Self { ptr })
    }

    /// Set whether thinking is enabled.
    pub fn enable_thinking(&self, enable_thinking: bool) -> Result<(), Error> {
        let code =
            unsafe { litert_lm_thinking_config_set_enable_thinking(self.ptr, enable_thinking) };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }

        Ok(())
    }
}

impl Drop for ThinkingConfig {
    fn drop(&mut self) {
        unsafe { litert_lm_thinking_config_delete(self.ptr) }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: FunctionParameters,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum Tool {
    #[serde(rename = "function")]
    Function { function: ToolFunction },
}

#[derive(Debug)]
pub struct ConversationConfig {
    ptr: *mut LiteRtLmConversationConfig,
}

impl ConversationConfig {
    /// Make a new conversation config.
    pub fn new() -> Result<Self, Error> {
        let mut config: MaybeUninit<*mut LiteRtLmConversationConfig> = MaybeUninit::zeroed();

        let code = unsafe { litert_lm_conversation_config_create(config.as_mut_ptr()) };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        let ptr = unsafe { config.assume_init() };

        Ok(Self { ptr })
    }

    /// Set the prompt template.
    pub fn set_prompt_template(&self, prompt_template: &str) -> Result<(), Error> {
        let prompt_template = CString::new(prompt_template).map_err(|error| {
            Error::custom_with_error("prompt_template contains an interior nul", error)
        })?;
        let code = unsafe {
            litert_lm_conversation_config_set_prompt_template(self.ptr, prompt_template.as_ptr())
        };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        Ok(())
    }

    /// Set the system message.
    pub fn set_system_message(&self, content: &str) -> Result<(), Error> {
        #[derive(Debug, serde::Serialize)]
        struct SystemMessage<'a> {
            #[serde(rename = "type")]
            type_: &'static str,
            text: &'a str,
        }

        let system_message = serde_json::to_string(&SystemMessage {
            type_: "text",
            text: content,
        })
        .map_err(|error| Error::custom_with_error("Failed to serialize system_message", error))?;
        let system_message = CString::new(system_message).map_err(|error| {
            Error::custom_with_error("system_message had an interior nul", error)
        })?;
        let code = unsafe {
            litert_lm_conversation_config_set_system_message(self.ptr, system_message.as_ptr())
        };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }

        Ok(())
    }

    /// Set the thinking config.
    pub fn set_thinking_config(&self, config: &ThinkingConfig) -> Result<(), Error> {
        let code =
            unsafe { litert_lm_conversation_config_set_thinking_config(self.ptr, config.ptr) };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        Ok(())
    }

    /// Set the tools.
    pub fn set_tools(&self, tools: &[Tool]) -> Result<(), Error> {
        let tools = serde_json::to_string(tools)
            .map_err(|error| Error::custom_with_error("Failed to serialize tools", error))?;
        let tools = CString::new(tools).map_err(|error| {
            Error::custom_with_error("Tools json contained an interior nul", error)
        })?;
        let code = unsafe { litert_lm_conversation_config_set_tools(self.ptr, tools.as_ptr()) };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        Ok(())
    }
}

impl Drop for ConversationConfig {
    fn drop(&mut self) {
        unsafe { litert_lm_conversation_config_delete(self.ptr) }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConversationMessage {
    pub role: ConversationMessageRole,

    /// This can be empty if the model attempts a tool call.
    #[serde(default)]
    pub content: Vec<ConversationMessageContent>,

    #[serde(default)]
    pub tool_calls: Vec<ConversationToolCall>,

    // This should only be specified for response messages.
    #[serde(default)]
    pub channels: HashMap<String, String>,
}

pub struct ConversationOptionalArgs {
    ptr: *mut LiteRtLmConversationOptionalArgs,
}

impl ConversationOptionalArgs {
    /// Create new conversation optional args.
    pub fn new() -> Result<Self, Error> {
        let mut args: MaybeUninit<*mut LiteRtLmConversationOptionalArgs> = MaybeUninit::zeroed();
        let code = unsafe { litert_lm_conversation_optional_args_create(args.as_mut_ptr()) };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        let ptr = unsafe { args.assume_init() };
        Ok(Self { ptr })
    }

    /// Set the thinking config.
    pub fn set_thinking_config(&self, thinking_config: &ThinkingConfig) -> Result<(), Error> {
        let code = unsafe {
            litert_lm_conversation_optional_args_set_thinking_config(self.ptr, thinking_config.ptr)
        };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        Ok(())
    }
}

impl Drop for ConversationOptionalArgs {
    fn drop(&mut self) {
        unsafe { litert_lm_conversation_optional_args_delete(self.ptr) }
    }
}

#[derive(Debug)]
pub struct Conversation<'a> {
    ptr: *mut LiteRtLmConversation,
    _engine: PhantomData<&'a Engine>,
}

impl<'a> Conversation<'a> {
    /// Make a new conversation.
    pub fn new(engine: &'a Engine, config: &ConversationConfig) -> Result<Self, Error> {
        let mut conversation: MaybeUninit<*mut LiteRtLmConversation> = MaybeUninit::zeroed();
        let code = unsafe {
            litert_lm_conversation_create(engine.as_ptr(), config.ptr, conversation.as_mut_ptr())
        };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        let ptr = unsafe { conversation.assume_init() };

        Ok(Self {
            ptr,
            _engine: PhantomData,
        })
    }

    /// Send a message.
    pub fn send_message(
        &self,
        message: &ConversationMessage,
        optional_args: Option<&ConversationOptionalArgs>,
    ) -> Result<JsonResponse, Error> {
        // As of 2026-11-16, the C library actually throws an exception if we don't validate this here.
        if message.content.iter().any(|content| {
            matches!(
                content,
                ConversationMessageContent::Image {
                    path: None,
                    blob: None
                }
            )
        }) || message.content.iter().any(|content| {
            matches!(
                content,
                ConversationMessageContent::Image {
                    path: Some(_),
                    blob: Some(_)
                }
            )
        }) {
            return Err(Error::custom(
                "ConversationMessageContent::Image must either specify blob or path.",
            ));
        }
        if message.content.iter().any(|content| {
            matches!(
                content,
                ConversationMessageContent::Audio {
                    path: None,
                    blob: None
                }
            )
        }) || message.content.iter().any(|content| {
            matches!(
                content,
                ConversationMessageContent::Audio {
                    path: Some(_),
                    blob: Some(_)
                }
            )
        }) {
            return Err(Error::custom(
                "ConversationMessageContent::Audio must either specify blob or path.",
            ));
        }

        let message_json = serde_json::to_string(message).map_err(|error| {
            Error::custom_with_error("Failed to serialize message to JSON", error)
        })?;
        let message_json = CString::new(message_json)
            .map_err(|error| Error::custom_with_error("message_json has an interior NUL", error))?;
        let extra_context = std::ptr::null();
        let mut response: MaybeUninit<*mut LiteRtLmJsonResponse> = MaybeUninit::zeroed();
        let code = unsafe {
            litert_lm_conversation_send_message(
                self.ptr,
                message_json.as_ptr(),
                extra_context,
                optional_args
                    .map(|args| args.ptr)
                    .unwrap_or(std::ptr::null_mut()),
                response.as_mut_ptr(),
            )
        };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        let ptr = unsafe { response.assume_init() };

        Ok(JsonResponse { ptr })
    }
}

impl Drop for Conversation<'_> {
    fn drop(&mut self) {
        unsafe { litert_lm_conversation_delete(self.ptr) }
    }
}

#[derive(Debug)]
pub struct JsonResponse {
    ptr: *mut LiteRtLmJsonResponse,
}

impl JsonResponse {
    /// Get the response as a JSON CStr.
    pub fn get_string(&self) -> Result<&CStr, Error> {
        let mut string = std::ptr::null();
        let code = unsafe { litert_lm_json_response_get_string(self.ptr, &mut string) };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        let cstr = unsafe { CStr::from_ptr(string) };

        Ok(cstr)
    }

    /// Get the response string and parse it.
    pub fn get_string_parsed(&self) -> Result<ConversationMessage, Error> {
        let string = self.get_string()?;
        let string = string
            .to_str()
            .map_err(|error| Error::custom_with_error("Response is not utf8", error))?;
        let parsed = serde_json::from_str(string)
            .map_err(|error| Error::custom_with_error("Failed to parse response json", error))?;
        Ok(parsed)
    }
}

impl Drop for JsonResponse {
    fn drop(&mut self) {
        unsafe { litert_lm_json_response_delete(self.ptr) }
    }
}
