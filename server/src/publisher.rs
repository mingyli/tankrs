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
        task::sleep(time::Duration::from_millis(17)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schema::World;
    use std::collections::VecDeque;

    // TODO(mluogh): this test is flaky, should not depend on time, inject a clock or move logic
    // elsewhere
    #[async_std::test]
    async fn test_publish() -> Result<()> {
        let mut sink = VecDeque::new();
        let world = Arc::new(RwLock::new(World::new()));
        let publish_future = publish(&mut sink, world.clone());
        let timeout = time::Duration::from_millis(50);

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
                //tungstenite::Message::Binary(expected.write_to_bytes()?),
            ]
        );
        Ok(())
    }
}
