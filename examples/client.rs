use tungstenite::{connect, Message};
use url::Url;

use schema::world_generated::get_root_as_world_buf;

fn main() {
    let (mut socket, response) =
        connect(Url::parse("ws://localhost:9001/socket").unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    socket
        .write_message(Message::Text("Hello WebSocket".into()))
        .unwrap();
    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {:?}", msg);
        match msg {
            Message::Binary(buffer) => {
                let world = get_root_as_world_buf(&buffer);
                println!(
                    "My world has width {} and height {}.",
                    world.width(),
                    world.height()
                );
            }
            Message::Text(text) => {
                println!("Received text: {}", text);
            }
            _ => {}
        }
    }
}
