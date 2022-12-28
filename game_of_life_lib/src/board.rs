use crate::{cell::Cell, utility::generate_id};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Board {
    id: String,
    rows: i32,
    cols: i32,
    pub cells: HashSet<Cell>,
}

impl Board {
    pub fn new(rows: i32, cols: i32) -> Self {
        let mut cells = HashSet::new();

        for row in 0..rows {
            for col in 0..cols {
                let cell = Cell::new(row, col);
                cells.insert(cell);
            }
        }

        Self {
            id: generate_id(),
            rows,
            cols,
            cells,
        }
    }

    pub fn run_cells_transform<F>(&self, f: F) -> Self
    where
        F: Fn(&HashSet<Cell>) -> HashSet<Cell>,
    {
        let new_cells = f(&self.cells);

        //self.cells = new_cells;
        Self {
            cells: new_cells,
            id: self.id.clone(),
            rows: self.rows,
            cols: self.cols,
        }
    }

    pub fn get_cells(&self) -> Vec<Cell> {
        self.cells.clone().into_iter().collect()
    }

    pub fn take_cells(self) -> HashSet<Cell> {
        self.cells
    }
}
