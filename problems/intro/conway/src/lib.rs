#![forbid(unsafe_code)]
#[derive(Clone, PartialEq, Eq)]
pub struct Grid<T> {
    rows: usize,
    cols: usize,
    grid: Vec<T>,
}

impl<T: Clone + Default> Grid<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            grid: vec![T::default(); rows * cols],
        }
    }

    pub fn from_slice(grid: &[T], rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            grid: grid.to_vec(),
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    pub fn get(&self, row: usize, col: usize) -> &T {
        &self.grid[row * self.cols + col]
    }

    pub fn set(&mut self, value: T, row: usize, col: usize) {
        self.grid[row * self.cols + col] = value;
    }

    pub fn neighbours(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::<(usize, usize)>::new();

        let row_start = if row == 0 { 0 } else { row - 1 };
        let row_end = if row == self.rows - 1 {
            self.rows - 1
        } else {
            row + 1
        };

        let col_start = if col == 0 { 0 } else { col - 1 };
        let col_end = if col == self.cols - 1 {
            self.cols - 1
        } else {
            col + 1
        };

        for y in row_start..=row_end {
            for x in col_start..=col_end {
                if (y, x) != (row, col) {
                    result.push((y, x));
                }
            }
        }

        result
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Dead,
    Alive,
}

impl Default for Cell {
    fn default() -> Self {
        Self::Dead
    }
}

#[derive(PartialEq, Eq)]
pub struct GameOfLife {
    grid: Grid<Cell>,
}

impl GameOfLife {
    pub fn from_grid(grid: Grid<Cell>) -> Self {
        Self { grid }
    }

    pub fn get_grid(&self) -> &Grid<Cell> {
        &self.grid
    }

    pub fn step(&mut self) {
        let mut alives = vec![0usize; self.grid.rows * self.grid.cols];

        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                alives[y * self.grid.cols + x] = self.count_alive(&(y, x));
            }
        }

        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                let alive_count = alives[y * self.grid.cols + x];

                match self.grid.get(y, x) {
                    Cell::Alive => {
                        self.process_alive(&(y, x), alive_count);
                    }

                    Cell::Dead => {
                        self.process_dead(&(y, x), alive_count);
                    }
                }
            }
        }
    }

    pub fn count_alive(&self, cell: &(usize, usize)) -> usize {
        let neighbours = self.grid.neighbours(cell.0, cell.1);

        let mut alive_count = 0usize;

        for cell in neighbours {
            if self.grid.get(cell.0, cell.1) == &Cell::Alive {
                alive_count += 1;
            }
        }

        alive_count
    }

    pub fn process_alive(&mut self, cell: &(usize, usize), alive_count: usize) {
        match alive_count {
            0..=1 => {
                self.grid.set(Cell::Dead, cell.0, cell.1);
            }

            4.. => {
                self.grid.set(Cell::Dead, cell.0, cell.1);
            }

            _ => {}
        }
    }

    pub fn process_dead(&mut self, cell: &(usize, usize), alive_count: usize) {
        if alive_count == 3 {
            self.grid.set(Cell::Alive, cell.0, cell.1);
        }
    }
}
