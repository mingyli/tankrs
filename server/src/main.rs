use std::collections::HashMap;
use std::convert::TryFrom;
use std::time;

use anyhow::Result;
use async_std::sync::{Arc, Mutex, RwLock};
use async_std::task;

mod listener;
mod math;
mod publisher;
mod world;

async fn run() {
    let mut game_world = world::World::default();
    let actions = Arc::new(Mutex::new(HashMap::new()));
    let world_state = Arc::new(RwLock::new(schema::World::new()));
    let new_players = Arc::new(Mutex::new(Vec::new()));

    task::spawn(listener::accept_new_connections(
        Arc::clone(&actions),
        Arc::clone(&world_state),
        Arc::clone(&new_players),
    ));

    loop {
        let accumulated_actions = {
            let mut guard = actions.lock().await;
            std::mem::replace(&mut *guard, HashMap::new())
        };

        let accumulated_new_players = {
            let mut guard = new_players.lock().await;
            std::mem::replace(&mut *guard, Vec::new())
        };

        for player_uuid in accumulated_new_players {
            game_world.register_player(player_uuid);
        }

        game_world.apply_player_actions(&accumulated_actions);
        game_world.tick();

        {
            let mut write_guard = world_state.write().await;
            match schema::World::try_from(&game_world) {
                Ok(proto) => *write_guard = proto,
                Err(msg) => {
                    panic!("Could not parse proto {:?}", msg);
                }
            }
        }
        task::sleep(time::Duration::from_millis(100)).await;
    }
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

    task::block_on(run());
    Ok(())
}
