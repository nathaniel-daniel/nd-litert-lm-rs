use serde::de::Error;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, serde::Deserialize, serde::Serialize)]
pub enum ConversationMessageRole {
    #[serde(rename = "system")]
    System,

    #[serde(rename = "user")]
    User,

    #[serde(rename = "assistant")]
    Model,

    #[serde(rename = "tool")]
    Tool,
}

// TODO: Consider splitting Image and Audio into 2 different variants,
// and writing manual serde impls.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum ConversationMessageContent {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image")]
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(
            with = "optional_base64_bytes",
            skip_serializing_if = "Option::is_none"
        )]
        blob: Option<Vec<u8>>,
    },

    #[serde(rename = "audio")]
    Audio {
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(
            with = "optional_base64_bytes",
            skip_serializing_if = "Option::is_none"
        )]
        blob: Option<Vec<u8>>,
    },

    #[serde(rename = "tool_response")]
    ToolResponse {
        name: String,
        response: HashMap<String, String>,
    },
}

#[derive(Debug)]
pub struct ToolCall {
    pub name: String,
    pub arguments: HashMap<String, String>,
}

impl<'de> serde::Deserialize<'de> for ToolCall {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, serde::Deserialize)]
        struct ToolCallFunctionJson {
            name: String,
            arguments: HashMap<String, String>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct ToolCallJson {
            #[serde(rename = "type")]
            type_: String,
            function: ToolCallFunctionJson,
        }

        let json = ToolCallJson::deserialize(deserializer)?;
        if json.type_ != "function" {
            return Err(D::Error::custom("\"type_\" field should be \"function\""));
        }

        Ok(Self {
            name: json.function.name,
            arguments: json.function.arguments,
        })
    }
}

impl serde::Serialize for ToolCall {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Debug, serde::Serialize)]
        struct ToolCallFunctionJson<'a> {
            name: &'a str,
            arguments: &'a HashMap<String, String>,
        }

        #[derive(Debug, serde::Serialize)]
        struct ToolCallJson<'a> {
            #[serde(rename = "type")]
            type_: &'static str,
            function: ToolCallFunctionJson<'a>,
        }

        let json = ToolCallJson {
            type_: "function",
            function: ToolCallFunctionJson {
                name: &self.name,
                arguments: &self.arguments,
            },
        };

        json.serialize(serializer)
    }
}

mod optional_base64_bytes {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;
    use serde::de::Error;

    pub fn serialize<S>(optional_bytes: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match optional_bytes {
            Some(bytes) => {
                let encoded = STANDARD.encode(bytes);
                serializer.serialize_some(&encoded)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let optional_string = Option::<String>::deserialize(deserializer)?;
        match optional_string {
            Some(string) => Ok(Some(
                STANDARD
                    .decode(string.as_bytes())
                    .map_err(D::Error::custom)?,
            )),
            None => Ok(None),
        }
    }
}
