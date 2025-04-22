use serde_json::{json, Map, Value};

use crate::types::{enums::*, media::*};

#[derive(Debug)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub description: String,
    pub definition: String,
    pub greeting: String,
    pub avatar: Option<Avatar>,
    pub visibility: Visibility,
    pub upvotes: Option<i64>,
    pub title: String,
    pub author_username: Option<String>,
    pub num_interactions: Option<i64>,
    pub internal_id: String,
    pub voice_id: String,
    pub default_voice_id: String,
    pub identifier: String,
    pub copyable: bool,
    pub starter_prompts: Map<String, Value>,
    pub comments_enabled: bool,
    pub songs: Vec<String>,
    pub image_gen_enabled: bool,
    pub base_image_prompt: String,
    pub image_prompt_regex: String,
    pub strip_image_prompt_from_message: bool
}

impl Character {
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>, definition: impl Into<String>, greeting: impl Into<String>, avatar: Option<Avatar>, visibility: Visibility, upvotes: Option<i64>, title: impl Into<String>, author_username: Option<String>, num_interactions: Option<i64>, internal_id: impl Into<String>, voice_id: impl Into<String>, default_voice_id: impl Into<String>, identifier: impl Into<String>, copyable: bool, starter_prompts: Map<String, Value>, comments_enabled: bool, songs: Vec<String>, image_gen_enabled: bool, base_image_prompt: impl Into<String>, image_prompt_regex: impl Into<String>, strip_image_prompt_from_message: bool) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            definition: definition.into(),
            greeting: greeting.into(),
            avatar,
            visibility,
            upvotes,
            title: title.into(),
            author_username,
            num_interactions,
            internal_id: internal_id.into(),
            voice_id: voice_id.into(),
            default_voice_id: default_voice_id.into(),
            identifier: identifier.into(),
            copyable,
            starter_prompts,
            comments_enabled,
            songs,
            image_gen_enabled,
            base_image_prompt: base_image_prompt.into(),
            image_prompt_regex: image_prompt_regex.into(),
            strip_image_prompt_from_message
        }
    }

    pub fn from_json(json: &Value) -> Self {
        let blank = json!("");
        let blank_bool = json!(false);

        Self::new(
            json.get("external_id").unwrap_or(&blank).as_str().unwrap(),
            json.get("participant__name").unwrap_or(json.get("name").unwrap_or(&blank)).as_str().unwrap(),
            json.get("description").unwrap_or(&blank).as_str().unwrap(),
            json.get("definition").unwrap_or(&blank).as_str().unwrap_or(""),
            json.get("greeting").unwrap_or(&blank).as_str().unwrap(),
            Avatar::from_json(&json),
            Visibility::from_str(json.get("visibility").unwrap_or(&json!("PUBLIC")).as_str().unwrap()),
            json.get("upvotes").and_then(|v| v.as_i64().map(i64::from)),
            json.get("title").unwrap_or(&blank).as_str().unwrap(),
            json.get("user__username").and_then(|v| v.as_str().map(String::from)),
            json.get("participant__num_interactions").and_then(|v| v.as_i64().map(i64::from)),
            json.get("participant__user__username").unwrap_or(&blank).as_str().unwrap(),
            json.get("voice_id").unwrap_or(&blank).as_str().unwrap(),
            json.get("default_voice_id").unwrap_or(&blank).as_str().unwrap(),
            json.get("identifier").unwrap_or(&blank).as_str().unwrap(),
            json.get("copyable").unwrap_or(&blank_bool).as_bool().unwrap(),
            json.get("starter_prompts").unwrap_or(&json!({})).as_object().unwrap().clone(),
            json.get("comments_enabled").unwrap_or(&blank_bool).as_bool().unwrap(),
            json.get("starter_prompts").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| v.as_str()).map(|v| v.to_string()).collect(),
            json.get("img_gen_enabled").unwrap_or(&blank_bool).as_bool().unwrap(),
            json.get("base_img_prompt").unwrap_or(&blank).as_str().unwrap(),
            json.get("img_prompt_regex").unwrap_or(&blank).as_str().unwrap(),
            json.get("strip_img_prompt_from_msg").unwrap_or(&blank_bool).as_bool().unwrap(),
        )
    }
}

#[derive(Debug)]
pub struct PartialCharacter {
    pub id: String,
    pub name: String,
    pub description: String,
    pub definition: String,
    pub greeting: String,
    pub avatar: Option<Avatar>,
    pub visibility: Visibility,
    pub upvotes: Option<i64>,
    pub title: String,
    pub author_username: Option<String>,
    pub num_interactions: Option<i64>
}

impl PartialCharacter {
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>, definition: impl Into<String>, greeting: impl Into<String>, avatar: Option<Avatar>, visibility: Visibility, upvotes: Option<i64>, title: impl Into<String>, author_username: Option<String>, num_interactions: Option<i64>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            definition: definition.into(),
            greeting: greeting.into(),
            avatar,
            visibility,
            upvotes,
            title: title.into(),
            author_username,
            num_interactions
        }
    }

    pub fn from_json(json: &Value) -> Self {
        let blank = json!("");

        Self::new(
            json.get("external_id").unwrap_or(&blank).as_str().unwrap(),
            json.get("participant__name").unwrap_or(json.get("name").unwrap_or(&blank)).as_str().unwrap(),
            json.get("description").unwrap_or(&blank).as_str().unwrap(),
            json.get("definition").unwrap_or(&blank).as_str().unwrap_or(""),
            json.get("greeting").unwrap_or(&blank).as_str().unwrap(),
            Avatar::from_json(&json),
            Visibility::from_str(json.get("visibility").unwrap_or(&json!("PUBLIC")).as_str().unwrap_or("PUBLIC")),
            json.get("upvotes").and_then(|v| v.as_i64().map(i64::from)),
            json.get("title").unwrap_or(&blank).as_str().unwrap(),
            json.get("user__username").and_then(|v| v.as_str().map(String::from)),
            json.get("participant__num_interactions").and_then(|v| v.as_i64().map(i64::from))
        )
    }
}