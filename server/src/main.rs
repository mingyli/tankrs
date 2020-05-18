use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;

use anyhow::Result;
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use futures::StreamExt;
use log::{debug, info, warn};

use schema::{geometry, tank, world};
mod listener;
mod publisher;

type Buffer = Vec<u8>;
type Peers = Arc<Mutex<HashSet<SocketAddr>>>;
type WorldState = Arc<world::World>;
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
    let publisher = publisher::publish(&mut outgoing, world_state, peers.clone());

    // Listen for and enqueue actions from client.
    let listener = listener::listen(&mut incoming, actions);

    futures::future::select(Box::pin(publisher), Box::pin(listener)).await;

    warn!("Peer disconnected: {}", address);
    peers.lock().await.remove(&address);
    Ok(())
}

async fn run() -> Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:9001").await?;
    info!("Starting server");
    let peers = Peers::new(Mutex::new(HashSet::new()));
    let actions = ActionQueue::new(Mutex::new(VecDeque::<Buffer>::new()));

    let mut world = world::World::new();
    world.mut_tanks().push({
        let mut tank = tank::Tank::new();
        tank.set_position(geometry::Vec2 {
            x: 4.5,
            y: 6.7,
            ..Default::default()
        });
        tank
    });
    world.mut_tanks().push(tank::Tank::new());
    world.mut_tanks().push(tank::Tank::new());
    let world = Arc::new(world);

    // Spawn task to consume actions from action queue.
    task::spawn(listener::apply_actions(actions.clone()));

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

    task::block_on(run())
}
