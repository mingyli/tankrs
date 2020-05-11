mod world;

use std::sync::Arc;
use std::time;
use world::{Block, BlockType, World};

use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use futures::prelude::*;
use tungstenite::Message;

use schema::message_generated::{
    get_root_as_visible_state_buf, VisibleStateBuf, VisibleStateBufArgs,
};

async fn handle_client(stream: TcpStream, my: Arc<Vec<u8>>) -> anyhow::Result<()> {
    let mut ws_stream = async_tungstenite::accept_async(stream).await?;

    println!("Running test");
    let x_end: f64 = 10.0;
    let tick_rate = time::Duration::from_millis(1000 / 60);
    let ticks = 3 * 60;

    let world = get_root_as_visible_state_buf(my.as_slice());

    loop {
        ws_stream.send(Message::Binary(my.to_vec())).await?;
        println!("{:?}", world.blocks().unwrap().len());
        for i in 0..ticks {
            let pos: f64 = (x_end / f64::from(ticks)) * f64::from(i);
            ws_stream.send(Message::Text(pos.to_string())).await?;
            task::sleep(tick_rate).await;
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
    let mut my_world = World::new(69, 420);
    my_world.add_block(Block::new(0, 0, BlockType::Destructible));
    my_world.add_block(Block::new(1, 0, BlockType::Indestructible));

    builder.reset();
    let blocks_buf = Some(my_world.add_blocks_to_fb(&mut builder));
    let state = VisibleStateBuf::create(&mut builder, &VisibleStateBufArgs { blocks: blocks_buf });
    builder.finish(state, None);
    let state_arc = Arc::new(builder.finished_data().to_vec());

    let server = TcpListener::bind("127.0.0.1:9001").await?;
    println!("Starting server");
    while let Ok((stream, address)) = server.accept().await {
        println!("Received on address {}", address);
        let w = state_arc.clone();
        task::spawn(handle_client(stream, w));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    task::block_on(run())
}
