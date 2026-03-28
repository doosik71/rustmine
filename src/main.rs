use crate::mesh::{build_chunk_top_mesh, build_debug_triangle};
use crate::render::Renderer;
use crate::world::{Block, Chunk, ChunkPos, World, CHUNK_SIZE, WORLD_HEIGHT};
use crate::{entity::Monster, physics::step_monster};
use std::sync::Arc;
use winit::event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

mod core;
mod entity;
mod mesh;
mod physics;
mod render;
mod world;

struct InputState {
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    mouse_held: bool,
    mouse_dx: f32,
    mouse_dy: f32,
    last_cursor: Option<(f64, f64)>,
}

impl InputState {
    fn new() -> Self {
        Self {
            forward: false,
            back: false,
            left: false,
            right: false,
            mouse_held: false,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            last_cursor: None,
        }
    }

    fn clear_frame(&mut self) {
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
    }
}

struct Camera {
    position: glam::Vec3,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    fn new(position: glam::Vec3) -> Self {
        Self {
            position,
            yaw: -135.0_f32.to_radians(),
            pitch: -45.0_f32.to_radians(),
        }
    }

    fn forward(&self) -> glam::Vec3 {
        glam::Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    fn right(&self) -> glam::Vec3 {
        self.forward().cross(glam::Vec3::Y).normalize()
    }

    fn apply_mouse(&mut self, dx: f32, dy: f32) {
        let sensitivity = 0.0025;
        self.yaw += dx * sensitivity;
        self.pitch -= dy * sensitivity;
        let limit = 1.5;
        if self.pitch > limit {
            self.pitch = limit;
        } else if self.pitch < -limit {
            self.pitch = -limit;
        }
    }

    fn step(&mut self, input: &InputState, dt: f32) {
        let mut dir = glam::Vec3::ZERO;
        if input.forward {
            dir += self.forward();
        }
        if input.back {
            dir -= self.forward();
        }
        if input.right {
            dir += self.right();
        }
        if input.left {
            dir -= self.right();
        }

        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
        }

        let speed = 8.0;
        self.position += dir * speed * dt;
    }
}

fn main() {
    let seed = 12345u64;
    let world = World::new(seed);
    let mut chunk = world.generate_chunk(ChunkPos { x: 0, z: 0 });

    let debug_triangle = false;
    let mut mesh = if debug_triangle {
        build_debug_triangle()
    } else {
        build_chunk_top_mesh(&chunk)
    };

    let event_loop = EventLoop::new().expect("event loop");
    let window = Arc::new(
        WindowBuilder::new()
        .with_title("RustMine - Prototype")
        .build(&event_loop)
        .expect("window"),
    );

    let window_for_renderer = Arc::clone(&window);
    let mut renderer = pollster::block_on(Renderer::new(&window_for_renderer, &mesh));
    if debug_triangle {
        renderer.update_view_proj(glam::Mat4::IDENTITY);
    }
    let mut input = InputState::new();
    let mut camera = Camera::new(glam::Vec3::new(20.0, 90.0, 20.0));
    let mut monster = Monster::new(crate::core::Vec3::new(8.0, 80.0, 8.0));
    let mut last_frame = std::time::Instant::now();

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => renderer.resize(size),
                WindowEvent::KeyboardInput { event: key, .. } => {
                    handle_keyboard(&mut input, &key);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if debug_triangle {
                        return;
                    }
                    if button == MouseButton::Left {
                        input.mouse_held = state == ElementState::Pressed;
                        if !input.mouse_held {
                            input.last_cursor = None;
                        }
                    }

                    if state == ElementState::Pressed {
                        let eye = camera.position;
                        let dir = camera.forward();
                        if let Some(hit) = raycast_block(&chunk, eye, dir, 64.0, 0.1) {
                            match button {
                                MouseButton::Left => {
                                    if set_block_world(&mut chunk, hit.block, Block::Air) {
                                        mesh = build_chunk_top_mesh(&chunk);
                                        renderer.update_mesh(&mesh);
                                    }
                                }
                                MouseButton::Right => {
                                    if set_block_world(&mut chunk, hit.prev, Block::Dirt) {
                                        mesh = build_chunk_top_mesh(&chunk);
                                        renderer.update_mesh(&mesh);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if input.mouse_held {
                        if let Some((last_x, last_y)) = input.last_cursor {
                            input.mouse_dx += (position.x - last_x) as f32;
                            input.mouse_dy += (position.y - last_y) as f32;
                        }
                        input.last_cursor = Some((position.x, position.y));
                    }
                }
                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = (now - last_frame).as_secs_f32().min(0.05);
                    last_frame = now;

                    if !debug_triangle {
                        if input.mouse_held {
                            camera.apply_mouse(input.mouse_dx, input.mouse_dy);
                        }
                        camera.step(&input, dt);
                        step_monster(
                            &world,
                            &mut monster,
                            crate::core::Vec3::new(
                                camera.position.x,
                                camera.position.y,
                                camera.position.z,
                            ),
                            dt,
                        );
                        let eye = camera.position;
                        let center = eye + camera.forward();
                        renderer.update_camera(eye, center);
                    }

                    input.clear_frame();
                    if let Err(err) = renderer.render() {
                        match err {
                            wgpu::SurfaceError::Lost => renderer.resize(window.inner_size()),
                            wgpu::SurfaceError::OutOfMemory => elwt.exit(),
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).expect("event loop run");
}

fn handle_keyboard(input: &mut InputState, event: &KeyEvent) {
    let pressed = event.state == ElementState::Pressed;
    match event.physical_key {
        PhysicalKey::Code(KeyCode::KeyW) => input.forward = pressed,
        PhysicalKey::Code(KeyCode::KeyS) => input.back = pressed,
        PhysicalKey::Code(KeyCode::KeyA) => input.left = pressed,
        PhysicalKey::Code(KeyCode::KeyD) => input.right = pressed,
        _ => {}
    }
}

#[derive(Clone, Copy, Debug)]
struct RayHit {
    block: (i32, i32, i32),
    prev: (i32, i32, i32),
}

fn raycast_block(
    chunk: &Chunk,
    origin: glam::Vec3,
    dir: glam::Vec3,
    max_dist: f32,
    step: f32,
) -> Option<RayHit> {
    if dir.length_squared() <= 0.0 {
        return None;
    }
    let mut t = 0.0;
    let mut last_cell = None;
    while t <= max_dist {
        let p = origin + dir * t;
        let cell = (p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
        if last_cell != Some(cell) {
            if in_chunk(cell) {
                let (x, y, z) = (cell.0 as usize, cell.1 as usize, cell.2 as usize);
                if chunk.block_at(x, y, z) != Block::Air {
                    let prev = last_cell.unwrap_or(cell);
                    return Some(RayHit { block: cell, prev });
                }
            }
            last_cell = Some(cell);
        }
        t += step;
    }
    None
}

fn in_chunk(cell: (i32, i32, i32)) -> bool {
    cell.0 >= 0
        && cell.2 >= 0
        && cell.1 >= 0
        && cell.0 < CHUNK_SIZE as i32
        && cell.2 < CHUNK_SIZE as i32
        && cell.1 < WORLD_HEIGHT as i32
}

fn set_block_world(chunk: &mut Chunk, cell: (i32, i32, i32), block: Block) -> bool {
    if !in_chunk(cell) {
        return false;
    }
    chunk.set_block(cell.0 as usize, cell.1 as usize, cell.2 as usize, block)
}
