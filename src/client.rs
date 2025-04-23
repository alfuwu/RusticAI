use std::{sync::Arc, collections::HashMap};
use tokio::sync::RwLock;

use crate::{methods::*, requester::*, types::user::Account};

pub struct AsyncClient {
    token: RwLock<Option<String>>,
    requester: Arc<Requester>,

    pub data: RwLock<Account>,

    pub account: RwLock<Option<AccountMethods>>,
    pub user: RwLock<Option<UserMethods>>,
    pub chat: RwLock<Option<ChatMethods>>,
    pub character: RwLock<Option<CharacterMethods>>,
    pub utils: RwLock<Option<UtilsMethods>>,
}

impl AsyncClient {
    pub async fn new(token: Option<String>) -> Arc<Self> {
        let requester = Arc::new(Requester::new());
        
        let arc = Arc::new(Self {
            token: RwLock::new(token),
            requester: requester.clone(),
            
            data: RwLock::new(Account::default()),

            account: RwLock::new(None),
            user: RwLock::new(None),
            chat: RwLock::new(None),
            character: RwLock::new(None),
            utils: RwLock::new(None),
        });

        let header_provider = arc.clone();

        {
            *arc.account.write().await = Some(AccountMethods::new(requester.clone(), header_provider.clone()));
            *arc.user.write().await = Some(UserMethods::new(requester.clone(), header_provider.clone()));
            *arc.chat.write().await = Some(ChatMethods::new(requester.clone(), header_provider.clone()));
            *arc.character.write().await = Some(CharacterMethods::new(requester.clone(), header_provider.clone()));
            *arc.utils.write().await = Some(UtilsMethods::new(requester.clone(), header_provider.clone()));
            
            let res = arc.account().await.fetch_profile().await;
            if let Ok(account) = res {
                arc.set_data(account).await;
            }
        }

        arc
    }

    pub async fn token(&self) -> String {
        self.token.read().await.as_ref().unwrap().clone()
    }

    pub async fn data(&self) -> Account {
        self.data.read().await.as_ref().clone()
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

    pub async fn set_token(&self, token: impl Into<String>) {
        *self.token.write().await = Some(token.into());
    }

    pub async fn set_data(&self, account: impl Into<Account>) {
        *self.data.write().await = account.into();
    }

    pub async fn get_headers(&self, token: Option<String>) -> HashMap<String, String> {
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

    pub fn get_requester(&self) -> Arc<Requester> {
        self.requester.clone()
    }

    pub async fn close_session(&self) {
        self.requester.ws_close().await;
    }
}

#[tokio::main]
pub async fn main() {
    let d = AsyncClient::new(Some("9912bd86846346c083cf48cd88f7300a114cf5c7".to_string())).await;
    let da = d.account().await;
    let account: Account = da.fetch_profile().await.expect("fuck");
    assert!(account == d.data().await);
    println!("\x1b[1m\x1b[32mHEADERS\x1b[0m: {:?}", d.get_headers(None).await);
    println!("\x1b[1m\x1b[32mPROFILE\x1b[0m: {:?}", account);
    println!("\x1b[1m\x1b[32mSETTINGS\x1b[0m: {:?}", da.fetch_settings().await.expect("fuck"));
    println!("\x1b[1m\x1b[32mFOLLOWERS\x1b[0m: {:?}", da.fetch_followers().await.expect("fuck"));
    println!("\x1b[1m\x1b[32mFOLLOWING\x1b[0m: {:?}", da.fetch_following().await.expect("fuck"));
    println!("\x1b[1m\x1b[32mPERSONAS\x1b[0m: {:?}", da.fetch_personas().await.expect("fuck")[0]);
    println!("\x1b[1m\x1b[32mCHARS\x1b[0m: {:?}", da.fetch_characters_ranked().await.expect("fuck")[0]);
    println!("\x1b[1m\x1b[32mEDIT ACC SUCCEEDED\x1b[0m: {}", da.edit_account(&account.name, &account.username, Some(&account.bio), if let Some(avi) = &account.avatar { Some(&avi.file_name) } else { None }).await.expect("fuck"));
}