use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

#[derive(Clone)]
struct User {
    id: usize,
    nick: String,
}

type Users = Arc<Mutex<Vec<User>>>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientMessage {
    message_type: MessageType,
    data: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum MessageType {
    Users,
    Register,
    Message,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerMessage {
    message_type: MessageType,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Serialize)]
struct ChatMessage {
    from: String,
    message: String,
    time: f64,
}

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

fn now_millis() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as f64)
        .unwrap_or_default()
}

fn users_message(users: &Users) -> Result<String, serde_json::Error> {
    let names = users
        .lock()
        .unwrap()
        .iter()
        .map(|user| user.nick.clone())
        .collect();

    serde_json::to_string(&ServerMessage {
        message_type: MessageType::Users,
        data_array: Some(names),
        data: None,
    })
}

fn register_user(users: &Users, id: usize, nick: String) {
    let mut users = users.lock().unwrap();
    if let Some(user) = users.iter_mut().find(|user| user.id == id) {
        user.nick = nick;
    } else {
        users.push(User { id, nick });
    }
}

fn remove_user(users: &Users, id: usize) {
    users.lock().unwrap().retain(|user| user.id != id);
}

fn username_for(users: &Users, id: usize) -> Option<String> {
    users
        .lock()
        .unwrap()
        .iter()
        .find(|user| user.id == id)
        .map(|user| user.nick.clone())
}

fn chat_message(from: String, message: String) -> Result<String, serde_json::Error> {
    let data = serde_json::to_string(&ChatMessage {
        from,
        message,
        time: now_millis(),
    })?;

    serde_json::to_string(&ServerMessage {
        message_type: MessageType::Message,
        data_array: None,
        data: Some(data),
    })
}

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
    users: Users,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    let mut bcast_rx = bcast_tx.subscribe();

    println!("New websocket connection from {addr}");

    loop {
        tokio::select! {
            Some(msg) = ws_stream.next() => {
                let msg = msg?;
                if let Some(text) = msg.as_text() {
                    let parsed: ClientMessage = serde_json::from_str(text)?;
                    match parsed.message_type {
                        MessageType::Register => {
                            if let Some(nick) = parsed.data {
                                let nick = nick.trim().to_string();
                                if !nick.is_empty() {
                                    println!("Registering {addr} as {nick}");
                                    register_user(&users, id, nick);
                                    bcast_tx.send(users_message(&users)?)?;
                                }
                            }
                        }
                        MessageType::Message => {
                            if let (Some(from), Some(message)) = (username_for(&users, id), parsed.data) {
                                if !message.trim().is_empty() {
                                    println!("From {from}: {message}");
                                    bcast_tx.send(chat_message(from, message)?)?;
                                }
                            }
                        }
                        MessageType::Users => {}
                    }
                }
            }
            msg = bcast_rx.recv() => {
                let msg = msg?;
                ws_stream.send(Message::text(msg)).await?;
            }
            else => break,
        }
    }

    remove_user(&users, id);
    bcast_tx.send(users_message(&users)?)?;
    println!("Connection closed from {addr}");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(32);
    let users = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Rust websocket server listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        let bcast_tx = bcast_tx.clone();
        let users = users.clone();

        tokio::spawn(async move {
            let result = async move {
                let (_req, ws_stream) = ServerBuilder::new().accept(socket).await?;
                handle_connection(addr, ws_stream, bcast_tx, users).await
            }
            .await;

            if let Err(error) = result {
                eprintln!("Error handling {addr}: {error}");
            }
        });
    }
}
