use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures_util::stream::StreamExt;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::models::WsMessage;

pub struct Broadcaster {
    clients: Arc<RwLock<HashMap<String, actix_ws::Session>>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn broadcast(&self, msg: WsMessage) {
        let clients = self.clients.read();
        let json = serde_json::to_string(&msg).unwrap();
        for session in clients.values() {
            let mut session = session.clone();
            let json = json.clone();
            actix_web::rt::spawn(async move {
                if let Err(err) = session.text(json).await {
                    log::error!("Error broadcasting message: {}", err);
                }
            });
        }
    }
}

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, Error> {
    let (response, session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    let client_id = Uuid::new_v4().to_string();

    broadcaster
        .clients
        .write()
        .insert(client_id.clone(), session.clone());

    // Handle incoming messages
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    log::info!("Received message: {}", text);
                }
                Message::Close(reason) => {
                    log::info!("Client disconnected: {:?}", reason);
                    break;
                }
                _ => {}
            }
        }

        broadcaster.clients.write().remove(&client_id);
    });

    Ok(response)
}
