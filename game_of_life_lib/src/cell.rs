use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    row: i32,
    col: i32,
    alive: bool,
}

impl Hash for Cell {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.row.hash(state);
        self.col.hash(state);
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row && self.col == other.col
    }
}

impl Cell {
    pub fn new(row: i32, col: i32) -> Self {
        Self {
            row,
            col,
            alive: false,
        }
    }

    pub fn kill(c: &Self) -> Self {
        Self {
            alive: false,
            row: c.row,
            col: c.col,
        }
    }

    pub fn birth(c: &Self) -> Self {
        Self {
            alive: true,
            row: c.row,
            col: c.col,
        }
    }

    pub fn to_tuple(c: &Self) -> (i32, i32) {
        (c.row, c.col)
    }

    pub fn is_alive(c: &Self) -> bool {
        c.alive
    }
}

pub fn generate_live_cells() -> HashSet<Cell> {
    let mut rng = rand::thread_rng();
    let mut live_cells = HashSet::new();

    loop {
        if live_cells.len() >= 10000 {
            break;
        }

        let row = rng.gen_range(0..100);
        let col = rng.gen_range(0..100);

        let cell = Cell::birth(&Cell::new(row, col));

        live_cells.insert(cell);
    }

    live_cells
}
