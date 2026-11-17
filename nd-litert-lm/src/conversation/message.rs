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
