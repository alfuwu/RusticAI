use std::collections::HashMap;
use serde_json::{json, Value};

use crate::types::media::*;

#[derive(Debug)]
pub struct Account {
    pub id: i32,
    pub email: Option<String>,
    pub username: String,
    pub name: String,
    pub bio: String,
    pub avatar: Option<Avatar>,
    pub avatar_type: String,
    pub first_name: Option<String>,
    pub is_human: bool
}

impl Account {
    pub fn new(id: i32, email: Option<String>, username: impl Into<String>, name: impl Into<String>, bio: impl Into<String>, avatar: Option<Avatar>, avatar_type: Option<String>, first_name: Option<String>, is_human: bool) -> Self {
        Self { 
            id,
            email,
            username: username.into(),
            name: name.into(),
            bio: bio.into(),
            avatar,
            avatar_type: if let Some(avi) = avatar_type { avi } else { "DEFAULT".to_string() },
            first_name,
            is_human
        }
    }

    pub fn from_json(json: &Value) -> Self {
        let acc = json.get("account").expect("account key should be present within JSON Value provided");

        let blank = json!("");

        Self::new(
            json.get("id").unwrap_or(&json!(0)).as_i64().unwrap() as i32,
            json.get("email").and_then(|v| v.as_str().map(String::from)),
            json.get("username").unwrap_or(&blank).as_str().unwrap(),
            acc.get("name").unwrap_or(&blank).as_str().unwrap(),
            json.get("bio").unwrap_or(&blank).as_str().unwrap(),
            Avatar::from_json(&acc),
            acc.get("avatar_type").and_then(|v| v.as_str().map(String::from)),
            json.get("first_name").and_then(|v| v.as_str().map(String::from)),
            json.get("is_human").unwrap_or(&json!(true)).as_bool().unwrap()
        )
    }
}

#[derive(Debug)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub definition: String,
    pub avatar: Option<Avatar>,
    pub author_username: Option<String>
}

impl Persona {
    pub fn new(id: impl Into<String>, name: impl Into<String>, definition: impl Into<String>, avatar: Option<Avatar>, author_username: Option<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            definition: definition.into(),
            avatar,
            author_username
        }
    }

    pub fn from_json(json: &Value) -> Self {
        let blank = json!("");

        Self::new(
            json.get("external_id").unwrap_or(&blank).as_str().unwrap(),
            json.get("participant__name").unwrap_or(json.get("name").unwrap_or(&blank)).as_str().unwrap(),
            json.get("definition").unwrap_or(&blank).as_str().unwrap(),
            Avatar::from_json(&json),
            json.get("author_username").and_then(|v| v.as_str().map(String::from))
        )
    }
}

#[derive(Debug)]
pub struct Settings {
    pub default_persona_id: String,
    pub discord_settings: Value,
    pub model_preference_settings: Value,
    pub output_style_settings: Value,
    pub persona_overrides: HashMap<String, String>
}

impl Settings {
    pub fn new(default_persona_id: impl Into<String>, discord_settings: Value, model_preference_settings: Value, output_style_settings: Value, persona_overrides: HashMap<String, String>) -> Self {
        Self {
            default_persona_id: default_persona_id.into(),
            discord_settings,
            model_preference_settings,
            output_style_settings,
            persona_overrides
        }
    }

    pub fn default() -> Self {
        Self {
            default_persona_id: "".to_string(),
            discord_settings: Value::Null,
            model_preference_settings: Value::Null,
            output_style_settings: Value::Null,
            persona_overrides: HashMap::new()
        }
    }

    pub fn from_json(json: &Value) -> Self {
        Self::new(
            json.get("default_persona_id").unwrap_or(&json!("")).as_str().unwrap(),
            json.get("discordSettings").unwrap_or(&Value::Null).clone(),
            json.get("modelPreferenceSettings").unwrap_or(&Value::Null).clone(),
            json.get("outputStyleSettings").unwrap_or(&Value::Null).clone(),
            json.get("personaOverrides").unwrap_or(&json!({})).as_object().unwrap().into_iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect()
        )
    }

    pub fn to_json(&self) -> Value {
        json!({
            "default_persona_id": self.default_persona_id,
            "discordSettings": self.discord_settings,
            "modelPreferenceSettings": self.model_preference_settings,
            "outputStyleSettings": self.output_style_settings,
            "personaOverrides": self.persona_overrides
        })
    }
}