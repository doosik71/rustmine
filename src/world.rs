#[derive(Clone, Copy, Debug)]
pub enum Biome {
    Plains,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
}

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_HEIGHT: usize = 384;

#[derive(Clone, Copy, Debug)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

pub struct World {
    seed: u64,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
        Chunk::generate(self.seed, pos)
    }

pub fn height_at_world(&self, world_x: i32, world_z: i32) -> u16 {
        let chunk_x = div_floor(world_x, CHUNK_SIZE as i32);
        let chunk_z = div_floor(world_z, CHUNK_SIZE as i32);
        let local_x = mod_floor(world_x, CHUNK_SIZE as i32) as usize;
        let local_z = mod_floor(world_z, CHUNK_SIZE as i32) as usize;
        let chunk = self.generate_chunk(ChunkPos {
            x: chunk_x,
            z: chunk_z,
        });
        chunk.height_at(local_x, local_z)
    }

    pub fn block_at_world(&self, world_x: i32, world_y: i32, world_z: i32) -> Block {
        if world_y < 0 || world_y >= WORLD_HEIGHT as i32 {
            return Block::Air;
        }
        let chunk_x = div_floor(world_x, CHUNK_SIZE as i32);
        let chunk_z = div_floor(world_z, CHUNK_SIZE as i32);
        let local_x = mod_floor(world_x, CHUNK_SIZE as i32) as usize;
        let local_z = mod_floor(world_z, CHUNK_SIZE as i32) as usize;
        let chunk = self.generate_chunk(ChunkPos {
            x: chunk_x,
            z: chunk_z,
        });
        chunk.block_at(local_x, world_y as usize, local_z)
    }
}

pub struct Chunk {
    pos: ChunkPos,
    biome: Biome,
    blocks: Vec<Block>,
    heightmap: [u16; CHUNK_SIZE * CHUNK_SIZE],
}

impl Chunk {
    fn generate(seed: u64, pos: ChunkPos) -> Self {
        let biome = Biome::Plains;
        let mut blocks = vec![Block::Air; CHUNK_SIZE * WORLD_HEIGHT * CHUNK_SIZE];
        let mut heightmap = [0u16; CHUNK_SIZE * CHUNK_SIZE];

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = pos.x as i64 * CHUNK_SIZE as i64 + x as i64;
                let world_z = pos.z as i64 * CHUNK_SIZE as i64 + z as i64;
                let height = generate_height(seed, world_x, world_z);
                heightmap[z * CHUNK_SIZE + x] = height as u16;

                let top = height.saturating_sub(1) as usize;
                for y in 0..=top {
                    let block = if y == top {
                        Block::Grass
                    } else if y >= top.saturating_sub(3) {
                        Block::Dirt
                    } else {
                        Block::Stone
                    };
                    let idx = index(x, y, z);
                    blocks[idx] = block;
                }
            }
        }

        Self {
            pos,
            biome,
            blocks,
            heightmap,
        }
    }

    pub fn biome(&self) -> Biome {
        self.biome
    }

    pub fn height_at(&self, x: usize, z: usize) -> u16 {
        self.heightmap[z * CHUNK_SIZE + x]
    }

    pub fn block_at(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[index(x, y, z)]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) -> bool {
        if x >= CHUNK_SIZE || y >= WORLD_HEIGHT || z >= CHUNK_SIZE {
            return false;
        }
        let idx = index(x, y, z);
        self.blocks[idx] = block;
        self.recompute_height_column(x, z);
        true
    }

    pub fn stats(&self) -> ChunkStats {
        let mut min_height = u16::MAX;
        let mut max_height = 0u16;
        let mut sum_height: u64 = 0;

        for h in self.heightmap.iter() {
            min_height = min_height.min(*h);
            max_height = max_height.max(*h);
            sum_height += *h as u64;
        }

        let mut air = 0u64;
        let mut grass = 0u64;
        let mut dirt = 0u64;
        let mut stone = 0u64;

        for block in self.blocks.iter() {
            match block {
                Block::Air => air += 1,
                Block::Grass => grass += 1,
                Block::Dirt => dirt += 1,
                Block::Stone => stone += 1,
            }
        }

        let avg_height = sum_height as f32 / (CHUNK_SIZE * CHUNK_SIZE) as f32;

        ChunkStats {
            min_height,
            max_height,
            avg_height,
            air,
            grass,
            dirt,
            stone,
        }
    }

    #[allow(dead_code)]
    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    fn recompute_height_column(&mut self, x: usize, z: usize) {
        let mut height = 0u16;
        for y in (0..WORLD_HEIGHT).rev() {
            if self.block_at(x, y, z) != Block::Air {
                height = (y + 1) as u16;
                break;
            }
        }
        self.heightmap[z * CHUNK_SIZE + x] = height;
    }
}

pub struct ChunkStats {
    pub min_height: u16,
    pub max_height: u16,
    pub avg_height: f32,
    pub air: u64,
    pub grass: u64,
    pub dirt: u64,
    pub stone: u64,
}

fn index(x: usize, y: usize, z: usize) -> usize {
    (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x
}

fn generate_height(seed: u64, world_x: i64, world_z: i64) -> u32 {
    let base = 64.0;
    let coarse = value_noise(seed, world_x, world_z, 64.0);
    let fine = value_noise(seed ^ 0x9E37, world_x, world_z, 16.0);
    let mut h = base + coarse * 12.0 + fine * 4.0;

    // simple smoothing: average with 4-neighbors
    let n1 = value_noise(seed, world_x - 1, world_z, 64.0) * 12.0
        + value_noise(seed ^ 0x9E37, world_x - 1, world_z, 16.0) * 4.0;
    let n2 = value_noise(seed, world_x + 1, world_z, 64.0) * 12.0
        + value_noise(seed ^ 0x9E37, world_x + 1, world_z, 16.0) * 4.0;
    let n3 = value_noise(seed, world_x, world_z - 1, 64.0) * 12.0
        + value_noise(seed ^ 0x9E37, world_x, world_z - 1, 16.0) * 4.0;
    let n4 = value_noise(seed, world_x, world_z + 1, 64.0) * 12.0
        + value_noise(seed ^ 0x9E37, world_x, world_z + 1, 16.0) * 4.0;

    h = (h + base + n1 + base + n2 + base + n3 + base + n4) / 5.0;

    h.round().clamp(1.0, (WORLD_HEIGHT - 1) as f32) as u32
}

fn value_noise(seed: u64, world_x: i64, world_z: i64, scale: f32) -> f32 {
    let fx = world_x as f32 / scale;
    let fz = world_z as f32 / scale;
    let x0 = fx.floor() as i64;
    let z0 = fz.floor() as i64;
    let x1 = x0 + 1;
    let z1 = z0 + 1;

    let tx = fx - x0 as f32;
    let tz = fz - z0 as f32;
    let u = smoothstep(tx);
    let v = smoothstep(tz);

    let v00 = hash2d(seed, x0, z0) as f32 / u32::MAX as f32;
    let v10 = hash2d(seed, x1, z0) as f32 / u32::MAX as f32;
    let v01 = hash2d(seed, x0, z1) as f32 / u32::MAX as f32;
    let v11 = hash2d(seed, x1, z1) as f32 / u32::MAX as f32;

    let a = lerp(v00, v10, u);
    let b = lerp(v01, v11, u);
    lerp(a, b, v)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn world_to_chunk(world_x: i32, world_z: i32) -> (ChunkPos, usize, usize) {
    let chunk_x = div_floor(world_x, CHUNK_SIZE as i32);
    let chunk_z = div_floor(world_z, CHUNK_SIZE as i32);
    let local_x = mod_floor(world_x, CHUNK_SIZE as i32) as usize;
    let local_z = mod_floor(world_z, CHUNK_SIZE as i32) as usize;
    (
        ChunkPos {
            x: chunk_x,
            z: chunk_z,
        },
        local_x,
        local_z,
    )
}

fn div_floor(a: i32, b: i32) -> i32 {
    let mut q = a / b;
    let r = a % b;
    if r != 0 && ((r > 0) != (b > 0)) {
        q -= 1;
    }
    q
}

fn mod_floor(a: i32, b: i32) -> i32 {
    let r = a % b;
    if r != 0 && ((r > 0) != (b > 0)) {
        r + b
    } else {
        r
    }
}

fn hash2d(seed: u64, x: i64, z: i64) -> u32 {
    let mut v = seed ^ (x as u64).wrapping_mul(0x9E3779B97F4A7C15);
    v = v.wrapping_add((z as u64).wrapping_mul(0xC2B2AE3D27D4EB4F));
    v ^= v >> 33;
    v = v.wrapping_mul(0xFF51AFD7ED558CCD);
    v ^= v >> 33;
    v = v.wrapping_mul(0xC4CEB9FE1A85EC53);
    v ^= v >> 33;
    (v & 0xFFFF_FFFF) as u32
}
