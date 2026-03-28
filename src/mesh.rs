use crate::world::{Block, Chunk, CHUNK_SIZE, WORLD_HEIGHT};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
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
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn build_chunk_mesh<F>(chunk: &Chunk, base_x: i32, base_z: i32, is_air: &F) -> Mesh
where
    F: Fn(i32, i32, i32) -> bool,
{
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    append_chunk_mesh(&mut vertices, &mut indices, chunk, base_x, base_z, is_air);

    Mesh { vertices, indices }
}

// Lighter mesh: only top 3 layers (surface + sides) to avoid huge vertex counts.
pub fn build_chunk_surface_mesh<F>(
    chunk: &Chunk,
    base_x: i32,
    base_z: i32,
    is_air: &F,
) -> Mesh
where
    F: Fn(i32, i32, i32) -> bool,
{
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let height = chunk.height_at(x, z) as i32;
            if height <= 0 {
                continue;
            }
            let top = height - 1;
            let bottom = (top - 2).max(0);
            for y in bottom..=top {
                let block = chunk.block_at(x, y as usize, z);
                if block == Block::Air {
                    continue;
                }

                let wx = base_x + x as i32;
                let wy = y as i32;
                let wz = base_z + z as i32;

                if is_air(wx, wy + 1, wz) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::Top,
                        wx,
                        wy,
                        wz,
                        shade(block, 1.0),
                    );
                }
                if is_air(wx, wy, wz - 1) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::North,
                        wx,
                        wy,
                        wz,
                        shade(block, 0.8),
                    );
                }
                if is_air(wx, wy, wz + 1) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::South,
                        wx,
                        wy,
                        wz,
                        shade(block, 0.8),
                    );
                }
                if is_air(wx - 1, wy, wz) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::West,
                        wx,
                        wy,
                        wz,
                        shade(block, 0.75),
                    );
                }
                if is_air(wx + 1, wy, wz) {
                    add_face(
                        &mut vertices,
                        &mut indices,
                        Face::East,
                        wx,
                        wy,
                        wz,
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
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.0, 0.5, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];
    let indices = vec![0, 1, 2];
    Mesh { vertices, indices }
}

pub fn build_crosshair_mesh() -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // horizontal bar
    add_rect(
        &mut vertices,
        &mut indices,
        [-0.03, -0.002, 0.0],
        [0.03, 0.002, 0.0],
        [1.0, 1.0, 1.0, 1.0],
    );

    // vertical bar
    add_rect(
        &mut vertices,
        &mut indices,
        [-0.002, -0.03, 0.0],
        [0.002, 0.03, 0.0],
        [1.0, 1.0, 1.0, 1.0],
    );

    Mesh { vertices, indices }
}

pub fn build_hud_mesh(fps: u32, loading: Option<f32>) -> Mesh {
    let mut meshes = Vec::new();

    // Optional loading overlay
    if let Some(p) = loading {
        let mut overlay = Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        add_rect(
            &mut overlay.vertices,
            &mut overlay.indices,
            [-1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.05, 0.06, 0.08, 0.6],
        );
        // reduce alpha by blending; color still used
        meshes.push(overlay);

        let text = format!("LOADING {}%", (p * 100.0).round() as u32);
        meshes.push(build_text_mesh(&text, -0.8, 0.4, 0.06, [1.0, 1.0, 1.0, 1.0]));
    }

    // FPS top-right
    let fps_text = format!("FPS: {}", fps);
    meshes.push(build_text_mesh(&fps_text, 0.32, 0.9, 0.012, [1.0, 1.0, 1.0, 1.0]));

    // Crosshair center
    meshes.push(build_crosshair_mesh());

    merge_meshes(&meshes)
}

pub fn merge_meshes(meshes: &[Mesh]) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for mesh in meshes {
        let base = vertices.len() as u32;
        vertices.extend_from_slice(&mesh.vertices);
        indices.extend(mesh.indices.iter().map(|i| i + base));
    }
    Mesh { vertices, indices }
}

pub fn build_text_mesh(text: &str, x: f32, y: f32, scale: f32, color: [f32; 4]) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut cursor_x = x;
    let cursor_y = y;

    for ch in text.chars() {
        if ch == ' ' {
            cursor_x += scale * 6.0;
            continue;
        }
        if let Some(bitmap) = font_bitmap(ch) {
            for (row, mask) in bitmap.iter().enumerate() {
                for col in 0..5 {
                    if (mask & (1 << (4 - col))) != 0 {
                        let px = cursor_x + col as f32 * scale;
                        let py = cursor_y - row as f32 * scale;
                        add_rect(
                            &mut vertices,
                            &mut indices,
                            [px, py - scale, 0.0],
                            [px + scale, py, 0.0],
                            color,
                        );
                    }
                }
            }
        }
        cursor_x += scale * 6.0;
    }

    Mesh { vertices, indices }
}

fn font_bitmap(ch: char) -> Option<[u8; 7]> {
    match ch.to_ascii_uppercase() {
        '0' => Some([0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110]),
        '1' => Some([0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
        '2' => Some([0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111]),
        '3' => Some([0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110]),
        '4' => Some([0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010]),
        '5' => Some([0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110]),
        '6' => Some([0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110]),
        '7' => Some([0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000]),
        '8' => Some([0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
        '9' => Some([0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100]),
        'F' => Some([0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b10000]),
        'P' => Some([0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000]),
        'S' => Some([0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110]),
        'L' => Some([0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111]),
        'O' => Some([0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        'A' => Some([0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
        'D' => Some([0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110]),
        'I' => Some([0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
        'N' => Some([0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001]),
        'G' => Some([0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110]),
        ':' => Some([0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000]),
        '%' => Some([0b11001, 0b11010, 0b00100, 0b01000, 0b10110, 0b00110, 0b00000]),
        _ => None,
    }
}

fn add_rect(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    min: [f32; 3],
    max: [f32; 3],
    color: [f32; 4],
) {
    let base = vertices.len() as u32;
    vertices.push(Vertex {
        position: [min[0], min[1], min[2]],
        color,
    });
    vertices.push(Vertex {
        position: [max[0], min[1], min[2]],
        color,
    });
    vertices.push(Vertex {
        position: [max[0], max[1], max[2]],
        color,
    });
    vertices.push(Vertex {
        position: [min[0], max[1], min[2]],
        color,
    });
    indices.extend_from_slice(&[
        base,
        base + 1,
        base + 2,
        base,
        base + 2,
        base + 3,
    ]);
}

fn block_color(block: Block) -> [f32; 4] {
    match block {
        Block::Grass => [0.3, 0.7, 0.3, 1.0],
        Block::Dirt => [0.55, 0.35, 0.2, 1.0],
        Block::Stone => [0.5, 0.5, 0.5, 1.0],
        Block::Air => [0.7, 0.85, 1.0, 1.0],
    }
}

fn shade(block: Block, factor: f32) -> [f32; 4] {
    let base = block_color(block);
    [base[0] * factor, base[1] * factor, base[2] * factor, base[3]]
}

pub fn append_chunk_mesh<F>(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    chunk: &Chunk,
    base_x: i32,
    base_z: i32,
    is_air: &F,
) where
    F: Fn(i32, i32, i32) -> bool,
{
    for y in 0..WORLD_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.block_at(x, y, z);
                if block == Block::Air {
                    continue;
                }

                let wx = base_x + x as i32;
                let wy = y as i32;
                let wz = base_z + z as i32;

                // Top
                if is_air(wx, wy + 1, wz) {
                    add_face(
                        vertices,
                        indices,
                        Face::Top,
                        wx,
                        wy,
                        wz,
                        shade(block, 1.0),
                    );
                }
                // Bottom
                if is_air(wx, wy - 1, wz) {
                    add_face(
                        vertices,
                        indices,
                        Face::Bottom,
                        wx,
                        wy,
                        wz,
                        shade(block, 0.6),
                    );
                }
                // North (-z)
                if is_air(wx, wy, wz - 1) {
                    add_face(
                        vertices,
                        indices,
                        Face::North,
                        wx,
                        wy,
                        wz,
                        shade(block, 0.8),
                    );
                }
                // South (+z)
                if is_air(wx, wy, wz + 1) {
                    add_face(
                        vertices,
                        indices,
                        Face::South,
                        wx,
                        wy,
                        wz,
                        shade(block, 0.8),
                    );
                }
                // West (-x)
                if is_air(wx - 1, wy, wz) {
                    add_face(
                        vertices,
                        indices,
                        Face::West,
                        wx,
                        wy,
                        wz,
                        shade(block, 0.75),
                    );
                }
                // East (+x)
                if is_air(wx + 1, wy, wz) {
                    add_face(
                        vertices,
                        indices,
                        Face::East,
                        wx,
                        wy,
                        wz,
                        shade(block, 0.75),
                    );
                }
            }
        }
    }
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
    x: i32,
    y: i32,
    z: i32,
    color: [f32; 4],
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
