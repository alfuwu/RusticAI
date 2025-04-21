use std::sync::Arc;
use serde_json::json;
use crate::{client::HeaderProvider, requester::*, types::*};

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
        
        return match req {
            Ok(json) => { Account::from_json(json.get("user").expect(ERR).get("user").expect(ERR)) },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_settings(&self) -> Settings {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/settings/",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        return match req {
            Ok(json) => { Settings::from_json(&json) },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_followers(&self) -> Vec<String> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/followers/",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        return match req {
            Ok(json) => { json.get("followers").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).into_iter().filter_map(|v| v.as_str().map(String::from)).collect() },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_following(&self) -> Vec<String> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/following/",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        return match req {
            Ok(json) => { json.get("following").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).into_iter().filter_map(|v| v.as_str().map(String::from)).collect() },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_persona(&self, persona: impl Into<String>) -> Persona {
        let req = self.requester.request_async(
            format!("https://plus.character.ai/chat/persona/?id={}", persona.into()),
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        return match req {
            Ok(json) => { Persona::from_json(&json) },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
        }
    }

    pub async fn fetch_personas(&self) -> Vec<Persona> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/personas/?force_refresh=1",
            RequestOptions::new("GET", self.header_provider.get_headers(None).await, None)
        ).await;
        
        return match req {
            Ok(json) => { json.get("personas").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).into_iter().filter_map(|v| Some(Persona::from_json(&v))).collect() },
            Err(_) => { panic!("AHHHH THE WORLD IS ENDING") }
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