use auto_ops::{impl_op_ex, impl_op_ex_commutative};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const UP: Vec2 = Vec2 { x: 0.0, y: 1.0 };
    pub const DOWN: Vec2 = Vec2 { x: 0.0, y: -1.0 };
    pub const LEFT: Vec2 = Vec2 { x: -1.0, y: 0.0 };
    pub const RIGHT: Vec2 = Vec2 { x: 1.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }
}

impl_op_ex!(+ |a: &Vec2, b: &Vec2| -> Vec2 {
    Vec2 {
        x: a.x + b.x, 
        y: a.y + b.y
    } 
});

impl_op_ex!(-|a: &Vec2, b: &Vec2| -> Vec2 {
    Vec2 {
        x: a.x - b.x,
        y: a.y - b.y,
    }
});

impl_op_ex!(+= |a: &mut Vec2, b: &Vec2| {
    a.x += b.x;
    a.y += b.y
});

impl_op_ex_commutative!(*|magnitude: f32, vec: &Vec2| -> Vec2 {
    Vec2 {
        x: magnitude * vec.x,
        y: magnitude * vec.y,
    }
});
