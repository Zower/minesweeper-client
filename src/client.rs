use minesweeper::minesweeper_client::MinesweeperClient;
use minesweeper::*;

pub mod minesweeper{
    tonic::include_proto!("proto");
}

struct Game {
    level_id: String,
    rows: i32,
    columns: i32,
    mines: i32
}
impl Game {

    async fn init(&mut self, client: &mut MinesweeperClient<tonic::transport::Channel>) -> Result<(), Box<dyn std::error::Error>> {
        println!("Rows: {}", self.rows);
        println!("Columns: {}", self.columns);
        println!("Mines: {}", self.mines);

        let response = start_game(client,self.level_id).await?;

        let game_data = &response.into_inner();

        self.rows = game_data.rows;
        self.columns = game_data.columns;

        Ok(())
    }

    async fn start_game(&self, client: &mut MinesweeperClient<tonic::transport::Channel>, level_id: String) -> Result<tonic::Response<StartLevelResponse>, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(StartLevelRequest{
            level_id: level_id.into(),
        });
    
        let response: tonic::Response<StartLevelResponse> = client.start_level(request).await?;
        Ok(response)
    }

    async fn click(client: &mut MinesweeperClient<tonic::transport::Channel>, level_id: String, row: i32, column: i32) -> Result<tonic::Response<ClickResponse>, Box<dyn std::error::Error>> {
        let position = minesweeper::Position{column, row};
    
        let request = tonic::Request::new(ClickRequest{
            level_id,
            tile: Some(position)
        });
    
        let response: tonic::Response<ClickResponse> = client.click(request).await?;
        
        Ok(response)
    }
    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut client = MinesweeperClient::connect("http://minesweeper:1989").await?;

    let response = new_id(&mut client).await?;
    let id = response.into_inner().level_id;

    let game = Game{level_id: id, rows: 0, columns: 0, mines: 0};

    game.init();

    // let response = start_game(&mut client, id.clone()).await?;

    // println!("RESPONSE={:?}", response);

    // let game_response = response.into_inner();

    // let game = Game{level_id: id, rows: game_response.rows, columns: game_response.columns, mines: game_response.mines};

    // game.init();

    // let response = click(&mut client, game.level_id, 3, 2).await?;

    // println!("RESPONSE={:?}", response);

    Ok(())
}

async fn new_id(client: &mut MinesweeperClient<tonic::transport::Channel>) -> Result<tonic::Response<NewGameResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(NewGameRequest{});
    let response: tonic::Response<NewGameResponse> = client.new_game(request).await?;

    Ok(response)
}

async fn start_game(client: &mut MinesweeperClient<tonic::transport::Channel>, level_id: String) -> Result<tonic::Response<StartLevelResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(StartLevelRequest{
        level_id: level_id.into(),
    });

    let response: tonic::Response<StartLevelResponse> = client.start_level(request).await?;
    Ok(response)
}

async fn click(client: &mut MinesweeperClient<tonic::transport::Channel>, level_id: String, row: i32, column: i32) -> Result<tonic::Response<ClickResponse>, Box<dyn std::error::Error>> {
    let position = minesweeper::Position{column, row};

    let request = tonic::Request::new(ClickRequest{
        level_id,
        tile: Some(position)
    });

    let response: tonic::Response<ClickResponse> = client.click(request).await?;
    
    Ok(response)
}