use std::collections::HashMap;
use std::time;

use anyhow::Result;
use async_std::sync::{Arc, Mutex, RwLock};
use async_std::task;
use futures::{stream, StreamExt};
use log::{debug, warn};
use uuid::Uuid;

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
pub async fn run_game_loop(
    world: Arc<Mutex<world::World>>,
    actions: Arc<Mutex<HashMap<Uuid, world::PlayerAction>>>,
    world_state: Arc<RwLock<schema::World>>,
) -> Result<()> {
    loop {
        //info!("Clearing action queue...");
        let actions = {
            let mut guard = actions.lock().await;
            std::mem::replace(&mut *guard, HashMap::new())
        };
        //info!("Contents of action queue:");
        {
            let mut w = world.lock().await;
            w.apply_player_actions(&actions);
            w.tick();
        }
        {
            let mut write_guard = world_state.write().await;
            *write_guard = schema::World::from(&*world.lock().await);
        }
        task::sleep(time::Duration::from_millis(100)).await;
    }
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
