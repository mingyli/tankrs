use std::collections::HashMap;

use anyhow::Result;
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::{Arc, Mutex, RwLock};
use async_std::task;
use futures::{stream, StreamExt};
use log::{debug, info, warn};
use uuid::Uuid;

use crate::publisher;
use crate::world;

// Listen for and enqueue actions from client.
pub async fn listen<T>(
    player_id: Uuid,
    incoming: &mut T,
    actions: Arc<Mutex<HashMap<Uuid, world::PlayerAction>>>,
) -> Result<()>
where
    T: stream::Stream<Item = Result<tungstenite::Message, tungstenite::Error>> + std::marker::Unpin,
{
    while let Some(message) = incoming.next().await {
        let message = message?;
        match message {
            tungstenite::Message::Text(message) => {
                warn!("Received text: {}", message);
            }
            tungstenite::Message::Binary(buffer) => {
                let try_parse_action = protobuf::parse_from_bytes::<schema::Action>(&buffer);
                match try_parse_action {
                    Ok(mut action) => {
                        let player_action = world::PlayerAction::new(action.take_actions());
                        actions.lock().await.insert(player_id, player_action);
                    }
                    Err(msg) => warn!("Could not parse client message {:?}", msg),
                }
            }
            tungstenite::Message::Close(_) => {
                warn!("Input stream ended.");
                break;
            }
            _ => {
                debug!("Ignoring message {:?}", message);
            }
        }
    }
    Ok(())
}

// TODO(ming): Consume actions in non-blocking fashion instead of displaying actions periodically.
// pub async fn run_game_loop(
//     world: Arc<Mutex<world::World>>,
//     actions: Arc<Mutex<HashMap<Uuid, world::PlayerAction>>>,
//     world_state: Arc<RwLock<schema::World>>,
// ) -> Result<()> {
// }

async fn handle_client(
    player_id: Uuid,
    stream: TcpStream,
    actions: Arc<Mutex<HashMap<Uuid, world::PlayerAction>>>,
    world_state: Arc<RwLock<schema::World>>,
) -> Result<()> {
    info!("Handling client.");

    let ws_stream = async_tungstenite::accept_async(stream).await?;

    let (mut outgoing, mut incoming) = ws_stream.split();

    // Spawn task to publish world state.
    // TODO: Publish to clients in batch instead.
    let publisher = publisher::publish(&mut outgoing, world_state);

    // Listen for and enqueue actions from client.
    let listener = listen(player_id, &mut incoming, actions);

    futures::future::select(Box::pin(publisher), Box::pin(listener)).await;

    Ok(())
}

pub async fn accept_new_connections(
    actions: Arc<Mutex<HashMap<Uuid, world::PlayerAction>>>,
    world_state: Arc<RwLock<schema::World>>,
    new_players: Arc<Mutex<Vec<Uuid>>>,
) -> Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:9001").await?;
    info!("Starting server");
    // Listen for new WebSocket connections.
    while let Ok((stream, address)) = tcp_listener.accept().await {
        debug!("Received on address {}", address);
        let new_player_id = Uuid::new_v4();
        new_players.lock().await.push(new_player_id);
        task::spawn(handle_client(
            new_player_id,
            stream,
            Arc::clone(&actions),
            Arc::clone(&world_state),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::sync::Mutex;

    #[async_std::test]
    async fn test_listen() -> Result<()> {
        let mut stream = stream::iter(vec![
            Ok(tungstenite::Message::Text("hi".to_string())),
            Ok(tungstenite::Message::Close(None)),
            Ok(tungstenite::Message::Text("bye".to_string())),
        ]);
        let actions = Arc::new(Mutex::new(HashMap::new()));
        listen(Uuid::new_v4(), &mut stream, Arc::clone(&actions)).await?;
        assert_eq!(actions.lock().await.len(), 0);
        Ok(())
    }
}
