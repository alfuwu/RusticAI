use reqwest::{Client, Response as ReqwestResponse, Body};
use std::{sync::Arc, collections::HashMap};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{connect_async, tungstenite::{protocol::Message, ClientRequestBuilder}, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use http::Uri;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequesterError {
    #[error("Request failed")]
    RequestFailed,
    #[error("Authentication failed")]
    AuthenticationError,
    #[error("WebSocket connection error")]
    WsError,
}

pub struct RequestOptions {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Body>,
}

impl RequestOptions {
    pub fn new(method: impl Into<String>, headers: HashMap<String, String>, body: Option<Body>) -> Self {
        Self { method: method.into(), headers, body }
    }
}

#[derive(Debug, Clone)]
pub struct Requester {
    client: Client,
    //impersonate: Option<String>,
    //proxy: Option<String>,
    ws_client: Arc<RwLock<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
}

impl Requester {
    pub fn new(/*impersonate: Option<String>, proxy: Option<String>*/) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to build client");

        Self {
            client,
            //impersonate,
            //proxy,
            ws_client: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn request_resp_async(
        &self,
        url: impl Into<String>,
        options: RequestOptions,
    ) -> Result<ReqwestResponse, RequesterError> {
        let url = url.into();
        let mut req = match options.method.as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "PATCH" => self.client.patch(url),
            "DELETE" => self.client.delete(url),
            _ => return Err(RequesterError::RequestFailed),
        };

        for (key, value) in options.headers.iter() {
            req = req.header(key, value);
        }

        if let Some(body) = options.body {
            req = req.body(body);
        }

        let res: ReqwestResponse = req.send().await.map_err(|_| RequesterError::RequestFailed)?;

        if res.status() == 401 {
            return Err(RequesterError::AuthenticationError);
        }
        
        Ok(res)
    }

    pub async fn request_async(
        &self,
        url: impl Into<String>,
        options: RequestOptions,
    ) -> Result<Value, RequesterError> {
        let text = self.request_resp_async(url, options).await?.text().await.map_err(|_| RequesterError::RequestFailed)?;
        let json: Value = serde_json::from_str(&text).map_err(|_| RequesterError::RequestFailed)?;
        Ok(json)
    }

    pub async fn ws_connect(&self, token: impl Into<String>) -> Result<(), RequesterError> {
        let uri: Uri = "ws://localhost:3012/socket".parse().unwrap();
        let builder = ClientRequestBuilder::new(uri)
            .with_header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .with_header("Cookie", format!("HTTP_AUTHORIZATION=\"Token {}\"", token.into()));
        let (ws_stream, _) = connect_async(builder)
            .await
            .map_err(|_| RequesterError::WsError)?;

        let mut guard = self.ws_client.write().await;
        *guard = Some(ws_stream);

        // Optionally send headers as part of initial message (not supported in tungstenite natively)
        // Consider using a custom client for that if needed

        Ok(())
    }

    pub async fn ws_close(&self) {
        let mut guard = self.ws_client.write().await;
        if let Some(mut ws) = guard.take() {
            let _ = ws.close(None).await;
        }
    }

    pub async fn ws_send(&self, message: &Value) -> Result<(), RequesterError> {
        let mut guard = self.ws_client.write().await;

        if let Some(ws) = guard.as_mut() {
            let text = serde_json::to_string(message).unwrap();
            ws.send(Message::Text(text.into())).await.map_err(|_| RequesterError::WsError)?;
            Ok(())
        } else {
            Err(RequesterError::WsError)
        }
    }

    pub async fn ws_receive(&self) -> Result<Value, RequesterError> {
        let mut guard = self.ws_client.write().await;

        if let Some(ws) = guard.as_mut() {
            if let Some(msg) = ws.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let val: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                        Ok(val)
                    }
                    Ok(Message::Close(_)) => Err(RequesterError::WsError),
                    _ => Err(RequesterError::WsError),
                }
            } else {
                Err(RequesterError::WsError)
            }
        } else {
            Err(RequesterError::WsError)
        }
    }
}