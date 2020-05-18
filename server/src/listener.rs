use std::collections::VecDeque;
use std::time;

use anyhow::Result;
use async_std::task;
use futures::{stream, StreamExt};
use log::{debug, info, warn};

use crate::ActionQueue;
// use schema::{heartbeat, math, tank, world};

// Listen for and enqueue actions from client.
pub async fn listen<T>(incoming: &mut T, actions: ActionQueue) -> Result<()>
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
                actions.lock().await.push_back(buffer);
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

// TODO: Consume actions in non-blocking fashion instead of displaying actions periodically.
pub async fn apply_actions(actions_arc: ActionQueue) -> Result<()> {
    loop {
        info!("Clearing action queue...");
        let actions = {
            let mut guard = actions_arc.lock().await;
            std::mem::replace(&mut *guard, VecDeque::new())
        };
        info!("Contents of action queue:");
        for action in actions {
            let action: schema::action::Action = protobuf::parse_from_bytes(&action)?;
            info!("  Action: {:?}", action);
        }
        task::sleep(time::Duration::from_secs(1)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Buffer;
    use async_std::sync::Mutex;

    #[async_std::test]
    async fn test_listen() -> Result<()> {
        let mut stream = stream::iter(vec![
            Ok(tungstenite::Message::Text("hi".to_string())),
            Ok(tungstenite::Message::Close(None)),
            Ok(tungstenite::Message::Text("bye".to_string())),
        ]);
        let actions = ActionQueue::new(Mutex::new(VecDeque::<Buffer>::new()));
        listen(&mut stream, actions.clone()).await?;
        assert_eq!(*actions.lock().await, VecDeque::<Buffer>::new());
        Ok(())
    }
}
