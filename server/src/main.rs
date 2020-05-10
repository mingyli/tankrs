use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::time;

use async_std::net::{TcpListener, TcpStream};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use async_tungstenite::WebSocketStream;
use futures::{stream, SinkExt, StreamExt};
use tungstenite::Message;

use schema::world_generated::{World, WorldArgs};

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

    let (outgoing, incoming) = ws_stream.split();

    // Spawn task to publish world state.
    // TODO: Publish to clients in batch instead.
    let publisher = publish(outgoing, world_state, peers.clone());

    // Listen for and enqueue actions from client.
    let listener = listen(incoming, actions);

    futures::pin_mut!(publisher);
    futures::pin_mut!(listener);

    futures::future::select(publisher, listener).await;

    println!("Peer disconnected: {}", address);
    peers.lock().await.remove(&address);
    Ok(())
}

async fn publish(
    mut outgoing: stream::SplitSink<WebSocketStream<TcpStream>, Message>,
    world_state: WorldState,
    peers: Peers,
) -> anyhow::Result<()> {
    loop {
        outgoing.send(Message::Binary(world_state.to_vec())).await?;
        outgoing
            .send(Message::Text(format!(
                "Here are the peers connected to server: {:?}",
                peers.lock().await
            )))
            .await?;
        task::sleep(time::Duration::from_secs(1)).await;
    }
}

async fn listen(
    mut incoming: stream::SplitStream<WebSocketStream<TcpStream>>,
    actions: ActionQueue,
) -> anyhow::Result<()> {
    while let Some(message) = incoming.next().await {
        let message = message?;
        actions.lock().await.push_back(format!("{}", message));
        match message {
            Message::Text(_) | Message::Binary(_) => {
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
    let world = World::create(
        &mut builder,
        &WorldArgs {
            width: 40,
            height: 30,
        },
    );
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
