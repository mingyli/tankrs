use std::collections::HashSet;
use std::net::SocketAddr;
use std::time;

use anyhow::Result;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use futures::SinkExt;
use protobuf::Message;

// Publish world state at a regular interval.
pub async fn publish<T>(
    outgoing: &mut T,
    world_state: Arc<schema::World>,
    peers: Arc<Mutex<HashSet<SocketAddr>>>,
) -> Result<()>
where
    T: futures::Sink<tungstenite::Message> + std::marker::Unpin,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    loop {
        let mut heartbeat = schema::Heartbeat::new();
        heartbeat.set_world((*world_state).clone());
        outgoing
            .send(tungstenite::Message::Binary(heartbeat.write_to_bytes()?))
            .await?;
        outgoing
            .send(tungstenite::Message::Text(format!(
                "Here are the peers connected to the server: {:?}",
                peers.lock().await
            )))
            .await?;
        task::sleep(time::Duration::from_secs(1)).await;
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
        let world = Arc::new(World::new());
        let peers = [SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            69,
        )]
        .iter()
        .cloned()
        .collect();
        let peers = schema::Peers::new(Mutex::new(peers));
        let publish_future = publish(&mut sink, world.clone(), peers.clone());
        let timeout = time::Duration::from_secs_f32(1.5);

        let expected = {
            let mut expected = schema::Heartbeat::new();
            expected.set_world(World::new());
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
