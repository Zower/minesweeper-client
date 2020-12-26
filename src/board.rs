use array2d::Array2D;

use std::fmt;
use std::fmt::Formatter;

#[derive(Clone)]
struct BoardPiece {
    row: i32,
    column: i32,
    value: i32,
    mine: bool,
    neighbours: Vec<(i32, i32)>,
}

impl BoardPiece {
    fn new() -> BoardPiece {
        BoardPiece {
            row: 0,
            column: 0,
            value: 0,
            mine: false,
            neighbours: Vec::new(),
        }
    }
}
pub struct Board {
    rows: i32,
    columns: i32,
    board: Array2D<BoardPiece>,
}

impl Board {
    pub fn new() -> Board {
        Board {
            rows: 0,
            columns: 0,
            board: Array2D::filled_with(BoardPiece::new(), 0, 0),
        }
    }

    pub fn build(&mut self) {
        self.board =
            Array2D::filled_with(BoardPiece::new(), self.rows as usize, self.columns as usize);
        for row in 0..self.board.row_len() {
            for column in 0..self.board.column_len() {
                let neighbours = self.build_neighbours(row as i32, column as i32);
                self.board[(row, column)] = BoardPiece {
                    row: row as i32,
                    column: column as i32,
                    value: -1,
                    mine: false,
                    neighbours,
                }
            }
        }
    }

    pub fn get_tile_value(&self, row: usize, column: usize) -> i32 {
        self.board[(row, column)].value
    }

    pub fn get_neighbours(&self, row: usize, column: usize) -> Vec<(i32, i32)> {
        self.board[(row, column)].neighbours.clone()
    }

    pub fn mark_mine(&mut self, row: usize, column: usize) {
        self.board[(row as usize, column as usize)].mine = true;
    }

    pub fn set_value(&mut self, row: usize, column: usize, value: i32) {
        self.board[(row, column)].value = value;
    }

    pub fn set_size(&mut self, rows: i32, columns: i32) {
        println!("Setting size to: {}, {}", rows, columns);
        self.rows = rows;
        self.columns = columns;
    }

    fn build_neighbours(&self, row: i32, column: i32) -> Vec<(i32, i32)> {
        let mut init_neighbours: Vec<(i32, i32)> = Vec::new();

        //8-connectivity
        init_neighbours.push((row - 1, column + 1));
        init_neighbours.push((row - 1, column));
        init_neighbours.push((row - 1, column - 1));
        init_neighbours.push((row, column + 1));
        init_neighbours.push((row, column - 1));
        init_neighbours.push((row + 1, column + 1));
        init_neighbours.push((row + 1, column));
        init_neighbours.push((row + 1, column - 1));

        let mut neighbours: Vec<(i32, i32)> = Vec::new();

        for neighbour in 0..init_neighbours.len() {
            if !(init_neighbours[neighbour].0 < 0
                || init_neighbours[neighbour].0 > self.rows - 1
                || init_neighbours[neighbour].1 < 0
                || init_neighbours[neighbour].1 > self.columns - 1)
            // Only add if not out of boar
            {
                neighbours.push(init_neighbours[neighbour]);
            }
        }
        neighbours
    }
}

impl std::fmt::Debug for BoardPiece {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nX: {} Y: {} Value: {} Neighbours: {:?}",
            self.row, self.column, self.value, self.neighbours
        )
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut board_as_string: String = String::from(" ");
        for i in 0..self.board.column_len() {
            board_as_string.push_str(&i.to_string());
            board_as_string.push(' ');
        }
        board_as_string.push_str("\n");
        for row in 0..self.board.row_len() {
            for column in 0..self.board.column_len() {
                let element: &BoardPiece = &self.board[(row, column)];

                if element.column == 0 {
                    board_as_string.push('|');
                }

                if element.mine {
                    board_as_string.push('*');
                } else if element.value == -1 {
                    board_as_string.push('X')
                } else {
                    board_as_string.push_str(&element.value.to_string()[..])
                }

                board_as_string.push('|');

                if element.column == self.board.row_len() as i32 - 1 {
                    board_as_string.push(' ');
                    board_as_string.push_str(&row.to_string());
                    board_as_string.push_str("\n");
                }
            }
        }
        write!(
            f,
            "\nRows: {} Columns: {} State:\n{}",
            self.rows, self.columns, board_as_string
        )
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nX: {} Y: {}\nBoard: {:?}",
            self.rows, self.columns, self.board
        )
    }
}
