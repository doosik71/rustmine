use crate::world::Block;

pub struct Hotbar {
    slots: [Block; 5],
    selected: usize,
}

impl Hotbar {
    pub fn new() -> Self {
        Self {
            slots: [Block::Grass, Block::Dirt, Block::Stone, Block::Air, Block::Air],
            selected: 0,
        }
    }

    pub fn set_selected(&mut self, index: usize) {
        if index < self.slots.len() {
            self.selected = index;
        }
    }

    pub fn current_block(&self) -> Block {
        self.slots[self.selected]
    }

    #[allow(dead_code)]
    pub fn selected_index(&self) -> usize {
        self.selected
    }
}
