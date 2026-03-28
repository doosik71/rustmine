use crate::core::Vec3;
use crate::entity::{Monster, Player};
use crate::world::World;

const GRAVITY: f32 = -9.8;
const WALK_SPEED: f32 = 4.0;
const MONSTER_SPEED: f32 = 2.5;

pub struct InputState {
    pub move_dir: Vec3,
    pub jump: bool,
}

impl InputState {
    pub fn idle() -> Self {
        Self {
            move_dir: Vec3::default(),
            jump: false,
        }
    }
}

pub fn step_player(world: &World, player: &mut Player, input: &InputState, dt: f32) {
    let mut wish = input.move_dir;
    if wish.length() > 1.0 {
        wish = wish.normalized();
    }

    player.vel.x = wish.x * WALK_SPEED;
    player.vel.z = wish.z * WALK_SPEED;

    if input.jump && player.on_ground {
        player.vel.y = 6.5;
        player.on_ground = false;
    }

    player.vel.y += GRAVITY * dt;

    player.pos.x += player.vel.x * dt;
    player.pos.y += player.vel.y * dt;
    player.pos.z += player.vel.z * dt;

    let ground = world.height_at_world(player.pos.x.floor() as i32, player.pos.z.floor() as i32) as f32;
    if player.pos.y < ground {
        player.pos.y = ground;
        player.vel.y = 0.0;
        player.on_ground = true;
    }
}

pub fn step_monster(world: &World, monster: &mut Monster, target: Vec3, dt: f32) {
    let mut dir = Vec3::new(target.x - monster.pos.x, 0.0, target.z - monster.pos.z);
    if dir.length() > 0.1 {
        dir = dir.normalized();
    } else {
        dir = Vec3::default();
    }

    monster.vel.x = dir.x * MONSTER_SPEED;
    monster.vel.z = dir.z * MONSTER_SPEED;
    monster.vel.y += GRAVITY * dt;

    monster.pos.x += monster.vel.x * dt;
    monster.pos.y += monster.vel.y * dt;
    monster.pos.z += monster.vel.z * dt;

    let ground = world.height_at_world(monster.pos.x.floor() as i32, monster.pos.z.floor() as i32)
        as f32;
    if monster.pos.y < ground {
        monster.pos.y = ground;
        monster.vel.y = 0.0;
        monster.on_ground = true;
    }
}
