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

    move_with_block_collision(world, &mut player.pos, &mut player.vel, &mut player.on_ground, dt);
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

    move_with_block_collision(
        world,
        &mut monster.pos,
        &mut monster.vel,
        &mut monster.on_ground,
        dt,
    );
}

fn move_with_block_collision(
    world: &World,
    pos: &mut Vec3,
    vel: &mut Vec3,
    on_ground: &mut bool,
    dt: f32,
) {
    *on_ground = false;
    let mut p = *pos;

    // X axis
    p.x += vel.x * dt;
    if collides(world, p) {
        p.x -= vel.x * dt;
        vel.x = 0.0;
    }

    // Z axis
    p.z += vel.z * dt;
    if collides(world, p) {
        p.z -= vel.z * dt;
        vel.z = 0.0;
    }

    // Y axis
    p.y += vel.y * dt;
    if collides(world, p) {
        if vel.y < 0.0 {
            *on_ground = true;
        }
        p.y -= vel.y * dt;
        vel.y = 0.0;
    }

    *pos = p;
}

fn collides(world: &World, pos: Vec3) -> bool {
    let half = Vec3::new(0.3, 0.9, 0.3);
    let min = Vec3::new(pos.x - half.x, pos.y, pos.z - half.z);
    let max = Vec3::new(pos.x + half.x, pos.y + 1.8, pos.z + half.z);

    let min_x = min.x.floor() as i32;
    let min_y = min.y.floor() as i32;
    let min_z = min.z.floor() as i32;
    let max_x = max.x.floor() as i32;
    let max_y = max.y.floor() as i32;
    let max_z = max.z.floor() as i32;

    for y in min_y..=max_y {
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                if world.block_at_world(x, y, z) != crate::world::Block::Air {
                    return true;
                }
            }
        }
    }
    false
}
