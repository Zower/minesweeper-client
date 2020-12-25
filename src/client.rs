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
    value: i8,
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
impl Game {
    async fn init(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.start_game(client, &self.level_id).await?;

        let game_data = response.into_inner();

        println!("{:?}", self.board);

        self.columns = game_data.columns;
        self.mines = game_data.mines;
        self.board = self.build_board();

        println!("{:?}", self.board);

        Ok(())
    }

    fn build_board(&mut self) -> Array2D<BoardPiece> {
        let board = Array2D::filled_with(BoardPiece::new(), 2, 2);
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
        &self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
        row: i32,
        column: i32,
    ) -> Result<tonic::Response<ClickResponse>, Box<dyn std::error::Error>> {
        // Make this function modify the game representation, not return response object
        let position = minesweeper::Position { column, row };

        let request = tonic::Request::new(ClickRequest {
            level_id: self.level_id.clone(),
            tile: Some(position),
        });

        let response: tonic::Response<ClickResponse> = client.click(request).await?;

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MinesweeperClient::connect("http://minesweeper:1989").await?;

    let mut game = new_id(&mut client).await?;

    // let response = new_id(&mut client).await?;
    // let id = response.into_inner().level_id;

    game.init(&mut client).await?;

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
        board: Array2D::filled_with(BoardPiece::new(), 0, 0),
    };

    Ok(game)
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rows: {}\nColumns: {}\nMines: {}",
            self.rows, self.columns, self.mines
        )
    }
}

impl std::fmt::Debug for BoardPiece {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "X: {} Y: {} Value: {}", self.x, self.y, self.value)
    }
}
