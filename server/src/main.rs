use std::collections::HashMap;

use anyhow::Result;
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::{Arc, Mutex, RwLock};
use async_std::task;
use futures::StreamExt;
use log::{debug, info};
use uuid::Uuid;

mod listener;
mod math;
mod publisher;
mod serialization;
mod world;

async fn handle_client(
    player_id: Uuid,
    stream: TcpStream,
    actions: Arc<Mutex<HashMap<Uuid, world::PlayerAction>>>,
    world_state: Arc<RwLock<schema::World>>,
) -> Result<()> {
    info!("Handling client.");

    let ws_stream = async_tungstenite::accept_async(stream).await?;

    let (mut outgoing, mut incoming) = ws_stream.split();

    // Spawn task to publish world state.
    // TODO: Publish to clients in batch instead.
    let publisher = publisher::publish(&mut outgoing, world_state);

    // Listen for and enqueue actions from client.
    let listener = listener::listen(player_id, &mut incoming, actions);

    futures::future::select(Box::pin(publisher), Box::pin(listener)).await;

    Ok(())
}

async fn run() -> Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:9001").await?;
    info!("Starting server");
    let actions = Arc::new(Mutex::new(HashMap::new()));

    let world = Arc::new(Mutex::new(world::World::new()));
    let world_state = Arc::new(RwLock::new(schema::World::new()));

    // Spawn task to consume actions from action queue.
    task::spawn(listener::run_game_loop(
        Arc::clone(&world),
        Arc::clone(&actions),
        Arc::clone(&world_state),
    ));

    // Listen for new WebSocket connections.
    while let Ok((stream, address)) = tcp_listener.accept().await {
        debug!("Received on address {}", address);
        let new_player_id = Uuid::new_v4();
        world.lock().await.register_player(new_player_id);
        task::spawn(handle_client(
            new_player_id,
            stream,
            Arc::clone(&actions),
            Arc::clone(&world_state),
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
