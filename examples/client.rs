use async_std::task;
use async_tungstenite::async_std::connect_async;
use futures::{SinkExt, StreamExt};
use tungstenite::Message;
use url::Url;

use schema::actions_generated;
use schema::messages_generated::{self, get_root_as_message_root};

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
            .send(Message::Text("Hello WebSocket".into()))
            .await
            .unwrap();
        loop {
            let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
            let action = actions_generated::ActionRoot::create(
                &mut builder,
                &actions_generated::ActionRootArgs {
                    movement: actions_generated::Movement::KeyUp,
                },
            );
            builder.finish(action, None);
            let buffer = builder.finished_data();
            write.send(Message::Binary(buffer.to_vec())).await.unwrap();

            task::sleep(std::time::Duration::from_millis(300)).await;
        }
    };

    let reader = async {
        println!("Beginning reader");
        while let Some(message) = read.next().await {
            let message = message.unwrap();
            match message {
                Message::Text(text) => {
                    println!("Received text: {}", text);
                }
                Message::Binary(buffer) => {
                    let message = get_root_as_message_root(buffer.as_slice());
                    match message.message_type() {
                        messages_generated::Message::WorldState => {
                            let world_state = message
                                .message_as_world_state()
                                .expect("Failed to read message as a world state.");
                            println!("Player: {:?}", world_state.player());
                            println!("Others: {:?}", world_state.others());
                        }
                        messages_generated::Message::GameParams => {
                            let game_params = message
                                .message_as_game_params()
                                .expect("Failed to read message as game params.");
                            println!("Game params width: {}", game_params.width());
                            println!("Game params height: {}", game_params.height());
                        }
                        _ => unreachable!(),
                    }
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
