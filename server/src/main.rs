use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::spawn;

use tungstenite::Message;

use schema::world_generated::{World, WorldArgs};

fn handle_client(stream: TcpStream, my: &Arc<Vec<u8>>) -> tungstenite::Result<()> {
    use std::{thread, time};
    let mut socket = tungstenite::accept(stream).unwrap();
    println!("Running test");
    let x_end: f32 = 10.0;
    let tick_rate = time::Duration::from_millis(1000 / 60);
    let ticks = 3 * 60;
    loop {
        for i in 0..ticks {
            let pos: f32 = (x_end / (ticks as f32)) * (i as f32);
            socket.write_message(Message::Text(pos.to_string()))?;
            thread::sleep(tick_rate);
        }
    }
}

fn main() {
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
