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
    x: i8,
    y: i8,
    value: i32,
}

impl std::fmt::Debug for BoardPiece {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\nX: {} Y: {} Value: {}", self.x, self.y, self.value)
    }
}

impl BoardPiece {
    fn new() -> BoardPiece {
        BoardPiece {
            x: 0,
            y: 0,
            value: 0,
        }
    }
}

struct Game {
    level_id: String,
    rows: i32,
    columns: i32,
    mines: i32,
    board: Array2D<BoardPiece>,
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

                if element.y == self.board.row_len() as i8 - 1 {
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
            "Rows: {}\nColumns: {}\nMines: {}\nBoard:\n{:?}",
            self.rows, self.columns, self.mines, self.board
        )
    }
}

impl Game {
    async fn init(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.start_game(client, &self.level_id).await?;

        println!("Got response..\nSending id and getting game board");
        let game_data = response.into_inner();

        println!("Got response..");
        self.rows = game_data.rows;
        self.columns = game_data.columns;
        self.mines = game_data.mines;
        self.board = self.build_board();

        Ok(())
    }

    fn build_board(&mut self) -> Array2D<BoardPiece> {
        println!("Building board..");
        let mut board =
            Array2D::filled_with(BoardPiece::new(), self.rows as usize, self.columns as usize);

        for x in 0..board.row_len() {
            for y in 0..board.column_len() {
                board[(x, y)] = BoardPiece {
                    x: x as i8,
                    y: y as i8,
                    value: -1,
                }
            }
        }
        board
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

    async fn click(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
        row: i32,
        column: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let position = minesweeper::Position { column, row };

        let request = tonic::Request::new(ClickRequest {
            level_id: self.level_id.clone(),
            tile: Some(position),
        });

        let response: tonic::Response<ClickResponse> = client.click(request).await?;

        let r = response.into_inner();

        self.board[(row as usize, column as usize)].value = r.value;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MinesweeperClient::connect("http://minesweeper:1989").await?;
    let mut game = new_id(&mut client).await?;

    game.init(&mut client).await?;

    println!("{}", game);

    game.click(&mut client, 0, 0).await?;

    println!("{}", game);

    Ok(())
}

async fn new_id(
    client: &mut MinesweeperClient<tonic::transport::Channel>,
) -> Result<Game, Box<dyn std::error::Error>> {
    println!("Getting id..");
    let request = tonic::Request::new(NewGameRequest {});
    let response: tonic::Response<NewGameResponse> = client.new_game(request).await?;

    let game = Game {
        level_id: response.into_inner().level_id,
        rows: 0,
        columns: 0,
        mines: 0,
        board: Array2D::filled_with(BoardPiece::new(), 0, 0),
    };

    Ok(game)
}
