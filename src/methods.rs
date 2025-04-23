use std::{sync::Arc, collections::HashMap};
use tokio::pin;
use futures_util::{Stream, StreamExt, stream::once};
use async_stream::stream;
use rand::Rng;
use base64::{Engine, engine::general_purpose};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{client::AsyncClient, requester::*, types::{character::*, chat::*, enums::Visibility, media::*, user::*}};

#[derive(Clone)]
pub struct AccountMethods {
    requester: Arc<Requester>,
    client: Arc<AsyncClient>
}

impl AccountMethods {
    pub fn new(requester: Arc<Requester>, client: Arc<AsyncClient>) -> Self {
        Self { requester, client }
    }

    pub async fn fetch_profile(&self) -> Result<Account, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/user/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;

        const ERR: &str = "Something went wrong with the Character.AI API; fetch_profile response was missing user object";
        Ok(Account::from_json(json.get("user").expect(ERR).get("user").expect(ERR)))
    }

    pub async fn fetch_settings(&self) -> Result<Settings, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/user/settings/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;

        Ok(Settings::from_json(&json))
    }

    pub async fn fetch_followers(&self) -> Result<Vec<String>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/user/followers/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
        
        Ok(json.get("followers").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| v.as_str().map(String::from)).collect())
    }

    pub async fn fetch_following(&self) -> Result<Vec<String>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/user/following/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
        
        Ok(json.get("following").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| v.as_str().map(String::from)).collect())
    }

    pub async fn fetch_persona(&self, id: impl Into<&String>) -> Result<Persona, RequesterError> {
        let json: Value = self.requester.request_async(
            format!("https://plus.character.ai/chat/persona/?id={}", id.into()),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
        
        Ok(Persona::from_json(&json))
    }

    pub async fn fetch_personas(&self) -> Result<Vec<Persona>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/personas/?force_refresh=1",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
        
        Ok(json.get("personas").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(Persona::from_json(&v))).collect())
    }

    pub async fn fetch_characters(&self) -> Result<Vec<PartialCharacter>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/chracters/?scope=user",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;

        Ok(json.get("characters").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(PartialCharacter::from_json(&v))).collect())
    }

    pub async fn fetch_characters_ranked(&self) -> Result<Vec<PartialCharacter>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/user/characters/upvoted/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
        
        Ok(json.get("characters").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(PartialCharacter::from_json(&v))).collect())
    }

    pub async fn fetch_voices(&self) -> Result<Vec<Voice>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/multimodal/api/v1/voices/user",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
        
        Ok(json.get("voices").unwrap_or(&json!([])).as_array().unwrap().into_iter().filter_map(|v| Some(Voice::from_json(&v))).collect())
    }

    async fn update_settings(&self, default_persona_id: Option<&String>, persona_override: Option<&String>, voice_override: Option<&String>, character_id: Option<&String>, settings: Option<&mut Settings>) -> Result<Settings, RequesterError> {
        if default_persona_id.is_none() && persona_override.is_none() && voice_override.is_none() {
            panic!("You must provide an updated value when calling update_settings");
        }
        let settings: &mut Settings = if let Some(real) = settings { real } else { &mut self.fetch_settings().await? };
        if let Some(dpi) = default_persona_id {
            settings.default_persona_id = dpi.to_string();
        }
        if let Some(ci) = character_id {
            if let Some(po) = persona_override {
                settings.persona_overrides.insert(ci.to_string(), po.to_string());
            }
        }
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/user/update_settings/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(settings.to_json().to_string().into()))
        ).await?;

        if json.get("success").unwrap_or(&json!(false)).as_bool().unwrap() {
            Err(RequesterError::RequestFailed(format!("cargo:warn=Malformed data received.\n{:?}", json)))
        } else {
            Ok(Settings::from_json(&json))
        }
    }

    pub async fn edit_account(&self, name: impl Into<&String>, username: impl Into<&String>, bio: Option<&String>, avatar_path: Option<&String>) -> Result<bool, RequesterError> {
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

        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/user/update/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(inf.to_string().into()))
        ).await?;

        Ok(json.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") != "OK")
    }

    pub async fn create_persona(&self, name: impl Into<&String>, definition: Option<String>, avatar_path: Option<String>) -> Result<Persona, RequesterError> {
        let name: &String = name.into();
        let definition: String = definition.unwrap_or_else(|| "".to_string());
        let avatar_path: String = avatar_path.unwrap_or_else(|| "".to_string());

        if name.len() < 3 || name.len() > 20 {
            panic!("name cannot be less than 3 characters or more than 20 (is {} characters)", name.len());
        }
        if definition.len() > 720 {
            panic!("definition cannot be more than 720 characters (is {} characters)", definition.len());
        }

        let json: Value = self.requester.request_async(
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
        ).await?;

        if json.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") != "OK" || json.get("persona").unwrap_or(&Value::Null) == &Value::Null {
            Err(RequesterError::RequestFailed(format!("{}", json.get("error").unwrap_or(&json!("")))))
        } else {
            Ok(Persona::from_json(&json.get("persona").unwrap_or(&json!({}))))
        }
    }

    async fn update_persona_internal(&self, json: Value) -> Result<Persona, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/persona/update/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json.to_string().into()))
        ).await?;

        if json.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") != "OK" || json.get("persona").unwrap_or(&Value::Null) == &Value::Null {
            Err(RequesterError::RequestFailed(format!("{}", json.get("error").unwrap_or(&json!("")))))
        } else {
            Ok(Persona::from_json(&json.get("persona").unwrap_or(&json!({}))))
        }
    }

    pub async fn edit_persona(&self, id: impl Into<&String>, name: Option<&String>, definition: Option<&String>, avatar_path: Option<&String>, persona: Option<&Persona>) -> Result<Persona, RequesterError> {
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

    pub async fn delete_persona(&self, id: impl Into<&String>, persona: Option<&Persona>) -> Result<Persona, RequesterError> {
        let id: &String = id.into();
        let persona: &Persona = if let Some(v) = persona { v } else { &self.fetch_persona(id).await? };
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
        self.update_settings(Some(id), None, None, None, settings).await.map_or(false, |_| true)
    }

    pub async fn set_persona(&self, character_id: impl Into<&String>, persona_id: Option<&String>, settings: Option<&mut Settings>) -> bool {
        let persona_id: &String = if let Some(v) = persona_id { v } else { &"".to_string() };
        self.update_settings(None, Some(persona_id), None, Some(character_id.into()), settings).await.map_or(false, |_| true)
    }

    pub async fn set_voice(&self, voice_id: Option<&String>, settings: Option<&mut Settings>) -> bool {
        let voice_id: &String = if let Some(v) = voice_id { v } else { &"".to_string() };
        self.update_settings(None, None, Some(voice_id), None, settings).await.map_or(false, |_| true)
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

    pub async fn fetch_user(&self, username: impl Into<&String>) -> Result<User, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/user/public/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({ "username": username.into() }).to_string().into()))
        ).await?;
    
        Ok(User::from_json(json.get("public_user").unwrap_or(&json!({}))))
    }
    
    pub async fn fetch_user_voices(&self, username: impl Into<&String>) -> Result<Vec<Voice>, RequesterError> {
        let json: Value = self.requester.request_async(
            format!("https://neo.character.ai/multimodal/api/v1/voices/search?creatorInfo.username={}", username.into()),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
        
        Ok(json.get("voices").and_then(|voices| voices.as_array()).unwrap_or(&vec![]).iter().map(|v| Voice::from_json(v)).collect())
    }
    
    pub async fn follow_user(&self, username: impl Into<&String>) -> bool {
        let resp = self.requester.request_async(
            "https://plus.character.ai/chat/user/follow/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({ "username": username.into() }).to_string().into()))
        ).await;
    
        resp.map_or(false, |v| v.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") == "OK")
    }
    
    pub async fn unfollow_user(&self, username: impl Into<&String>, token: Option<String>) -> bool {
        let resp = self.requester.request_async(
            "https://plus.character.ai/chat/user/unfollow/",
            RequestOptions::new("POST", self.client.get_headers(token).await, Some(json!({ "username": username.into() }).to_string().into()))
        ).await;
    
        resp.map_or(false, |v| v.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") == "OK")
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

    pub async fn fetch_histories(&self, character_id: impl Into<&String>, amount: usize) -> Result<Vec<ChatHistory>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/character/histories/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "external_id": character_id.into(),
                "number": amount
            }).to_string().into()))
        ).await?;
    
        Ok(json.get("histories").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|h| ChatHistory::from_json(h)).collect())
    }

    pub async fn fetch_chats(&self, character_id: impl Into<&String>, num_preview_turns: usize) -> Result<Vec<Chat>, RequesterError> {
        let json: Value = self.requester.request_async(
            format!("https://neo.character.ai/chats/?character_ids={}&num_preview_turns={}", character_id.into(), num_preview_turns),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(json.get("chats").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| Chat::from_json(c)).collect())
    }

    pub async fn fetch_chat(&self, chat_id: impl Into<&String>) -> Result<Chat, RequesterError> {
        let json: Value = self.requester.request_async(
            format!("https://neo.character.ai/chat/{}/", chat_id.into()),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(Chat::from_json(json.get("chat").unwrap_or(&json!({}))))
    }
    
    pub async fn fetch_recent_chats(&self) -> Result<Vec<Chat>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://neo.character.ai/chats/recent/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(json.get("chats").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| Chat::from_json(c)).collect())
    }

    pub async fn fetch_messages(&self, chat_id: impl Into<&String>, pinned_only: bool, next_token: Option<String>) -> Result<(Vec<Turn>, Option<String>), RequesterError> {
        let mut url = format!("https://neo.character.ai/turns/{}/", chat_id.into());
    
        if let Some(token) = &next_token {
            url = format!("{}?next_token={}", url, urlencoding::encode(token));
        }
    
        let json: Value = self.requester.request_async(
            url,
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
        
        let next_token = json.get("meta")
            .and_then(|meta| meta.get("next_token"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let turns = json.get("turns")
            .unwrap_or(&json!([]))
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter(|t| {
                if !pinned_only {
                    true
                } else {
                    t.get("is_pinned").and_then(|p| p.as_bool()).unwrap_or(false)
                }
            })
            .map(|t| Turn::from_json(t))
            .collect();
    
        Ok((turns, next_token))
    }
    
    pub async fn fetch_all_messages(&self, chat_id: impl Into<&String>, pinned_only: bool) -> Result<Vec<Turn>, RequesterError> {
        let chat_id = chat_id.into();
        let mut all_turns = Vec::new();
        let mut next_token = None;
    
        loop {
            let (turns, token) = self.fetch_messages(chat_id, pinned_only, next_token).await?;
            if turns.is_empty() {
                break;
            }
            all_turns.extend(turns);
            if token.is_none() {
                break;
            }
            next_token = token;
        }
    
        Ok(all_turns)
    }

    pub async fn fetch_following_messages(&self, chat_id: impl Into<&String>, turn_id: impl Into<&String>, pinned_only: bool) -> Result<Vec<Turn>, RequesterError> {
        let chat_id = chat_id.into();
        let target_turn_id = turn_id.into();
        let mut following_turns = Vec::new();
        let mut next_token = None;
    
        loop {
            let (turns, token) = self.fetch_messages(chat_id, pinned_only, next_token).await?;
            if turns.is_empty() {
                panic!("cannot fetch following messages");
            }
    
            for turn in &turns {
                if turn.id == *target_turn_id {
                    return Ok(following_turns);
                }
                following_turns.push(turn.clone());
            }
    
            if token.is_none() {
                panic!("Cannot fetch following messages. May be turn_id is invalid?");
            }
    
            next_token = token;
        }
    }

    pub async fn update_chat_name(&self, chat_id: impl Into<&String>, name: impl Into<&String>) -> bool {
        let resp = self.requester.request_resp_async(
            format!("https://neo.character.ai/chat/{}/update_name", chat_id.into()),
            RequestOptions::new("PATCH", self.client.get_headers(None).await, Some(json!({ "name": name.into() }).to_string().into()))
        ).await;
        
        resp.map_or(false, |_| true)
    }

    pub async fn archive_chat(&self, chat_id: impl Into<&String>) -> bool {
        let resp = self.requester.request_async(
            format!("https://neo.character.ai/chat/{}/archive", chat_id.into()),
            RequestOptions::new("PATCH", self.client.get_headers(None).await, Some("{}".to_string().into()))
        ).await;
        
        resp.map_or(false, |_| true)
    }
    
    pub async fn unarchive_chat(&self, chat_id: impl Into<&String>) -> bool {
        let resp = self.requester.request_async(
            format!("https://neo.character.ai/chat/{}/unarchive", chat_id.into()),
            RequestOptions::new("PATCH", self.client.get_headers(None).await, Some("{}".to_string().into()))
        ).await;
        
        resp.map_or(false, |_| true)
    }
    
    pub async fn copy_chat(&self, chat_id: impl Into<&String>, end_turn_id: impl Into<&String>) -> Result<Option<String>, RequesterError> {
        let json: Value = self.requester.request_async(
            format!("https://neo.character.ai/chat/{}/copy", chat_id.into()),
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({ "end_turn_id": end_turn_id.into() }).to_string().into()))
        ).await?;
    
        Ok(json.get("new_chat_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
    }
    
    pub async fn create_chat(&self, character_id: impl Into<&String>, greeting: bool, model_type: Option<String>) -> Result<(Chat, Option<Turn>), RequesterError> {
        let request_id = Uuid::new_v4().to_string();
        let chat_id = Uuid::new_v4().to_string();
        let character_id = character_id.into();
    
        let mut payload = json!({
            "chat": {
                "chat_id": chat_id,
                "creator_id": self.client.data().await.id,
                "visibility": "VISIBILITY_PRIVATE",
                "character_id": character_id,
                "type": "TYPE_ONE_ON_ONE",
            },
            "with_greeting": greeting,
        });
    
        if let Some(model) = model_type {
            payload["chat"]["preferred_model_type"] = json!(model);
        }
        
        let mut new_chat: Option<Chat> = None;
        let mut greeting_turn: Option<Turn> = None;
        
        let json = json!({
            "command": "create_chat",
            "request_id": request_id,
            "payload": payload,
        });
        let stream = self.requester.ws_send_and_receive(&json, self.client.token().await).await?;
        pin!(stream);
    
        while let Some(raw) = stream.next().await {
            let raw = raw.expect("WebSocket connection error");
    
            match raw.get("command").and_then(|v| v.as_str()) {
                Some("create_chat_response") => {
                    new_chat = Some(Chat::from_json(&raw["chat"]));
                    if !greeting {
                        break;
                    }
                },
                Some("add_turn") => {
                    greeting_turn = Some(Turn::from_json(&raw["turn"]));
                    break;
                },
                Some("neo_error") => {
                    let comment = raw.get("comment").and_then(|v| v.as_str()).unwrap_or("");
                    return Err(RequesterError::WsError(format!("cannot create a new chat: {}", comment)));
                },
                _ => {}
            }
        }
    
        if new_chat.is_none() || (greeting && greeting_turn.is_none()) {
            panic!("Cannot create a new chat.");
        }
    
        Ok((new_chat.unwrap(), greeting_turn))
    }
    
    pub async fn update_primary_candidate(&self, chat_id: impl Into<&String>, turn_id: impl Into<&String>, candidate_id: impl Into<&String>) -> bool {
        let json = json!({
            "command": "update_primary_candidate",
            "origin_id": "web-next",
            "payload": {
                "candidate_id": candidate_id.into(),
                "turn_key": { "chat_id": chat_id.into(), "turn_id": turn_id.into() },
            }
        });
        let resp = self.requester.ws_send_and_receive(&json, self.client.token().await).await;

        if let Ok(stream) = resp {
            pin!(stream);
            while let Some(raw) = stream.next().await {
                let raw = raw.expect("WebSocket connection error");
        
                match raw.get("command").and_then(|v| v.as_str()) {
                    Some("ok") => {
                        return true;
                    },
                    Some("neo_error") => {
                        return false;
                    },
                    _ => {}
                }
            }
        }
    
        false
    }
    
    pub async fn send_message_stream(&self, character_id: impl Into<&String>, chat_id: impl Into<&String>, text: impl Into<&String>) -> Result<impl Stream<Item = Turn>, RequesterError> {
        let candidate_id = Uuid::new_v4().to_string();
        let turn_id = Uuid::new_v4().to_string();
        let request_id = Uuid::new_v4().to_string();
        
        let ret_stream = stream! {
            let json = json!({
                "command": "create_and_generate_turn",
                "origin_id": "web-next",
                "payload": {
                    "character_id": character_id.into(),
                    "num_candidates": 1,
                    "previous_annotations": "self.default_annotations()",
                    "selected_language": "",
                    "tts_enabled": false,
                    "turn": {
                        "author": {
                            "author_id": self.client.data().await.id,
                            "is_human": true,
                            "name": "",
                        },
                        "candidates": [{
                            "candidate_id": candidate_id,
                            "raw_content": text.into(),
                        }],
                        "primary_candidate_id": candidate_id,
                        "turn_key": { "chat_id": chat_id.into(), "turn_id": turn_id },
                    },
                    "user_name": ""
                },
                "request_id": request_id,
            });
            let resp = self.requester.ws_send_and_receive(&json, self.client.token().await).await;
            if let Ok(stream) = resp {
                pin!(stream);

                while let Some(raw) = stream.next().await {
                    let raw = raw.expect("WebSocket connection error");
            
                    match raw.get("command").and_then(|v| v.as_str()) {
                        val if val == Some("add_turn") || val == Some("update_turn") => {
                            if raw["turn"]["author"]["is_human"].as_bool().unwrap_or(false) {
                                continue;
                            }
            
                            let turn = Turn::from_json(&raw["turn"]);
                            yield turn.clone();
            
                            if turn.get_primary_candidate().map_or(false, |c| c.is_final) {
                                break;
                            }
                        },
                        Some("neo_error") => {
                            let comment = raw["comment"].as_str().unwrap_or("");
                            panic!("cannot send message: {}", comment);
                        },
                        _ => {}
                    }
                }
            }
        };
    
        Ok(ret_stream)
    }
    

    pub async fn send_message(&self, character_id: impl Into<&String>, chat_id: impl Into<&String>, text: impl Into<&String>) -> Result<Turn, RequesterError> {
        let stream = self.send_message_stream(character_id, chat_id, text).await?;
        pin!(stream);

        let mut last_turn = Err(RequesterError::RequestFailed("could not find Turn".to_string()));

        while let Some(turn) = stream.next().await {
            last_turn = Ok(turn);
        }
        
        last_turn
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

    pub async fn fetch_characters_by_category(&self) -> Result<HashMap<String, Vec<PartialCharacter>>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/curated_categories/characters/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        let mut result = HashMap::new();
        for (k, v) in json.get("characters_by_curated_category").unwrap_or(&json!({})).as_object().unwrap_or(&serde_json::Map::new()) {
            let characters = v.as_array().unwrap_or(&vec![])
                .iter().map(|c| PartialCharacter::from_json(c)).collect();
            result.insert(k.to_string(), characters);
        }
        Ok(result)
    }
    
    pub async fn fetch_recommended_characters(&self) -> Result<Vec<PartialCharacter>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://neo.character.ai/recommendation/v1/user",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(json.get("characters").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| PartialCharacter::from_json(c)).collect())
    }
    
    pub async fn fetch_featured_characters(&self) -> Result<Vec<PartialCharacter>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/characters/featured_v2/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(json.get("characters").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| PartialCharacter::from_json(c)).collect())
    }

    pub async fn fetch_similar_characters(&self, character_id: impl Into<&String>) -> Result<Vec<PartialCharacter>, RequesterError> {
        let json: Value = self.requester.request_async(
            format!("https://neo.character.ai/recommendation/v1/character/{}", character_id.into()),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(json.get("characters").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| PartialCharacter::from_json(c)).collect())
    }
    
    pub async fn fetch_character_info(&self, character_id: impl Into<&String>) -> Result<Character, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/character/info/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({ "external_id": character_id.into() }).to_string().into()))
        ).await?;
    
        if json.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") != "OK" {
            panic!("{}", json.get("error").unwrap_or(&json!("")));
        }
        Ok(Character::from_json(json.get("character").unwrap_or(&json!({}))))
    }

    pub async fn search_characters(&self, character_name: impl Into<&String>) -> Result<Vec<PartialCharacter>, RequesterError> {
        let character_name = character_name.into();
        let payload = json!({
            "0": {
                "json": {
                    "searchQuery": urlencoding::encode(&character_name)
                }
            }
        });
        
        let json: Value = self.requester.request_async(
            format!("https://character.ai/api/trpc/search.search?batch=1&input={}", payload.to_string()),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(json[0]["result"]["data"]["json"]["characters"].as_array().unwrap_or(&vec![]).iter().map(|c| PartialCharacter::from_json(c)).collect())
    }
    
    pub async fn search_creators(&self, creator_name: impl Into<&String>) -> Result<Vec<String>, RequesterError> {
        let json: Value = self.requester.request_async(
            format!("https://plus.character.ai/chat/creators/search/?query={}", urlencoding::encode(&creator_name.into())),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(json.get("creators").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|c| c["name"].as_str().unwrap_or("").to_string()).collect())
    }

    pub async fn add_like_to_character(&self, character_id: impl Into<&String>, like: Option<bool>) -> bool {
        let resp = self.requester.request_async(
            "https://plus.character.ai/chat/character/vote/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "external_id": character_id.into(),
                "vote": like
            }).to_string().into()))
        ).await;
    
        resp.map_or(false, |v| v.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") == "OK")
    }

    pub async fn create_character(&self, name: impl Into<&String>, greeting: impl Into<&String>, title: impl Into<&String>, description: impl Into<&String>, definition: impl Into<&String>, copyable: bool, visibility: Visibility, avatar_rel_path: impl Into<&String>, default_voice_id: impl Into<&String>) -> Result<Character, RequesterError> {
        let name = name.into();
        let greeting = greeting.into();
        let title = title.into();
        let description = description.into();
        let definition = definition.into();
    
        if name.len() < 3 || name.len() > 20 {
            panic!("name cannot be less than 3 characters or more than 20 (is {} characters)", name.len());
        }
        if greeting.len() < 3 || greeting.len() > 2048 {
            panic!("greeting cannot be less than 3 characters or more than 2048 (is {} characters)", greeting.len());
        }
        if !title.is_empty() && (title.len() < 3 || title.len() > 50) {
            panic!("title cannot be less than 3 characters or more than 50 (is {} characters)", title.len());
        }
        if description.len() > 500 {
            panic!("description cannot be less than more than 500 (is {} characters)", description.len());
        }
        if definition.len() > 32000 {
            panic!("definition cannot be more than 32000 (is {} characters)", definition.len());
        }
        
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/character/create/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "avatar_rel_path": avatar_rel_path.into(),
                "base_img_prompt": "",
                "categories": [],
                "copyable": copyable,
                "default_voice_id": default_voice_id.into(),
                "definition": definition,
                "description": description,
                "greeting": greeting,
                "identifier": format!("id:{}", Uuid::new_v4()),
                "img_gen_enabled": false,
                "name": name,
                "strip_img_prompt_from_msg": false,
                "title": title,
                "visibility": visibility.to_string(),
                "voice_id": "",
            }).to_string().into()))
        ).await?;
    
        if json.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") == "OK" {
            Ok(Character::from_json(&json.get("character").unwrap_or(&json!({}))))
        } else {
            Err(RequesterError::RequestFailed("malformed JSON object received".to_string()))
        }
    }

    pub async fn edit_character(&self, character_id: impl Into<&String>, name: impl Into<&String>, greeting: impl Into<&String>, title: impl Into<&String>, description: impl Into<&String>, definition: impl Into<&String>, copyable: bool, visibility: Visibility, avatar_rel_path: impl Into<&String>, default_voice_id: impl Into<&String>) -> Result<Character, RequesterError> {
        let name = name.into();
        let greeting = greeting.into();
        let title = title.into();
        let description = description.into();
        let definition = definition.into();
    
        if name.len() < 3 || name.len() > 20 {
            panic!("name cannot be less than 3 characters or more than 20 (is {} characters)", name.len());
        }
        if greeting.len() < 3 || greeting.len() > 2048 {
            panic!("greeting cannot be less than 3 characters or more than 2048 (is {} characters)", greeting.len());
        }
        if !title.is_empty() && (title.len() < 3 || title.len() > 50) {
            panic!("title cannot be less than 3 characters or more than 50 (is {} characters)", title.len());
        }
        if description.len() > 500 {
            panic!("description cannot be less than more than 500 (is {} characters)", description.len());
        }
        if definition.len() > 32000 {
            panic!("definition cannot be more than 32000 (is {} characters)", definition.len());
        }
        
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/character/update/",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "archived": false,
                "avatar_rel_path": avatar_rel_path.into(),
                "base_img_prompt": "",
                "categories": [],
                "copyable": copyable,
                "default_voice_id": default_voice_id.into(),
                "definition": definition,
                "description": description,
                "external_id": character_id.into(),
                "greeting": greeting,
                "img_gen_enabled": false,
                "name": name,
                "strip_img_prompt_from_msg": false,
                "title": title,
                "visibility": visibility.to_string(),
                "voice_id": "",
            }).to_string().into()))
        ).await?;
    
        if json.get("status").unwrap_or(&json!("")).as_str().unwrap_or("") == "OK" {
            Ok(Character::from_json(&json.get("character").unwrap_or(&json!({}))))
        } else {
            Err(RequesterError::RequestFailed(format!("cannot edit character: {:?}", json.get("error").unwrap_or(&json!("")).to_string())))
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

    pub async fn fetch_voice(&self, voice_id: impl Into<&String>) -> Result<Voice, RequesterError> {
        let json: Value = self.requester.request_async(
            format!("https://neo.character.ai/multimodal/api/v1/voices/{}", voice_id.into()),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(Voice::from_json(&json.get("voice").unwrap_or(&json!({}))))
    }

    pub async fn search_voices(&self, voice_name: impl Into<&String>) -> Result<Vec<Voice>, RequesterError> {
        let json: Value = self.requester.request_async(
            format!("https://neo.character.ai/multimodal/api/v1/voices/search?query={}", urlencoding::encode(voice_name.into())),
            RequestOptions::new("GET", self.client.get_headers(None).await, None)
        ).await?;
    
        Ok(json.get("voices").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().map(|v| Voice::from_json(v)).collect())
    }
    
    pub async fn generate_image(&self, prompt: &str, num_candidates: Option<u8>) -> Result<Vec<String>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://plus.character.ai/chat/character/generate-avatar-options",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "prompt": prompt,
                "num_candidates": num_candidates.unwrap_or(4),
                "model_version": "v1"
            }).to_string().into())),
        ).await?;
    
        Ok(json.get("result").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).iter().filter_map(|img| img["url"].as_str().map(String::from)).collect())
    }

    pub async fn upload_avatar(&self, data: Vec<u8>, mime_type: String, check_image: bool) -> Result<Avatar, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://character.ai/api/trpc/user.uploadAvatar?batch=1",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "0": {
                    "json": {
                        "imageDataUrl": format!("data:{};base64,{}", mime_type, general_purpose::STANDARD.encode(&data))
                    }
                }
            }).to_string().into()))
        ).await?;
        
        let response = &json[0];
        if let Some(file_name) = response.get("result").and_then(|r| r.get("data")).and_then(|d| d.get("json")).and_then(|j| j.as_str()) {
            let avatar = Avatar::new(file_name.to_string());

            if check_image {
                let image_req = self.requester.request_async(
                    avatar.get_default_url(),
                    RequestOptions::new("GET", self.client.get_headers(None).await, None)
                ).await;

                match image_req {
                    Ok(_) => Ok(avatar),
                    Err(err) => panic!("Cannot upload avatar. {}", err),
                }
            } else {
                Ok(avatar)
            }
        } else {
            panic!("Cannot upload avatar. Invalid response.");
        }
    }

    pub async fn upload_voice(&self, data: Vec<u8>, mime_type: String, name: impl Into<&String>, description: Option<String>, visibility: Option<Visibility>) -> Result<Voice, RequesterError> {
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
    
        let json: Value = self.requester.request_async(
            "https://neo.character.ai/multimodal/api/v1/voices/",
            RequestOptions::new("POST", headers, Some(body.into()))
        ).await?;
    
        if let Some(voice_data) = json.get("voice") {
            let new_voice = Voice::from_json(voice_data);
            Ok(self.edit_voice(new_voice.clone(), Some(name.clone()), Some(description), Some(visibility)).await?)
        } else {
            panic!("Cannot upload voice. Invalid response.");
        }
    }

    pub async fn edit_voice(&self, voice: impl Into<VoiceOrId>, name: Option<String>, description: Option<String>, visibility: Option<Visibility>) -> Result<Voice, RequesterError> {
        let voice = match voice.into() {
            VoiceOrId::Id(id) => self.fetch_voice(&id).await?,
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
    
        let json: Value = self.requester.request_async(
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
        ).await?;
    
        Ok(Voice::from_json(&json.get("voice").unwrap_or(&json!({}))))
    }

    pub async fn delete_voice(&self, voice_id: impl Into<&String>) -> bool {
        let resp = self.requester.request_resp_async(
            format!("https://neo.character.ai/multimodal/api/v1/voices/{}", voice_id.into()),
            RequestOptions::new("DELETE", self.client.get_headers(None).await, None)
        ).await;
        
        resp.map_or(false, |_| true)
    }

    pub async fn generate_speech(&self, chat_id: impl Into<&String>, turn_id: impl Into<&String>, candidate_id: impl Into<&String>, voice_id: impl Into<&String>, return_url: bool) -> Result<Result<Vec<u8>, String>, RequesterError> {
        let json: Value = self.requester.request_async(
            "https://neo.character.ai/multimodal/api/v1/memo/replay",
            RequestOptions::new("POST", self.client.get_headers(None).await, Some(json!({
                "candidateId": candidate_id.into(),
                "roomId": chat_id.into(),
                "turnId": turn_id.into(),
                "voiceId": voice_id.into(),
            }).to_string().into()))
        ).await?;
    
        if let Some(audio_url) = json.get("replayUrl").and_then(|v| v.as_str()) {
            if return_url {
                return Ok(Err(audio_url.to_string())); // return the url as an "error" if set to return the url

                // there has GOT to be a better way to do this
            }

            let audio_data = self.requester.request_resp_async(
                audio_url.to_string(),
                RequestOptions::new("GET", self.client.get_headers(None).await, None)
            ).await?;

            match audio_data.bytes().await {
                Ok(bytes) => Ok(Ok(bytes.to_vec())),
                Err(data) => Err(RequesterError::RequestFailed(format!("could not retrieve bytes from generated voice url: {:?}", data)))
            }
        } else {
            panic!("could not generate speech: {}", json.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or(""));
        }
    }

    pub async fn ping(&self) -> bool {
        let resp = self.requester.request_resp_async(
            "https://neo.character.ai/ping/",
            RequestOptions::new("GET", self.client.get_headers(None).await, None),
        ).await;

        resp.map_or(false, |v| v.status() == 200)
    }
}