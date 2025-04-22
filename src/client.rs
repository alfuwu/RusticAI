use std::{sync::Arc, collections::HashMap};
use tokio::sync::RwLock;

use crate::{methods::*, requester::*};

pub struct AsyncClient {
    token: RwLock<Option<String>>,
    requester: Arc<Requester>,

    pub account: RwLock<Option<AccountMethods>>,
    pub user: RwLock<Option<UserMethods>>,
    pub chat: RwLock<Option<ChatMethods>>,
    pub character: RwLock<Option<CharacterMethods>>,
    pub utils: RwLock<Option<UtilsMethods>>,
}

#[async_trait::async_trait]
pub trait HeaderProvider: Send + Sync {
    async fn get_headers(&self, token: Option<String>) -> HashMap<String, String>;
}

#[async_trait::async_trait]
impl HeaderProvider for AsyncClient {
    async fn get_headers(&self, token: Option<String>) -> HashMap<String, String> {
        let tokey_tokey: &String;
        let read = self.token.read().await;
        if token.is_none() {
            if read.is_none() {
                panic!("No token found.");
            }
            tokey_tokey = read.as_ref().unwrap();
        } else {
            tokey_tokey = token.as_ref().unwrap();
        }

        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), format!("Token {}", tokey_tokey));
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        headers
    }
}

impl AsyncClient {
    pub async fn new(token: Option<String>) -> Arc<Self> {
        let requester = Arc::new(Requester::new());

        let client = Arc::new(AsyncClient {
            token: RwLock::new(token),
            requester: requester.clone(),

            account: RwLock::new(None),
            user: RwLock::new(None),
            chat: RwLock::new(None),
            character: RwLock::new(None),
            utils: RwLock::new(None),
        });

        let header_provider = client.clone() as Arc<dyn HeaderProvider>;

        {
            *client.account.write().await = Some(AccountMethods::new(requester.clone(), header_provider.clone()));
            *client.user.write().await = Some(UserMethods::new(requester.clone(), header_provider.clone()));
            *client.chat.write().await = Some(ChatMethods::new(requester.clone(), header_provider.clone()));
            *client.character.write().await = Some(CharacterMethods::new(requester.clone(), header_provider.clone()));
            *client.utils.write().await = Some(UtilsMethods::new(requester.clone(), header_provider.clone()));
        }

        client
    }

    pub async fn account(&self) -> Arc<AccountMethods> {
        Arc::new(self.account.read().await.as_ref().unwrap().clone())
    }

    pub async fn user(&self) -> Arc<UserMethods> {
        Arc::new(self.user.read().await.as_ref().unwrap().clone())
    }

    pub async fn chat(&self) -> Arc<ChatMethods> {
        Arc::new(self.chat.read().await.as_ref().unwrap().clone())
    }

    pub async fn character(&self) -> Arc<CharacterMethods> {
        Arc::new(self.character.read().await.as_ref().unwrap().clone())
    }

    pub async fn utils(&self) -> Arc<UtilsMethods> {
        Arc::new(self.utils.read().await.as_ref().unwrap().clone())
    }

    //pub async fn set_token(&mut self, token: impl Into<String>) {
    //    *self.token.write().await = Some(token.into());
    //}

    pub fn get_requester(&self) -> Arc<Requester> {
        self.requester.clone()
    }

    pub async fn close_session(&self) {
        self.requester.ws_close().await;
    }
}

#[tokio::main]
pub async fn main() {
    let d = AsyncClient::new(Some("TOKEN_HERE".to_string())).await;
    println!("\x1b[1m\x1b[32mHEADERS\x1b[0m: {:?}", d.get_headers(None).await);
    println!("\x1b[1m\x1b[32mPROFILE\x1b[0m: {:?}", d.account().await.fetch_profile().await);
    println!("\x1b[1m\x1b[32mSETTINGS\x1b[0m: {:?}", d.account().await.fetch_settings().await);
    println!("\x1b[1m\x1b[32mFOLLOWERS\x1b[0m: {:?}", d.account().await.fetch_followers().await);
    println!("\x1b[1m\x1b[32mFOLLOWING\x1b[0m: {:?}", d.account().await.fetch_following().await);
    println!("\x1b[1m\x1b[32mPERSONAS\x1b[0m: {:?}", d.account().await.fetch_personas().await);
    println!("\x1b[1m\x1b[32mCHARS\x1b[0m: {:?}", d.account().await.fetch_characters_ranked().await);
}