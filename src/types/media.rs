use serde_json::{json, Value};

use crate::types::enums::*;

#[derive(Debug)]
pub struct Avatar {
    pub file_name: String,
}

impl Avatar {
    pub fn new(file_name: impl Into<String>) -> Self {
        Self { file_name: file_name.into() }
    }

    pub fn from_json(json: &Value) -> Option<Self> {
        if let Some(avi) = json.get("avatar_file_name").and_then(|v| v.as_str().map(String::from)) { Some(Avatar::new(avi)) } else { None }
    }

    pub fn get_url(&self, size: i32, animated: bool) -> String {
        format!("https://characterai.io/i/{}/static/avatars/{}?webp=true&anim={}", size, self.file_name, if animated { 1 } else { 0 })
    }

    pub fn get_default_url(&self) -> String {
        self.get_url(400, false)
    }
}

#[derive(Debug)]
pub struct Voice {
    pub id: String,
    pub name: String,
    pub description: String,
    pub gender: Gender,
    pub visibility: Visibility,
    pub preview_text: String,
    pub preview_audio_uri: Option<String>,
    pub creator_id: Option<String>,
    pub creator_username: Option<String>,
    pub last_update: Option<String>,
    pub internal_status: String
}

impl Voice {
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>, gender: Gender, visibility: Visibility, preview_text: impl Into<String>, preview_audio_uri: Option<String>, creator_id: Option<String>, creator_username: Option<String>, last_update: Option<String>, internal_status: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            gender,
            visibility,
            preview_text: preview_text.into(),
            preview_audio_uri: preview_audio_uri.into(),
            creator_id,
            creator_username,
            last_update,
            internal_status: internal_status.into()
        }
    }

    pub fn from_json(json: &Value) -> Self {
        let blank = json!({});
        let creator = json.get("creatorInfo").unwrap_or(&blank);

        let blank = json!("");
        println!("{:?}", json);

        Self::new(
            json.get("id").unwrap_or(&blank).as_str().unwrap(),
            json.get("name").unwrap_or(&blank).as_str().unwrap(),
            json.get("description").unwrap_or(&blank).as_str().unwrap(),
            Gender::from_str(json.get("gender").unwrap_or(&json!("neutral")).as_str().unwrap()),
            Visibility::from_str(json.get("visibility").unwrap_or(&json!("PRIVATE")).as_str().unwrap()),
            json.get("preview_text").unwrap_or(&blank).as_str().unwrap(),
            json.get("preview_audio_uri").and_then(|v| v.as_str().map(String::from)),
            creator.get("id").and_then(|v| v.as_str().map(String::from)),
            creator.get("username").and_then(|v| v.as_str().map(String::from)),
            json.get("last_update").and_then(|v| v.as_str().map(String::from)),
            json.get("internal_status").unwrap_or(&blank).as_str().unwrap(),
        )
    }
}