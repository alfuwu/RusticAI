use std::{sync::Arc, collections::HashMap};
use rand::Rng;
use base64::{Engine, engine::general_purpose};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{client::AsyncClient, requester::*, types::{character::*, chat::*, enums::Visibility, media::*, message::*, user::*}};

#[derive(Clone)]
pub struct AccountMethods {
    requester: Arc<Requester>,
    client: Arc<AsyncClient>
}

impl AccountMethods {
    pub fn new(requester: Arc<Requester>, client: Arc<AsyncClient>) -> Self {
        Self { requester, client }
    }

    pub async fn fetch_profile(&self) -> Account {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;

        const ERR: &str = "Something went wrong with the Character.AI API; fetch_profile response was missing user object";
        
        match req {
            Ok(json) => Account::from_json(json.get("user").expect(ERR).get("user").expect(ERR)),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn fetch_settings(&self) -> Settings {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/settings/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => Settings::from_json(&json),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn fetch_followers(&self) -> Vec<String> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/followers/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => json.get("followers").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| v.as_str().map(String::from)).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn fetch_following(&self) -> Vec<String> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/following/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => json.get("following").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| v.as_str().map(String::from)).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn fetch_persona(&self, id: impl Into<&String>) -> Persona {
        let id: &String = id.into();
        let req = self.requester.request_async(
            format!("https://plus.character.ai/chat/persona/?id={}", id),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => Persona::from_json(&json),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn fetch_personas(&self) -> Vec<Persona> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/personas/?force_refresh=1",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => json.get("personas").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(Persona::from_json(&v))).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn fetch_characters(&self) -> Vec<PartialCharacter> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/chracters/?scope=user",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => json.get("characters").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(PartialCharacter::from_json(&v))).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn fetch_characters_ranked(&self) -> Vec<PartialCharacter> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/characters/upvoted/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => json.get("characters").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(PartialCharacter::from_json(&v))).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn fetch_voices(&self) -> Vec<Voice> {
        let req = self.requester.request_async(
            "https://plus.character.ai/multimodal/api/v1/voices/user",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
        
        match req {
            Ok(json) => json.get("voices").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(Voice::from_json(&v))).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }

    async fn update_settings(&self, default_persona_id: Option<&String>, persona_override: Option<&String>, voice_override: Option<&String>, character_id: Option<&String>, settings: Option<&mut Settings>) -> Option<Settings> {
        if default_persona_id.is_none() && persona_override.is_none() && voice_override.is_none() {
            panic!("You must provide an updated value when calling update_settings");
        }
        let settings: &mut Settings = if let Some(real) = settings { real } else { &mut self.fetch_settings().await };
        if let Some(dpi) = default_persona_id {
            settings.default_persona_id = dpi.to_string();
        }
        if let Some(ci) = character_id {
            if let Some(po) = persona_override {
                settings.persona_overrides.insert(ci.to_string(), po.to_string());
            }
        }
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/update_settings/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(settings.to_json().to_string().into()))
        ).await;

        match req {
            Ok(json) => {
                if json.get("success").unwrap_or(&json!(false)).as_bool().unwrap() {
                    println!("cargo:warn=Malformed data received.\n{:?}", json);
                }
                Some(Settings::from_json(&json))
            },
            Err(data) => { println!("cargo:warn={:?}", data); None }
        }
    }

    pub async fn edit_account(&self, name: impl Into<&String>, username: impl Into<&String>, bio: Option<&String>, avatar_path: Option<&String>) -> bool {
        let name: &String = name.into();
        let username: &String = username.into();
        let bio: &String = if let Some(v) = bio { v } else { &"".to_string() };
        let avatar_path: &String = if let Some(v) = avatar_path { v } else { &"".to_string() };

        if username.len() < 2 || username.len() > 20 {
            panic!("username cannot be less than 2 characters or more than 20 (is {} characters)", username.len());
        }
        if name.len() < 2 || name.len() > 50 {
            panic!("name cannot be less than 2 characters or more than 50 (is {} characters)", name.len());
        }
        if bio.len() > 500 {
            panic!("bio cannot be more than 500 characters (is {} characters)", bio.len());
        }

        let path = avatar_path.len() > 0;
        let mut inf = json!({
            "name": name,
            "username": username,
            "bio": bio,
            "avatar_type": if path { "UPLOADED" } else { "DEFAULT" }
        });
        if path {
            inf["avatar_rel_path"] = json!(avatar_path);
        }

        let req = self.requester.request_async(
            "https://plus.character.ai/chat/user/update/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(inf.to_string().into()))
        ).await;

        match req {
            Ok(json) => {
                if json.get("status").unwrap_or(&json!("OK")).as_str().unwrap_or("OK") != "OK" {
                    panic!("{}", json.get("error").unwrap_or(&json!("")));
                }
                true
            },
            Err(data) => { println!("cargo:warn={:?}", data); false }
        }
    }

    pub async fn create_persona(&self, name: impl Into<String>, definition: Option<String>, avatar_path: Option<String>) -> Persona {
        let name: String = name.into();
        let definition: String = definition.unwrap_or_else(|| "".to_string());
        let avatar_path: String = avatar_path.unwrap_or_else(|| "".to_string());

        if name.len() < 3 || name.len() > 20 {
            panic!("name cannot be less than 3 characters or more than 20 (is {} characters)", name.len());
        }
        if definition.len() > 720 {
            panic!("definition cannot be more than 720 characters (is {} characters)", definition.len());
        }

        let req = self.requester.request_async(
            "https://plus.character.ai/chat/persona/create/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "avatar_file_name": "",
                "avatar_rel_path": avatar_path,
                "base_img_prompt": "",
                "categories": [],
                "copyable": false,
                "definition": definition,
                "description": "This is my persona.",
                "greeting": "Hello! This is my persona",
                "identifier": format!("id:{}", Uuid::new_v4().to_string()),
                "img_gen_enabled": false,
                "name": name,
                "strip_img_prompt_from_msg": false,
                "title": name,
                "visibility": "PRIVATE",
                "voice_id": "",
            }).to_string().into()))
        ).await;

        match req {
            Ok(json) => {
                if json.get("status").unwrap_or(&json!("OK")).as_str().unwrap_or("OK") != "OK" || json.get("persona").unwrap_or(&Value::Null) == &Value::Null {
                    panic!("{}", json.get("error").unwrap_or(&json!("")));
                }
                Persona::from_json(&json.get("persona").unwrap_or(&json!({})))
            },
            Err(data) => panic!("{:?}", data)
        }
    }

    async fn update_persona_internal(&self, json: Value) -> Persona {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/persona/update/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json.to_string().into()))
        ).await;

        match req {
            Ok(json) => {
                if json.get("status").unwrap_or(&json!("OK")).as_str().unwrap_or("OK") != "OK" || json.get("persona").unwrap_or(&Value::Null) == &Value::Null {
                    panic!("{}", json.get("error").unwrap_or(&json!("")));
                }
                Persona::from_json(&json.get("persona").unwrap_or(&json!({})))
            },
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn edit_persona(&self, id: impl Into<&String>, name: Option<&String>, definition: Option<&String>, avatar_path: Option<&String>, persona: Option<&Persona>) -> Persona {
        let id: &String = id.into();
        let persona: &Persona = if let Some(v) = persona { v } else { &Persona::default() };
        let account: Account = self.client.data().await;

        let mut name: &String = if let Some(v) = name { v } else { &"".to_string() };
        let definition: &String = if let Some(v) = definition { v } else { &"".to_string() };
        let avatar_path: &String = if let Some(v) = avatar_path { v } else { &"".to_string() };

        if name.len() < 3 || name.len() > 20 {
            println!("cargo:warning=name cannot be less than 3 characters or more than 20 characters (is {} characters). Falling back to provided persona name", name.len());
            name = &persona.name;
            if name.len() < 3 || name.len() > 20 {
                // There wasn't a provided persona, causing the persona variable to be the default persona, and thus have an invalid name, or the provided persona already had an invalid name somehow.
                panic!("name cannot be less than 3 characters or more than 20 characters (is {} characters). Please provide a Persona object to `edit_persona` for it to fall back to if you plan to trigger this data validation error.", name.len());
            }
        }
        if definition.len() > 720 {
            panic!("definition cannot be more than 720 characters (is {} characters)", definition.len());
        }

        self.update_persona_internal(json!({
            "avatar_file_name": avatar_path,
            "avatar_rel_path": avatar_path,
            "default_voice_id": "",
            "is_persona": true,
            "copyable": false,
            "definition": definition,
            "external_id": id,
            "description": persona.description,
            "greeting": persona.greeting,
            "enabled": false,
            "img_gen_enabled": false,
            "name": name,
            "participant__name": name,
            "participant__num_interactions": 0,
            "strip_img_prompt_from_msg": false,
            "title": name,
            "user__id": account.id,
            "user__username": account.username,
            "visibility": "PRIVATE",
        })).await
    }

    pub async fn delete_persona(&self, id: impl Into<&String>, persona: Option<&Persona>) -> Persona {
        let id: &String = id.into();
        let persona: &Persona = if let Some(v) = persona { v } else { &self.fetch_persona(id).await };
        let p: String = if let Some(v) = persona.avatar.clone() { v.file_name } else { "".to_string() };
        let account: Account = self.client.data().await;

        self.update_persona_internal(json!({
            "archived": true,
            "avatar_file_name": p,
            "avatar_rel_path": p,
            "default_voice_id": "",
            "is_persona": true,
            "copyable": false,
            "definition": persona.definition,
            "external_id": id,
            "description": persona.description,
            "greeting": persona.greeting,
            "enabled": false,
            "img_gen_enabled": false,
            "name": persona.name,
            "participant__name": persona.name,
            "participant__num_interactions": 0,
            "strip_img_prompt_from_msg": false,
            "title": persona.name,
            "user__id": account.id,
            "user__username": account.username,
            "visibility": "PRIVATE",
        })).await
    }

    pub async fn set_default_persona(&self, id: Option<&String>, settings: Option<&mut Settings>) -> bool {
        let id: &String = if let Some(v) = id { v } else { &"".to_string() };
        !self.update_settings(Some(id), None, None, None, settings).await.is_none()
    }

    pub async fn set_persona(&self, character_id: impl Into<&String>, persona_id: Option<&String>, settings: Option<&mut Settings>) -> bool {
        let character_id: &String = character_id.into();
        let persona_id: &String = if let Some(v) = persona_id { v } else { &"".to_string() };
        !self.update_settings(None, Some(persona_id), None, Some(character_id), settings).await.is_none()
    }

    pub async fn set_voice(&self, voice_id: Option<&String>, settings: Option<&mut Settings>) -> bool {
        let voice_id: &String = if let Some(v) = voice_id { v } else { &"".to_string() };
        !self.update_settings(None, None, Some(voice_id), None, settings).await.is_none()
    }
}

#[derive(Clone)]
pub struct UserMethods {
    requester: Arc<Requester>,
    client: Arc<AsyncClient>
}

impl UserMethods {
    pub fn new(requester: Arc<Requester>, client: Arc<AsyncClient>) -> Self {
        Self { requester, client }
    }
}

#[derive(Clone)]
pub struct ChatMethods {
    requester: Arc<Requester>,
    client: Arc<AsyncClient>
}

impl ChatMethods {
    pub fn new(requester: Arc<Requester>, client: Arc<AsyncClient>) -> Self {
        Self { requester, client }
    }
}

#[derive(Clone)]
pub struct CharacterMethods {
    requester: Arc<Requester>,
    client: Arc<AsyncClient>
}

impl CharacterMethods {
    pub fn new(requester: Arc<Requester>, client: Arc<AsyncClient>) -> Self {
        Self { requester, client }
    }

    pub async fn fetch_characters_by_category(&self) -> HashMap<String, Vec<PartialCharacter>> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/curated_categories/characters/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
    
        match req {
            Ok(json) => {
                let mut result = HashMap::new();
                for (k, v) in json.get("characters_by_curated_category").unwrap_or(&json!({})).as_object().unwrap_or(&serde_json::Map::new()) {
                    let characters = v.as_array().unwrap_or(&vec![])
                        .iter().map(|c| PartialCharacter::from_json(c)).collect();
                    result.insert(k.to_string(), characters);
                }
                result
            },
            Err(data) => panic!("{:?}", data)
        }
    }
    
    pub async fn fetch_recommended_characters(&self) -> Vec<PartialCharacter> {
        let req = self.requester.request_async(
            "https://neo.character.ai/recommendation/v1/user",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
    
        match req {
            Ok(json) => json.get("characters").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| PartialCharacter::from_json(c)).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }
    
    pub async fn fetch_featured_characters(&self) -> Vec<PartialCharacter> {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/characters/featured_v2/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
    
        match req {
            Ok(json) => json.get("characters").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| PartialCharacter::from_json(c)).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn fetch_similar_characters(&self, character_id: impl Into<String>) -> Vec<PartialCharacter> {
        let req = self.requester.request_async(
            format!("https://neo.character.ai/recommendation/v1/character/{}", character_id.into()),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
    
        match req {
            Ok(json) => json.get("characters").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| PartialCharacter::from_json(c)).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }
    
    pub async fn fetch_character_info(&self, character_id: impl Into<String>) -> Character {
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/character/info/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({ "external_id": character_id.into() }).to_string().into()))
        ).await;
    
        match req {
            Ok(json) => {
                if json.get("status").unwrap_or(&json!("OK")).as_str().unwrap_or("OK") != "OK" {
                    panic!("{}", json.get("error").unwrap_or(&json!("")));
                }
                Character::from_json(json.get("character").unwrap_or(&json!({})))
            },
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn search_characters(&self, character_name: impl Into<&String>) -> Vec<PartialCharacter> {
        let character_name = character_name.into();
        let payload = json!({
            "0": {
                "json": {
                    "searchQuery": urlencoding::encode(&character_name)
                }
            }
        });
        
        let req = self.requester.request_async(
            format!("https://character.ai/api/trpc/search.search?batch=1&input={}", payload.to_string()),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
    
        match req {
            Ok(json) => json[0]["result"]["data"]["json"]["characters"].as_array().unwrap_or(&vec![]).iter().map(|c| PartialCharacter::from_json(c)).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }
    
    pub async fn search_creators(&self, creator_name: impl Into<String>) -> Vec<String> {
        let req = self.requester.request_async(
            format!("https://plus.character.ai/chat/creators/search/?query={}", urlencoding::encode(&creator_name.into())),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
    
        match req {
            Ok(json) => json.get("creators").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| c["name"].as_str().unwrap_or("").to_string()).collect(),
            Err(data) => panic!("Cannot search for creators.\n{:?}", data)
        }
    }
}

#[derive(Clone)]
pub struct UtilsMethods {
    requester: Arc<Requester>,
    client: Arc<AsyncClient>
}

impl UtilsMethods {
    pub fn new(requester: Arc<Requester>, client: Arc<AsyncClient>) -> Self {
        Self { requester, client }
    }

    pub async fn fetch_voice(&self, voice_id: impl Into<&String>) -> Voice {
        let voice_id: &String = voice_id.into();
        let req = self.requester.request_async(
            format!("https://neo.character.ai/multimodal/api/v1/voices/{}", voice_id),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
    
        match req {
            Ok(json) => Voice::from_json(&json.get("voice").unwrap_or(&json!({}))),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn search_voices(&self, voice_name: impl Into<&String>) -> Vec<Voice> {
        let voice_name: &String = voice_name.into();
        let query = urlencoding::encode(voice_name);
        let req = self.requester.request_async(
            format!("https://neo.character.ai/multimodal/api/v1/voices/search?query={}", query),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await;
    
        match req {
            Ok(json) => json.get("voices").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|v| Voice::from_json(v)).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }
    
    pub async fn generate_image(&self, prompt: &str, num_candidates: Option<u8>) -> Vec<String> {
        let num_candidates = num_candidates.unwrap_or(4);
    
        let req = self.requester.request_async(
            "https://plus.character.ai/chat/character/generate-avatar-options",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "prompt": prompt,
                "num_candidates": num_candidates,
                "model_version": "v1"
            }).to_string().into())),
        ).await;
    
        match req {
            Ok(json) => json.get("result").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().filter_map(|img| img["url"].as_str().map(String::from)).collect(),
            Err(data) => panic!("{:?}", data)
        }
    }

    pub async fn upload_avatar(&self, data: Vec<u8>, mime_type: String, check_image: bool) -> Avatar {
        let req = self.requester.request_async(
            "https://character.ai/api/trpc/user.uploadAvatar?batch=1",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "0": {
                    "json": {
                        "imageDataUrl": format!("data:{};base64,{}", mime_type, general_purpose::STANDARD.encode(&data))
                    }
                }
            }).to_string().into()))
        ).await;
    
        match req {
            Ok(json) => {
                let response = &json[0];
                if let Some(file_name) = response.get("result").and_then(|r| r.get("data")).and_then(|d| d.get("json")).and_then(|j| j.as_str()) {
                    let avatar = Avatar::new(file_name.to_string());
    
                    if check_image {
                        let image_req = self.requester.request_async(
                            avatar.get_default_url(),
                            RequestOptions::new("GET", self.client.get_headers(None).await, None)
                        ).await;
    
                        match image_req {
                            Ok(_) => avatar,
                            Err(err) => panic!("Cannot upload avatar. {}", err),
                        }
                    } else {
                        avatar
                    }
                } else {
                    panic!("Cannot upload avatar. Invalid response.");
                }
            },
            Err(err) => panic!("Cannot upload avatar. {}", err),
        }
    }

    pub async fn upload_voice(&self, data: Vec<u8>, mime_type: String, name: impl Into<String>, description: Option<String>, visibility: Option<Visibility>) -> Voice {
        let name = name.into();
        let description = description.unwrap_or_else(|| "".to_string());
        let visibility = visibility.unwrap_or_else(|| Visibility::Hidden);
    
        if name.len() < 3 || name.len() > 20 {
            panic!("mame cannot be less than 3 characters or more than 20 characters (is {} characters)", name.len());
        }
    
        if description.len() > 120 {
            panic!("description cannot be more than 120 characters (is {} characters)", description.len());
        }
    
        if visibility == Visibility::Unlisted {
            panic!("visibility cannot be Unlisted");
        }
    
        let boundary = format!("---------------------------{}", rand::rng().random::<u128>());
    
        let mut body = Vec::new();
        body.extend(format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"input.mp3\"\r\nContent-Type: {}\r\n\r\n",
            boundary, mime_type
        ).as_bytes());
        body.extend(&data);
        body.extend(format!(
            "\r\n--{}\r\nContent-Disposition: form-data; name=\"json\"\r\n\r\n{}\r\n--{}--\r\n",
            boundary,
            json!({
                "voice": {
                    "name": name,
                    "description": description,
                    "gender": "neutral",
                    "visibility": visibility.to_string(),
                    "previewText": "Good day! Here to make life a little less complicated.",
                    "audioSourceType": "file"
                }
            }).to_string(),
            boundary
        ).as_bytes());
    
        let headers = {
            let mut h = self.client.get_headers(None).await;
            h.insert("Content-Type".to_string(), format!("multipart/form-data; boundary={}", boundary));
            h
        };
    
        let req = self.requester.request_async(
            "https://neo.character.ai/multimodal/api/v1/voices/",
            RequestOptions::new("POST", headers, Some(body.into()))
        ).await;
    
        match req {
            Ok(json) => {
                if let Some(voice_data) = json.get("voice") {
                    let new_voice = Voice::from_json(voice_data);
                    self.edit_voice(new_voice.clone(), Some(name), Some(description), Some(visibility)).await
                } else {
                    panic!("Cannot upload voice. Invalid response.");
                }
            },
            Err(err) => panic!("Cannot upload voice. {}", err),
        }
    }

    pub async fn edit_voice(&self, voice: impl Into<VoiceOrId>, name: Option<String>, description: Option<String>, visibility: Option<Visibility>) -> Voice {
        let voice = match voice.into() {
            VoiceOrId::Id(id) => self.fetch_voice(&id).await,
            VoiceOrId::Voice(v) => v,
        };
    
        let name = name.unwrap_or_else(|| voice.name.clone());
        let description = description.unwrap_or_else(|| voice.description.clone());
        let visibility = visibility.unwrap_or_else(|| voice.visibility.clone());
    
        if name.is_empty() || description.is_empty() {
            panic!("mame and description must be specified.");
        }
        if name.len() < 3 || name.len() > 20 {
            panic!("mame cannot be less than 3 characters or more than 20 characters (is {} characters)", name.len());
        }
        if description.len() > 120 {
            panic!("Cannot edit voice. Description must be no more than 120 characters.");
        }
    
        if visibility == Visibility::Unlisted {
            panic!("Cannot edit voice. Visibility must be 'public' or 'private'.");
        }
    
        let req = self.requester.request_async(
            format!("https://neo.character.ai/multimodal/api/v1/voices/{}", voice.id),
            RequestOptions::new("PUT", self.client.get_headers(None).await, Some(json!({
                "voice": {
                    "audioSourceType": "file",
                    "backendId": voice.id,
                    "backendProvider": "cai",
                    "creatorInfo": {
                        "id": voice.creator_id,
                        "source": "user",
                        "username": "",
                    },
                    "description": description,
                    "gender": voice.gender.to_string(),
                    "id": voice.id,
                    "internalStatus": "draft",
                    "lastUpdateTime": "0001-01-01T00:00:00Z",
                    "name": name,
                    "previewAudioURI": voice.preview_audio_uri,
                    "previewText": voice.preview_text,
                    "visibility": visibility.to_string(),
                }
            }).to_string().into()))
        ).await;
    
        match req {
            Ok(json) => Voice::from_json(&json.get("voice").unwrap_or(&json!({}))),
            Err(data) => panic!("{:?}", data),
        }
    }

    pub async fn delete_voice(&self, voice_id: impl Into<&String>) -> bool {
        let voice_id: &String = voice_id.into();
        let req = self.requester.request_async(
            format!("https://neo.character.ai/multimodal/api/v1/voices/{}", voice_id),
            RequestOptions::new("DELETE", self.client.get_headers(None).await, None)
        ).await;
    
        match req {
            Ok(_) => true,
            Err(data) => panic!("{:?}", data),
        }
    }

    pub async fn generate_speech(&self, chat_id: impl Into<String>, turn_id: impl Into<String>, candidate_id: impl Into<String>, voice_id: impl Into<String>, return_url: bool) -> Result<Vec<u8>, String> {
        let req = self.requester.request_async(
            "https://neo.character.ai/multimodal/api/v1/memo/replay",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "candidateId": candidate_id.into(),
                "roomId": chat_id.into(),
                "turnId": turn_id.into(),
                "voiceId": voice_id.into(),
            }).to_string().into()))
        ).await;
    
        match req {
            Ok(json) => {
                if let Some(audio_url) = json.get("replayUrl").and_then(|v| v.as_str()) {
                    if return_url {
                        return Err(audio_url.to_string()); // return the url as an "error" if set to return the url
                    }
    
                    let audio_req = self.requester.request_resp_async(
                        audio_url.to_string(),
                        RequestOptions::new("GET", self.client.get_headers(None).await, None)
                    ).await;
    
                    match audio_req {
                        Ok(audio_data) => match audio_data.bytes().await {
                            Ok(bytes) => Ok(bytes.to_vec()),
                            Err(data) => panic!("{:?}", data)
                        },
                        Err(data) => panic!("{:?}", data),
                    }
                } else {
                    let err = json.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("");
                    panic!("could not generate speech: {}", err);
                }
            },
            Err(data) => panic!("{:?}", data),
        }
    }

    pub async fn ping(&self) -> bool {
        let req = self.requester.request_resp_async(
            "https://neo.character.ai/ping/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None),
        ).await;

        match req {
            Ok(resp) => resp.status() == 200,
            _ => false,
        }
    }
}