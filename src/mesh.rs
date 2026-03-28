use crate::world::{Block, Chunk, CHUNK_SIZE, WORLD_HEIGHT};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn build_chunk_top_mesh(chunk: &Chunk) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for y in 0..WORLD_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.block_at(x, y, z);
                if block == Block::Air {
                    continue;
                }

                // Top
                if is_air(chunk, x as i32, y as i32 + 1, z as i32) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::Top,
                        x,
                        y,
                        z,
                        shade(block, 1.0),
                    );
                }
                // Bottom
                if is_air(chunk, x as i32, y as i32 - 1, z as i32) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::Bottom,
                        x,
                        y,
                        z,
                        shade(block, 0.6),
                    );
                }
                // North (-z)
                if is_air(chunk, x as i32, y as i32, z as i32 - 1) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::North,
                        x,
                        y,
                        z,
                        shade(block, 0.8),
                    );
                }
                // South (+z)
                if is_air(chunk, x as i32, y as i32, z as i32 + 1) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::South,
                        x,
                        y,
                        z,
                        shade(block, 0.8),
                    );
                }
                // West (-x)
                if is_air(chunk, x as i32 - 1, y as i32, z as i32) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::West,
                        x,
                        y,
                        z,
                        shade(block, 0.75),
                    );
                }
                // East (+x)
                if is_air(chunk, x as i32 + 1, y as i32, z as i32) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::East,
                        x,
                        y,
                        z,
                        shade(block, 0.75),
                    );
                }
            }
        }
    }

    Mesh { vertices, indices }
}

#[allow(dead_code)]
pub fn build_debug_triangle() -> Mesh {
    let vertices = vec![
        Vertex {
            position: [-0.5, -0.5, 0.0],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.5, 0.0],
            color: [0.0, 0.0, 1.0],
        },
    ];
    let indices = vec![0, 1, 2];
    Mesh { vertices, indices }
}

fn block_color(block: Block) -> [f32; 3] {
    match block {
        Block::Grass => [0.3, 0.7, 0.3],
        Block::Dirt => [0.55, 0.35, 0.2],
        Block::Stone => [0.5, 0.5, 0.5],
        Block::Air => [0.7, 0.85, 1.0],
    }
}

fn shade(block: Block, factor: f32) -> [f32; 3] {
    let base = block_color(block);
    [base[0] * factor, base[1] * factor, base[2] * factor]
}

fn is_air(chunk: &Chunk, x: i32, y: i32, z: i32) -> bool {
    if x < 0
        || z < 0
        || y < 0
        || x >= CHUNK_SIZE as i32
        || z >= CHUNK_SIZE as i32
        || y >= WORLD_HEIGHT as i32
    {
        return true;
    }
    chunk.block_at(x as usize, y as usize, z as usize) == Block::Air
}

#[derive(Clone, Copy, Debug)]
enum Face {
    Top,
    Bottom,
    North,
    South,
    West,
    East,
}

fn add_face(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    face: Face,
    x: usize,
    y: usize,
    z: usize,
    color: [f32; 3],
) {
    let x0 = x as f32;
    let y0 = y as f32;
    let z0 = z as f32;
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let z1 = z0 + 1.0;

    let base_index = vertices.len() as u32;
    match face {
        Face::Top => {
            vertices.push(Vertex { position: [x0, y1, z0], color });
            vertices.push(Vertex { position: [x1, y1, z0], color });
            vertices.push(Vertex { position: [x1, y1, z1], color });
            vertices.push(Vertex { position: [x0, y1, z1], color });
        }
        Face::Bottom => {
            vertices.push(Vertex { position: [x0, y0, z0], color });
            vertices.push(Vertex { position: [x0, y0, z1], color });
            vertices.push(Vertex { position: [x1, y0, z1], color });
            vertices.push(Vertex { position: [x1, y0, z0], color });
        }
        Face::North => {
            vertices.push(Vertex { position: [x0, y0, z0], color });
            vertices.push(Vertex { position: [x1, y0, z0], color });
            vertices.push(Vertex { position: [x1, y1, z0], color });
            vertices.push(Vertex { position: [x0, y1, z0], color });
        }
        Face::South => {
            vertices.push(Vertex { position: [x0, y0, z1], color });
            vertices.push(Vertex { position: [x0, y1, z1], color });
            vertices.push(Vertex { position: [x1, y1, z1], color });
            vertices.push(Vertex { position: [x1, y0, z1], color });
        }
        Face::West => {
            vertices.push(Vertex { position: [x0, y0, z0], color });
            vertices.push(Vertex { position: [x0, y1, z0], color });
            vertices.push(Vertex { position: [x0, y1, z1], color });
            vertices.push(Vertex { position: [x0, y0, z1], color });
        }
        Face::East => {
            vertices.push(Vertex { position: [x1, y0, z0], color });
            vertices.push(Vertex { position: [x1, y0, z1], color });
            vertices.push(Vertex { position: [x1, y1, z1], color });
            vertices.push(Vertex { position: [x1, y1, z0], color });
        }
    }

    indices.extend_from_slice(&[
        base_index,
        base_index + 1,
        base_index + 2,
        base_index,
        base_index + 2,
        base_index + 3,
    ]);
}
