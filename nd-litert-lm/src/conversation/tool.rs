use serde::de::Error;
use std::collections::HashMap;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum Property {
    #[serde(rename = "string")]
    String { description: Option<String> },

    #[serde(rename = "integer")]
    Integer { description: Option<String> },

    #[serde(rename = "number")]
    Number { description: Option<String> },

    #[serde(rename = "boolean")]
    Boolean { description: Option<String> },

    #[serde(rename = "array")]
    Array {
        description: Option<String>,
        items: Box<Property>,
    },

    #[serde(rename = "object")]
    Object {
        description: Option<String>,
        properties: HashMap<String, Property>,
    },
}

#[derive(Debug)]
pub struct FunctionParameters {
    /// Properties
    pub properties: HashMap<String, Property>,

    /// The list of names of required parameters.
    pub required: Vec<String>,
}

impl<'de> serde::Deserialize<'de> for FunctionParameters {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct FunctionParameters {
            type_: String,

            #[serde(default)]
            properties: HashMap<String, Property>,

            #[serde(default)]
            required: Vec<String>,
        }

        let json = FunctionParameters::deserialize(deserializer)?;
        if json.type_ != "function" {
            return Err(D::Error::custom("\"type_\" field should be \"object\""));
        }

        Ok(Self {
            properties: json.properties,
            required: json.required,
        })
    }
}

impl serde::Serialize for FunctionParameters {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct FunctionParameters<'a> {
            type_: &'static str,

            #[serde(default)]
            properties: &'a HashMap<String, Property>,

            #[serde(default)]
            required: &'a Vec<String>,
        }

        let json = FunctionParameters {
            type_: "object",
            properties: &self.properties,
            required: &self.required,
        };

        json.serialize(serializer)
    }
}
