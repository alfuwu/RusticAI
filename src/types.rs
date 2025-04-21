use serde_json::{json, Value};

#[derive(Debug)]
pub struct Account {
    pub username: String,
    pub name: String,
    pub bio: String,
    pub avatar: Option<Avatar>,
    pub account_id: i32,
    pub first_name: Option<String>,
    pub avatar_type: String,
    pub is_human: bool,
    pub email: Option<String>
}

impl Account {
    pub fn new(username: impl Into<String>, name: impl Into<String>, bio: impl Into<String>, avatar: Option<Avatar>, avatar_file_name: Option<String>, account_id: i32, first_name: Option<String>, avatar_type: Option<String>, is_human: bool, email: Option<String>) -> Self {
        Self { 
            username: username.into(),
            name: name.into(),
            bio: bio.into(),
            avatar: if avatar.is_none() { if let Some(avi) = avatar_file_name { Some(Avatar::new(&avi)) } else { None } } else { avatar },
            account_id,
            first_name,
            avatar_type: if let Some(avi) = avatar_type { avi } else { "DEFAULT".to_string() },
            is_human,
            email
        }
    }

    pub fn from_json(json: &Value) -> Self {
        let acc = json.get("account").expect("account key should be present within JSON Value provided");

        let blank = json!("");

        Self::new(
            json.get("username").unwrap_or(&blank).as_str().unwrap_or("").to_string(),
            acc.get("name").unwrap_or(&blank).as_str().unwrap_or("").to_string(),
            json.get("bio").unwrap_or(&blank).as_str().unwrap_or("").to_string(),
            None,
            acc.get("avatar_file_name").and_then(|v| v.as_str().map(String::from)),
            json.get("id").unwrap_or(&json!(-1)).as_i64().unwrap_or(-1) as i32,
            json.get("first_name").and_then(|v| v.as_str().map(String::from)),
            acc.get("avatar_type").and_then(|v| v.as_str().map(String::from)),
            json.get("is_human").unwrap_or(&json!(true)).as_bool().unwrap_or(true),
            json.get("email").and_then(|v| v.as_str().map(String::from)),
        )
    }
}

#[derive(Debug)]
pub struct Avatar {
    pub file_name: String,
}

impl Avatar {
    pub fn new(file_name: impl Into<String>) -> Self {
        Self { file_name: file_name.into() }
    }

    pub fn get_url(&self, size: i32, animated: bool) -> String {
        format!("https://characterai.io/i/{}/static/avatars/{}?webp=true&anim={}", size, self.file_name, if animated { 1 } else { 0 })
    }

    pub fn get_default_url(&self) -> String {
        self.get_url(400, false)
    }
}

#[derive(Debug)]
pub struct Persona {

}

impl Persona {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn from_json(json: &Value) -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Settings {

}

impl Settings {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn from_json(json: &Value) -> Self {
        Self::new()
    }
}