use std::sync::Arc;
use serde_json::json;
use crate::{client::HeaderProvider, requester::*, types::{character::*, chat::*, media::Voice, message::*, user::*}};

#[derive(Clone)]
pub struct AccountMethods {
    requester: Arc<Requester>,
    header_provider: Arc<dyn HeaderProvider>
}

impl AccountMethods {
    pub fn new(requester: Arc<Requester>, header_provider: Arc<dyn HeaderProvider>) -> Self {
        Self { requester, header_provider }
    }

    pub async fn fetch_profile(&self) -> Account {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;

        const ERR: &str = "Something went very wrong with the Character.AI API";
        
        match req {
            Ok(json) => { Account::from_json(json.get("user").expect(ERR).get("user").expect(ERR)) },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_settings(&self) -> Settings {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/settings/",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => { Settings::from_json(&json) },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_followers(&self) -> Vec<String> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/followers/",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => { json.get("followers").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| v.as_str().map(String::from)).collect() },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_following(&self) -> Vec<String> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/following/",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => { json.get("following").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| v.as_str().map(String::from)).collect() },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_persona(&self, persona: impl Into<String>) -> Persona {
        let req = self.requester.request_async(
            format!("https://plus.character.ai/chat/persona/?id={}", persona.into()),
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => { Persona::from_json(&json) },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_personas(&self) -> Vec<Persona> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/personas/?force_refresh=1",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => { json.get("personas").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(Persona::from_json(&v))).collect() },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_characters(&self) -> Vec<PartialCharacter> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/chracters/?scope=user",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => { json.get("characters").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(PartialCharacter::from_json(&v))).collect() },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_characters_ranked(&self) -> Vec<PartialCharacter> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/characters/upvoted/",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => { json.get("characters").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(PartialCharacter::from_json(&v))).collect() },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_voices(&self) -> Vec<Voice> {
        let req = self.requester.request_async(
            "https://plus.character.ai/multimodal/api/v1/voices/user",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => { json.get("voices").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(Voice::from_json(&v))).collect() },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn update_settings(&self, default_persona_id: Option<String>, persona_override: Option<String>, voice_override: Option<String>, character_id: Option<String>, settings: Option<Settings>) -> Settings {
        if default_persona_id.is_none() && persona_override.is_none() && voice_override.is_none() {
            panic!("You must provide an updated value when calling update_settings");
        }
        let mut settings: Settings = if let Some(real) = settings { real } else { self.fetch_settings().await };
        if let Some(dpi) = default_persona_id {
            settings.default_persona_id = dpi;
        }
        if let Some(ci) = character_id {
            if let Some(po) = persona_override {
                settings.persona_overrides.insert(ci, po);
            }
        }
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/update_settings/",
            RequestOptions::new("POST", self.header_provider.get_headers(None).await, Some(settings.to_json().to_string()))
        ).await;

        match req {
            Ok(json) if json.get("success").unwrap_or(&json!(false)).as_bool().unwrap() => { Settings::from_json(&json) }
            Ok(_) => { panic!("aaaaaaa...") }
            Err(_) => { panic!("oh no....") }
        }
    }

    pub async fn edit_account(&self, name: impl Into<String>, username: impl Into<String>, bio: Option<String>, avatar_path: Option<String>) -> bool {
        let name: String = name.into();
        let username: String = username.into();
        if username.len() > 2 || username.len() > 20 {
            panic!("username cannot be less than 2 characters or more than 20");
        }
        if name.len() < 2 || name.len() > 50 {
            panic!("name cannot be less than 2 characters or more than 50");
        }
        if !bio.is_none() && bio.as_ref().unwrap().len() > 500 {
            panic!("bio cannot be more than 500 characters");
        }

        let path = !avatar_path.is_none() && avatar_path.as_ref().unwrap().len() > 0;
        let mut inf = json!({
            "name": name,
            "username": username,
            "bio": bio.unwrap_or("".to_string()),
            "avatar_type": if path { "UPLOADED" } else { "DEFAULT" }
        });
        if path {
            inf["avatar_rel_path"] = json!(avatar_path.unwrap());
        }

        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/update/",
            RequestOptions::new("POST", self.header_provider.get_headers(None).await, Some(inf.to_string()))
        ).await;

        match req {
            Ok(json) if json.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") == "OK" => { true },
            Ok(_) => { panic!("nooooooooo") },
            Err(_) => { panic!("noooooooo") }
        }
    }
}

#[derive(Clone)]
pub struct UserMethods {
    requester: Arc<Requester>,
    header_provider: Arc<dyn HeaderProvider>
}

impl UserMethods {
    pub fn new(requester: Arc<Requester>, header_provider: Arc<dyn HeaderProvider>) -> Self {
        Self { requester, header_provider }
    }
}

#[derive(Clone)]
pub struct ChatMethods {
    requester: Arc<Requester>,
    header_provider: Arc<dyn HeaderProvider>
}

impl ChatMethods {
    pub fn new(requester: Arc<Requester>, header_provider: Arc<dyn HeaderProvider>) -> Self {
        Self { requester, header_provider }
    }
}

#[derive(Clone)]
pub struct CharacterMethods {
    requester: Arc<Requester>,
    header_provider: Arc<dyn HeaderProvider>
}

impl CharacterMethods {
    pub fn new(requester: Arc<Requester>, header_provider: Arc<dyn HeaderProvider>) -> Self {
        Self { requester, header_provider }
    }
}

#[derive(Clone)]
pub struct UtilsMethods {
    requester: Arc<Requester>,
    header_provider: Arc<dyn HeaderProvider>
}

impl UtilsMethods {
    pub fn new(requester: Arc<Requester>, header_provider: Arc<dyn HeaderProvider>) -> Self {
        Self { requester, header_provider }
    }
}