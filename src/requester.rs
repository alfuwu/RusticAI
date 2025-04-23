use futures_util::Stream;
use reqwest::{Client, Response as ReqwestResponse, Body};
use std::{sync::Arc, collections::HashMap};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{connect_async, tungstenite::{protocol::Message, ClientRequestBuilder}, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use async_stream::stream;
use http::Uri;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RequesterError {
    #[error("Request failed")]
    RequestFailed(String),
    #[error("Authentication failed")]
    AuthenticationError,
    #[error("WebSocket connection error")]
    WsError(String),
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
    ws_client: Arc<RwLock<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
}

impl Requester {
    pub fn new() -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to build client");

        Self {
            client,
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
            _ => return Err(RequesterError::RequestFailed("Invalid method. Please use one of GET, POST, PUT, PATCH, or DELETE.".to_string())),
        };

        for (key, value) in options.headers.iter() {
            req = req.header(key, value);
        }

        if let Some(body) = options.body {
            req = req.body(body);
        }

        let res: ReqwestResponse = req.send().await.map_err(|err| RequesterError::RequestFailed(err.to_string()))?;

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
        let text = self.request_resp_async(url, options).await?.text().await.map_err(|err| RequesterError::RequestFailed(err.to_string()))?;
        let json: Value = serde_json::from_str(&text).map_err(|err| RequesterError::RequestFailed(err.to_string()))?;
        Ok(json)
    }

    pub async fn ws_connect(&self, token: impl Into<String>) -> Result<(), RequesterError> {
        let uri: Uri = "wss://neo.character.ai/ws/".parse().unwrap();
        let builder = ClientRequestBuilder::new(uri)
            .with_header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .with_header("Cookie", format!("HTTP_AUTHORIZATION=\"Token {}\"", token.into()));
        let (ws_stream, _) = connect_async(builder)
            .await
            .map_err(|_| RequesterError::WsError("could not connect to websocket url wss://neo.character.ai/ws/".to_string()))?;

        let mut guard = self.ws_client.write().await;
        *guard = Some(ws_stream);

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
            ws.send(Message::Text(text.into())).await.map_err(|_| RequesterError::WsError("websocket.send failed".to_string()))?;
            Ok(())
        } else {
            Err(RequesterError::WsError("could not obtain websocket".to_string()))
        }
    }

    pub async fn ws_receive(&self) -> impl Stream<Item = Result<Value, RequesterError>> {
        let ws_client = Arc::clone(&self.ws_client);

        stream! {
            let mut guard = ws_client.write().await;
            if let Some(ws) = guard.as_mut() {
                let (_, mut read) = ws.split();

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            match serde_json::from_str::<Value>(&text) {
                                Ok(val) => {
                                    yield Ok(val);
                                }
                                Err(_) => {
                                    yield Err(RequesterError::WsError("invalid JSON received".to_string()));
                                }
                            }
                        }
                        Ok(_) => continue,
                        Err(e) => {
                            yield Err(RequesterError::WsError(format!("WebSocket read error: {}", e)));
                            break;
                        }
                    }
                }
            }
        }
    }

    pub async fn ws_send_and_receive(&self, message: &Value, token: String) -> Result<impl Stream<Item = Result<Value, RequesterError>>, RequesterError> {
        if self.ws_client.read().await.is_none() {
            self.ws_connect(token).await?
        }
        self.ws_send(message).await?;
        Ok(self.ws_receive().await)
    }
}