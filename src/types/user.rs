use std::collections::HashMap;
use serde_json::{json, Value};

use crate::types::media::*;

use super::character::PartialCharacter;

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub id: i64,
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
    pub fn new(id: i64, email: Option<String>, username: impl Into<String>, name: impl Into<String>, bio: impl Into<String>, avatar: Option<Avatar>, avatar_type: Option<String>, first_name: Option<String>, is_human: bool) -> Self {
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

    pub fn default() -> Self {
        Self::new(
            0,
            None,
            "".to_string(),
            "".to_string(),
            "".to_string(),
            None,
            None,
            None,
            false
        )
    }

    pub fn from_json(json: &Value) -> Self {
        let acc = json.get("account").expect("account key should be present within JSON Value provided");

        let blank = json!("");

        Self::new(
            json.get("id").unwrap_or(&json!(0)).as_i64().unwrap(),
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

    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "email": self.email,
            "username": self.username,
            "bio": self.bio,
            "first_name": self.first_name,
            "is_human": self.is_human,
            "account": {
                "name": self.name,
                "avatar_file_name": self.avatar.as_ref().map(|a| &a.file_name),
                "avatar_type": self.avatar_type
            }
        })
    }
}

impl AsRef<Account> for Account {
    fn as_ref(&self) -> &Account {
        self
    }
}

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub name: String,
    pub bio: String,
    pub avatar: Option<Avatar>,
    pub num_following: i32,
    pub num_followers: i32,
    pub characters: Vec<PartialCharacter>,
    pub subscription_type: String
}

impl User {
    pub fn new(username: impl Into<String>, name: impl Into<String>, bio: impl Into<String>, avatar: Option<Avatar>, num_following: i32, num_followers: i32, characters: Vec<PartialCharacter>, subscription_type: impl Into<String>) -> Self {
        Self { 
            username: username.into(),
            name: name.into(),
            bio: bio.into(),
            avatar,
            num_following,
            num_followers,
            characters,
            subscription_type: subscription_type.into()
        }
    }

    pub fn from_json(json: &Value) -> Self {
        let blank = json!("");
        let blank_num = json!(0);

        Self::new(
            json.get("username").unwrap_or(&blank).as_str().unwrap(),
            json.get("name").unwrap_or(&blank).as_str().unwrap(),
            json.get("bio").unwrap_or(&blank).as_str().unwrap(),
            Avatar::from_json(&json),
            json.get("num_following").unwrap_or(&blank_num).as_i64().unwrap() as i32,
            json.get("num_followers").unwrap_or(&blank_num).as_i64().unwrap() as i32,
            json.get("characters").unwrap_or(&json!([])).as_array().unwrap().into_iter().map(|v| PartialCharacter::from_json(&v)).collect(),
            json.get("subscription_type").unwrap_or(&blank).as_str().unwrap_or("NONE")
        )
    }

    pub fn to_json(&self) -> Value {
        json!({
            "username": self.username,
            "name": self.name,
            "bio": self.bio,
            "avatar_file_name": self.avatar.as_ref().map(|a| &a.file_name),
            "num_following": self.num_following,
            "num_followers": self.num_followers,
            "characters": self.characters.iter().map(|c| c.to_json()).collect::<Vec<_>>(),
            "subscription_type": self.subscription_type
        })
    }
}

#[derive(Debug)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub greeting: String,
    pub description: String,
    pub definition: String,
    pub avatar: Option<Avatar>,
    pub archived: bool,
    pub author_username: Option<String>
}

impl Persona {
    pub fn new(id: impl Into<String>, name: impl Into<String>, greeting: impl Into<String>, description: impl Into<String>, definition: impl Into<String>, avatar: Option<Avatar>, archived: bool, author_username: Option<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            greeting: greeting.into(),
            description: description.into(),
            definition: definition.into(),
            archived,
            avatar,
            author_username
        }
    }

    pub fn default() -> Self {
        Self::new(
            "".to_string(),
            "".to_string(),
            "Hello! This is my persona".to_string(),
            "This is my persona.".to_string(),
            "".to_string(),
            None,
            true,
            None
        )
    }

    pub fn from_json(json: &Value) -> Self {
        let blank = json!("");

        Self::new(
            json.get("external_id").unwrap_or(&blank).as_str().unwrap(),
            json.get("participant__name").unwrap_or(json.get("name").unwrap_or(&blank)).as_str().unwrap(),
            json.get("greeting").unwrap_or(&blank).as_str().unwrap_or("Hello! This is my persona"),
            json.get("description").unwrap_or(&blank).as_str().unwrap_or("This is my persona."),
            json.get("definition").unwrap_or(&blank).as_str().unwrap(),
            Avatar::from_json(&json),
            json.get("archived").unwrap_or(&json!(false)).as_bool().unwrap_or(false),
            json.get("author_username").and_then(|v| v.as_str().map(String::from))
        )
    }
    
    pub fn to_json(&self) -> Value {
        json!({
            "external_id": self.id,
            "participant__name": self.name,
            "definition": self.definition,
            "avatar_file_name": self.avatar.as_ref().map(|a| &a.file_name),
            "author_username": self.author_username
        })
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