use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;

use anyhow::Result;
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::{Arc, Mutex, RwLock};
use async_std::task;
use futures::StreamExt;
use log::{debug, info, warn};

mod listener;
mod math;
mod publisher;
mod world;

async fn handle_client(
    stream: TcpStream,
    address: SocketAddr,
    actions: Arc<Mutex<VecDeque<Vec<u8>>>>,
    peers: Arc<Mutex<HashSet<SocketAddr>>>,
    world_state: Arc<RwLock<schema::World>>,
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
    let peers = Arc::new(Mutex::new(HashSet::new()));
    let actions = Arc::new(Mutex::new(VecDeque::<Vec<u8>>::new()));

    let mut world = schema::World::new();
    world.mut_tanks().push({
        let mut tank = schema::Tank::new();
        tank.set_position(schema::Vec2 {
            x: 4.5,
            y: 6.7,
            ..schema::Vec2::default()
        });
        tank
    });
    world.mut_tanks().push(schema::Tank::new());
    world.mut_tanks().push(schema::Tank::new());
    let world = Arc::new(RwLock::new(world));

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
