use std::time;

use anyhow::Result;
use async_std::sync::{Arc, RwLock};
use async_std::task;
use futures::SinkExt;
use protobuf::Message;

// Publish world state at a regular interval.
pub async fn publish<T>(outgoing: &mut T, world_state: Arc<RwLock<schema::World>>) -> Result<()>
where
    T: futures::Sink<tungstenite::Message> + std::marker::Unpin,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    let mut server_message = schema::ServerMessage::new();
    loop {
        server_message
            .mut_heartbeat()
            .set_world(world_state.read().await.clone());
        outgoing
            .send(tungstenite::Message::Binary(
                server_message.write_to_bytes()?,
            ))
            .await?;
        task::sleep(time::Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::sync::Mutex;
    use schema::World;
    use std::collections::VecDeque;
    use std::net::SocketAddr;

    #[async_std::test]
    async fn test_publish() -> Result<()> {
        let mut sink = VecDeque::new();
        let world = Arc::new(RwLock::new(World::new()));
        let peers = [SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            69,
        )]
        .iter()
        .cloned()
        .collect();
        let peers = Arc::new(Mutex::new(peers));
        let publish_future = publish(&mut sink, world.clone(), peers.clone());
        let timeout = time::Duration::from_secs_f32(1.5);

        let expected = {
            let mut expected = schema::ServerMessage::new();
            expected.mut_heartbeat().set_world(World::new());
            expected
        };
        assert!(async_std::future::timeout(timeout, publish_future)
            .await
            .is_err());
        assert_eq!(
            sink,
            vec![
                tungstenite::Message::Binary(expected.write_to_bytes()?),
                tungstenite::Message::Text(
                    "Here are the peers connected to the server: {V4(127.0.0.1:69)}".to_string()
                ),
                tungstenite::Message::Binary(expected.write_to_bytes()?),
                tungstenite::Message::Text(
                    "Here are the peers connected to the server: {V4(127.0.0.1:69)}".to_string()
                ),
            ]
        );
        Ok(())
    }
}
