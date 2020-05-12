mod world;

use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::time;
use world::{Serializable, World};

use async_std::net::{TcpListener, TcpStream};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use futures::{stream, SinkExt, StreamExt};
use tungstenite::Message;

type Peers = Arc<Mutex<HashSet<SocketAddr>>>;
type WorldState = Arc<Vec<u8>>;
type Action = String;
type ActionQueue = Arc<Mutex<VecDeque<Action>>>;

async fn handle_client(
    stream: TcpStream,
    address: SocketAddr,
    actions: ActionQueue,
    peers: Peers,
    world_state: WorldState,
) -> anyhow::Result<()> {
    println!("Handling client.");

    let ws_stream = async_tungstenite::accept_async(stream).await?;

    peers.lock().await.insert(address);

    let (mut outgoing, mut incoming) = ws_stream.split();

    // Spawn task to publish world state.
    // TODO: Publish to clients in batch instead.
    let publisher = publish(&mut outgoing, world_state, peers.clone());

    // Listen for and enqueue actions from client.
    let listener = listen(&mut incoming, actions);

    futures::future::select(Box::pin(publisher), Box::pin(listener)).await;

    println!("Peer disconnected: {}", address);
    peers.lock().await.remove(&address);
    Ok(())
}

// Publish world state at a regular interval.
async fn publish<T>(outgoing: &mut T, world_state: WorldState, peers: Peers) -> anyhow::Result<()>
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
async fn listen<T>(incoming: &mut T, actions: ActionQueue) -> anyhow::Result<()>
where
    T: stream::Stream<Item = Result<Message, tungstenite::Error>> + std::marker::Unpin,
{
    while let Some(message) = incoming.next().await {
        let message = message?;
        match message {
            Message::Text(_) | Message::Binary(_) => {
                actions.lock().await.push_back(format!("{}", message));
                println!("Received: {:?}", message);
            }
            Message::Close(_) => {
                println!("Input stream ended.");
                break;
            }
            _ => {
                println!("Ignoring message {:?}", message);
            }
        }
    }
    Ok(())
}

async fn run() -> anyhow::Result<()> {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
    let world = World::new(250, 250);
    let world = world.add_to_fb(&mut builder);
    builder.finish(world, None);
    let buffer = builder.finished_data().to_vec();
    let world = WorldState::new(buffer);

    let tcp_listener = TcpListener::bind("127.0.0.1:9001").await?;
    println!("Starting server");
    let peers = Peers::new(Mutex::new(HashSet::new()));
    let actions = ActionQueue::new(Mutex::new(VecDeque::<Action>::new()));

    // Spawn task to consume actions from action queue.
    // TODO: Consume actions in non-blocking fashion instead of displaying actions periodically.
    let actions_arc = actions.clone();
    task::spawn(async move {
        loop {
            println!("Contents of action queue: {:?}", actions_arc.lock().await);
            task::sleep(time::Duration::from_secs(1)).await;
        }
    });

    // Listen for new WebSocket connections.
    while let Ok((stream, address)) = tcp_listener.accept().await {
        println!("Received on address {}", address);
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

fn main() -> anyhow::Result<()> {
    task::block_on(run())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[async_std::test]
    async fn test_listen() -> anyhow::Result<()> {
        let mut stream = stream::iter(vec![
            Ok(Message::Text("hi".to_string())),
            Ok(Message::Close(None)),
            Ok(Message::Text("bye".to_string())),
        ]);
        let actions = ActionQueue::new(Mutex::new(VecDeque::<Action>::new()));
        listen(&mut stream, actions.clone()).await?;
        assert_eq!(*actions.lock().await, vec!["hi"]);
        Ok(())
    }

    #[async_std::test]
    async fn test_publish() -> anyhow::Result<()> {
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
