use crate::core::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub pos: Vec3,
    pub vel: Vec3,
    pub on_ground: bool,
}

impl Player {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            vel: Vec3::default(),
            on_ground: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Monster {
    pub pos: Vec3,
    pub vel: Vec3,
    pub on_ground: bool,
}

impl Monster {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            vel: Vec3::default(),
            on_ground: false,
        }
    }
}
