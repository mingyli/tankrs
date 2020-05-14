use crate::math::Position;

pub struct World {
    tanks: Vec<Tank>,
}

pub struct Tank {
    // TODO: discuss what player would look like and replace.
    player: u16,
    pos: Position,
}

impl Tank {
    pub fn player(&self) -> u16 {
        self.player
    }

    pub fn pos(&self) -> Position {
        self.pos
    }

    pub fn new(player: u16, pos: Position) -> Tank {
        Tank { player, pos }
    }
}

impl World {
    pub fn new() -> World {
        World { tanks: Vec::new() }
    }

    // TODO: replace with add_player after discussion on representing player.
    pub fn add_tank(&mut self, tank: Tank) {
        self.tanks.push(tank);
    }

    pub fn tanks(&self) -> &Vec<Tank> {
        &self.tanks
    }
}
