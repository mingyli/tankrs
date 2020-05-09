use std::sync::Arc;
use std::time;

use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use futures::prelude::*;
use tungstenite::Message;

use schema::world_generated::{World, WorldArgs};

async fn handle_client(stream: TcpStream, my: Arc<Vec<u8>>) -> anyhow::Result<()> {
    let mut ws_stream = async_tungstenite::accept_async(stream).await?;
    println!("Running test");
    let x_end: f64 = 10.0;
    let tick_rate = time::Duration::from_millis(1000 / 60);
    let ticks = 3 * 60;

    loop {
        ws_stream.send(Message::Binary(my.to_vec())).await?;
        for i in 0..ticks {
            let pos: f64 = (x_end / f64::from(ticks)) * f64::from(i);
            ws_stream.send(Message::Text(pos.to_string())).await?;
            task::sleep(tick_rate).await;
        }
    }
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
    let world_arc = Arc::new(buffer);

    let server = TcpListener::bind("127.0.0.1:9001").await?;
    println!("Starting server");
    while let Ok((stream, address)) = server.accept().await {
        println!("Received on address {}", address);
        let w = world_arc.clone();
        task::spawn(handle_client(stream, w));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    task::block_on(run())
}
