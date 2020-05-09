mod world;

use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::spawn;
use world::{Block, BlockType, DiscretePos, World};

use tungstenite::Message;

fn handle_client(
    stream: TcpStream,
    my: &Arc<Vec<u8>>,
) -> tungstenite::Result<()> {
    let mut socket = tungstenite::accept(stream).unwrap();
    println!("Running test");
    loop {
        use std::{thread, time};
        socket.write_message(Message::Text("hi hi hi".to_string()))?;
        socket.write_message(Message::Binary(my.to_vec()))?;
        thread::sleep(time::Duration::from_secs(2));
    }
}

fn main() {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
    let mut my_world = World::new(69, 420);
    my_world.add_block(Block::new(DiscretePos(0, 0), BlockType::DESTRUCTIBLE));
    my_world
        .add_block(Block::new(DiscretePos(1, 0), BlockType::INDESTRUCTIBLE));

    let world_arc = Arc::new(my_world.to_fb_bytes(&mut builder));

    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    println!("Starting server");
    for stream in server.incoming() {
        let w = world_arc.clone();
        spawn(move || match stream {
            Ok(stream) => {
                if let Err(err) = handle_client(stream, &w) {
                    match err {
                        tungstenite::Error::ConnectionClosed
                        | tungstenite::Error::Protocol(_)
                        | tungstenite::Error::Utf8 => (),
                        e => eprintln!("test: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Error accepting stream: {}", e),
        });
    }
}
