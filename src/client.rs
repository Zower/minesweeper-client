use std::fmt;
use std::fmt::Formatter;

use array2d::Array2D;

use minesweeper::minesweeper_client::MinesweeperClient;
use minesweeper::*;

pub mod minesweeper {
    tonic::include_proto!("proto");
}
#[derive(Clone)]
struct BoardPiece {
    x: i32,
    y: i32,
    value: i32,
    mine: bool,
    neighbours: Vec<(i32, i32)>,
}

impl BoardPiece {
    fn new() -> BoardPiece {
        BoardPiece {
            x: 0,
            y: 0,
            value: 0,
            mine: false,
            neighbours: Vec::new(),
        }
    }
}

struct Board {
    rows: i32,
    columns: i32,
    board: Array2D<BoardPiece>,
}

struct Game {
    level_id: String,
    rows: i32,
    columns: i32,
    mines: i32,
    found_mines: Vec<minesweeper::Position>,
    board: Array2D<BoardPiece>,
}

impl Game {
    async fn init(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.start_game(client, &self.level_id).await?;
        let game_data = response.into_inner();

        self.rows = game_data.rows;
        self.columns = game_data.columns;
        self.mines = game_data.mines;
        self.board = self.build_board();

        Ok(())
    }

    fn build_board(&mut self) -> Array2D<BoardPiece> {
        let mut board =
            Array2D::filled_with(BoardPiece::new(), self.rows as usize, self.columns as usize);

        for x in 0..board.row_len() {
            for y in 0..board.column_len() {
                let neighbours = self.build_neighbours(x as i32, y as i32);
                board[(x, y)] = BoardPiece {
                    x: x as i32,
                    y: y as i32,
                    value: -1,
                    mine: false,
                    neighbours,
                }
            }
        }
        board
    }

    fn build_neighbours(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
        let mut init_neighbours: Vec<(i32, i32)> = Vec::new();

        init_neighbours.push((x - 1, y + 1));
        init_neighbours.push((x - 1, y));
        init_neighbours.push((x - 1, y - 1));
        init_neighbours.push((x, y + 1));
        init_neighbours.push((x, y - 1));
        init_neighbours.push((x + 1, y + 1));
        init_neighbours.push((x + 1, y));
        init_neighbours.push((x + 1, y - 1));

        let mut neighbours: Vec<(i32, i32)> = Vec::new();

        let row_s = self.rows - 1;
        let col_s = self.columns - 1;
        for neighbour in 0..init_neighbours.len() {
            if !(init_neighbours[neighbour].0 < 0
                || init_neighbours[neighbour].0 > row_s
                || init_neighbours[neighbour].1 < 0
                || init_neighbours[neighbour].1 > col_s)
            {
                neighbours.push(init_neighbours[neighbour]);
            }
        }

        neighbours
    }

    async fn start_game(
        &self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
        level_id: &String,
    ) -> Result<tonic::Response<StartLevelResponse>, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(StartLevelRequest {
            level_id: level_id.into(),
        });

        let response: tonic::Response<StartLevelResponse> = client.start_level(request).await?;
        Ok(response)
    }

    async fn open_tile(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
        row: i32,
        column: i32,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        println!("Opening tile {}, {}", row, column);

        let position = minesweeper::Position { column, row };

        let request = tonic::Request::new(ClickRequest {
            level_id: self.level_id.clone(),
            tile: Some(position),
        });

        let response: tonic::Response<ClickResponse> = client.click(request).await?;
        let r = response.into_inner();

        println!("{:?}", r);

        self.board[(row as usize, column as usize)].value = r.value;

        Ok(r.value)
    }

    async fn mark_mine(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
        x: i32,
        y: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.board[(x as usize, y as usize)].mine = true;
        self.found_mines
            .push(minesweeper::Position { row: x, column: y });

        if self.found_mines.len() == self.mines as usize {
            self.solve(client).await?;
        }
        Ok(())
    }

    async fn solve(
        &self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(SolveLevelRequest {
            level_id: self.level_id.clone(),
            mines: self.found_mines.clone(),
        });

        println!("{:?}", self.found_mines);

        let response: tonic::Response<SolveLevelResponse> = client.solve_level(request).await?;

        let r = response.into_inner();

        println!("{:?}", r);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MinesweeperClient::connect("http://minesweeper:1989").await?;
    let mut game = new_id(&mut client).await?;

    game.init(&mut client).await?;

    let mut safe_stack = Vec::new();
    let mut unsafe_stack = Vec::new();
    safe_stack.push((0, 0));

    let mut pos;
    println!("Entering while..");
    while let Some(top) = safe_stack.pop() {
        game.open_tile(&mut client, top.0, top.1).await?;
        pos = (top.0 as usize, top.1 as usize);

        if game.board[pos].value == 0 {
            // No mines nearby
            for neighbour in game.board[pos].neighbours.clone() {
                // Then go over all neighbours
                if game.board[(neighbour.0 as usize, neighbour.1 as usize)].value == -1
                    && !safe_stack.contains(&neighbour)
                {
                    // Add to stack if unopened and not in stack
                    println!("Pushing: {}, {}", neighbour.0, neighbour.1);
                    safe_stack.push(neighbour);
                }
            }
        } else if game.board[pos].value > 0 {
            println!("{:?}, {}", pos, game.board[pos].value);
            unsafe_stack.push(pos);
        }
    }

    println!("{}", game);

    while let Some(top) = unsafe_stack.pop() {
        println!("Opened: {:?}", top);
        let mut i = 0;
        let mut index = (0, 0);
        println!("Neighbours:");
        for neighbour in game.board[top].neighbours.clone() {
            println!("{}, {}", neighbour.0, neighbour.1);
            if game.board[(neighbour.0 as usize, neighbour.1 as usize)].value == -1 {
                i += 1;
                index = (neighbour.0, neighbour.1);
            }
        }
        if i == 1 {
            println!("Identified {:?} as mine", index);
            game.mark_mine(&mut client, index.0, index.1).await?;
            unsafe_stack.clear();
        }
    }

    println!("{}", game);

    Ok(())
}

async fn new_id(
    client: &mut MinesweeperClient<tonic::transport::Channel>,
) -> Result<Game, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(NewGameRequest {});
    let response: tonic::Response<NewGameResponse> = client.new_game(request).await?;

    let game = Game {
        level_id: response.into_inner().level_id,
        rows: 0,
        columns: 0,
        mines: 0,
        found_mines: Vec::new(),
        board: Array2D::filled_with(BoardPiece::new(), 0, 0),
    };

    Ok(game)
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut board_as_string: String = String::from("");
        for x in 0..self.board.row_len() {
            for y in 0..self.board.column_len() {
                let element: &BoardPiece = &self.board[(x, y)];

                if element.y == 0 {
                    board_as_string.push('|');
                }

                if element.value == -1 {
                    board_as_string.push('X')
                } else {
                    board_as_string.push_str(&element.value.to_string()[..])
                }

                board_as_string.push('|');

                if element.y == self.board.row_len() as i32 - 1 {
                    board_as_string.push_str("\n");
                }
            }
        }
        write!(
            f,
            "Rows: {}, Columns: {}, Mines: {}, Board:\n{}",
            self.rows, self.columns, self.mines, board_as_string,
        )
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rows: {}\nColumns: {}\nMines: {}\n Found mines: {:?}\nBoard:\n{:?}",
            self.rows, self.columns, self.mines, self.found_mines, self.board
        )
    }
}

impl std::fmt::Debug for BoardPiece {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nX: {} Y: {} Value: {} Neighbours: {:?}",
            self.x, self.y, self.value, self.neighbours
        )
    }
}
