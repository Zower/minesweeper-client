use std::fmt;
use std::fmt::Formatter;

use minesweeper::minesweeper_client::MinesweeperClient;
use minesweeper::*;

pub mod minesweeper {
    tonic::include_proto!("proto");
}
struct Game {
    level_id: String,
    rows: i32,
    columns: i32,
    mines: i32,
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

        let test = self.click(client, 0, 0).await?;
        println!("{:?}", test);

        Ok(())
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
