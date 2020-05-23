use async_std::task;
use async_tungstenite::async_std::connect_async;
use futures::{SinkExt, StreamExt};
use protobuf::Message;
use url::Url;

use schema::action;

async fn run() -> anyhow::Result<()> {
    let (socket, response) = connect_async(Url::parse("ws://localhost:9001/socket")?).await?;

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    let (mut write, mut read) = socket.split();

    let writer = async {
        write
            .send(tungstenite::Message::Text("Hello WebSocket".into()))
            .await
            .unwrap();
        loop {
            let mut action = schema::Action::new();
            action.mut_actions().push(action::KeyPress::DOWN);
            write
                .send(tungstenite::Message::Binary(
                    action.write_to_bytes().unwrap(),
                ))
                .await
                .unwrap();

            task::sleep(std::time::Duration::from_millis(300)).await;
        }
    };

    let reader = async {
        println!("Beginning reader");
        while let Some(message) = read.next().await {
            let message = message.unwrap();
            match message {
                tungstenite::Message::Text(text) => {
                    println!("Received text: {}", text);
                }
                tungstenite::Message::Binary(buffer) => {
                    let server_message: schema::ServerMessage =
                        protobuf::parse_from_bytes(&buffer).unwrap();
                    println!("{:?}", server_message);
                }
                _ => {
                    println!("Received other");
                }
            }
        }
    };
    futures::future::select(Box::pin(writer), Box::pin(reader)).await;
    println!("Connection terminated.");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    task::block_on(run())
}
