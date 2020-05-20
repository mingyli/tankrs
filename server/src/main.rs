use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::time;

use anyhow::Result;
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use futures::{stream, SinkExt, StreamExt};
use log::{debug, info, warn};
use tungstenite::Message;
use uuid::Uuid;

use schema::actions_generated::get_root_as_action_root;

mod math;
mod serialization;
mod world;
use serialization::{Buffer, Config, SerializableAsMessage};
use world::World;

type Peers = Arc<Mutex<HashSet<SocketAddr>>>;
type WorldState = Arc<Buffer>;
type ActionQueue = Arc<Mutex<VecDeque<Buffer>>>;

async fn handle_client(
    stream: TcpStream,
    address: SocketAddr,
    actions: ActionQueue,
    peers: Peers,
    world_state: WorldState,
) -> Result<()> {
    info!("Handling client.");

    let ws_stream = async_tungstenite::accept_async(stream).await?;

    peers.lock().await.insert(address);

    let (mut outgoing, mut incoming) = ws_stream.split();

    // Spawn task to publish world state.
    // TODO: Publish to clients in batch instead.
    let publisher = publish(&mut outgoing, world_state, peers.clone());

    // Listen for and enqueue actions from client.
    let listener = listen(&mut incoming, actions);

    futures::future::select(Box::pin(publisher), Box::pin(listener)).await;

    warn!("Peer disconnected: {}", address);
    peers.lock().await.remove(&address);
    Ok(())
}

// Publish world state at a regular interval.
async fn publish<T>(outgoing: &mut T, world_state: WorldState, peers: Peers) -> Result<()>
where
    T: futures::Sink<Message> + std::marker::Unpin,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    loop {
        outgoing.send(Message::Binary(world_state.to_vec())).await?;
        outgoing
            .send(Message::Text(format!(
                "Here are the peers connected to the server: {:?}",
                peers.lock().await
            )))
            .await?;
        task::sleep(time::Duration::from_secs(1)).await;
    }
}

// Listen for and enqueue actions from client.
async fn listen<T>(incoming: &mut T, actions: ActionQueue) -> Result<()>
where
    T: stream::Stream<Item = Result<Message, tungstenite::Error>> + std::marker::Unpin,
{
    while let Some(message) = incoming.next().await {
        let message = message?;
        match message {
            Message::Text(message) => {
                warn!("Received text: {}", message);
            }
            Message::Binary(buffer) => {
                actions.lock().await.push_back(buffer);
            }
            Message::Close(_) => {
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

async fn run() -> Result<()> {
    env_logger::builder()
        .format(|buffer, record| {
            use std::io::Write;
            writeln!(
                buffer,
                "[{} {} {}:{}] {}",
                buffer.timestamp(),
                buffer.default_styled_level(record.level()),
                record.file().unwrap_or("UNKNOWNFILE"),
                record.line().unwrap_or_default(),
                record.args()
            )
        })
        .init();
    let tcp_listener = TcpListener::bind("127.0.0.1:9001").await?;
    info!("Starting server");
    let peers = Peers::new(Mutex::new(HashSet::new()));
    let actions = ActionQueue::new(Mutex::new(VecDeque::<Buffer>::new()));

    // Spawn task to consume actions from action queue.
    // TODO: Consume actions in non-blocking fashion instead of displaying actions periodically.
    let actions_arc = actions.clone();
    task::spawn(async move {
        loop {
            info!("Clearing action queue...");
            let actions = {
                let mut guard = actions_arc.lock().await;
                std::mem::replace(&mut *guard, VecDeque::new())
            };
            info!("Contents of action queue:");
            for action in actions {
                let action = get_root_as_action_root(action.as_slice());
                info!("Action: {:?}", action.movement());
            }
            task::sleep(time::Duration::from_secs(1)).await;
        }
    });

    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
    let mut world = World::new();
    let first_uuid = Uuid::new_v4();

    world.register_player(first_uuid);
    world.register_player(Uuid::new_v4());
    world.register_player(Uuid::new_v4());

    let world = Arc::new(world.serialize(&mut builder, &Config::new(first_uuid))?);

    // Listen for new WebSocket connections.
    while let Ok((stream, address)) = tcp_listener.accept().await {
        debug!("Received on address {}", address);
        task::spawn(handle_client(
            stream,
            address,
            actions.clone(),
            peers.clone(),
            world.clone(),
        ));
    }
    Ok(())
}

fn main() -> Result<()> {
    task::block_on(run())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[async_std::test]
    async fn test_listen() -> Result<()> {
        let mut stream = stream::iter(vec![
            Ok(Message::Text("hi".to_string())),
            Ok(Message::Close(None)),
            Ok(Message::Text("bye".to_string())),
        ]);
        let actions = ActionQueue::new(Mutex::new(VecDeque::<Buffer>::new()));
        listen(&mut stream, actions.clone()).await?;
        assert_eq!(*actions.lock().await, VecDeque::<Buffer>::new());
        Ok(())
    }

    #[async_std::test]
    async fn test_publish() -> Result<()> {
        let mut sink = VecDeque::new();
        let world = WorldState::new(vec![0u8, 1]);
        let peers = [SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            69,
        )]
        .iter()
        .cloned()
        .collect();
        let peers = Peers::new(Mutex::new(peers));
        let publish_future = publish(&mut sink, world.clone(), peers.clone());
        let timeout = time::Duration::from_secs_f32(1.5);
        assert!(async_std::future::timeout(timeout, publish_future)
            .await
            .is_err());
        assert_eq!(
            sink,
            vec![
                Message::Binary(vec![0, 1]),
                Message::Text(
                    "Here are the peers connected to the server: {V4(127.0.0.1:69)}".to_string()
                ),
                Message::Binary(vec![0, 1]),
                Message::Text(
                    "Here are the peers connected to the server: {V4(127.0.0.1:69)}".to_string()
                ),
            ]
        );
        Ok(())
    }
}
