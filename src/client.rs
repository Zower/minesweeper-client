use std::fmt;
use std::fmt::Formatter;

use minesweeper::minesweeper_client::MinesweeperClient;
use minesweeper::*;

use board::Board;

mod board;

pub mod minesweeper {
    tonic::include_proto!("proto");
}

struct Game {
    level_id: String,
    mines: usize,
    found_mines: Vec<minesweeper::Position>,
    board: Board,
}

impl Game {
    pub async fn init(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.start_game(client, &self.level_id).await?;
        let game_data = response.into_inner();

        self.mines = game_data.mines as usize;
        self.board.set_size(game_data.rows, game_data.columns);
        self.board.build();

        Ok(())
    }

    async fn start_game(
        &self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
        level_id: &String,
    ) -> Result<tonic::Response<StartLevelResponse>, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(StartLevelRequest {
            level_id: level_id.clone(),
        });

        let response: tonic::Response<StartLevelResponse> = client.start_level(request).await?;
        Ok(response)
    }

    async fn open_tile(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
        row: usize,
        column: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let position = minesweeper::Position {
            column: column as i32,
            row: row as i32,
        };

        let request = tonic::Request::new(ClickRequest {
            level_id: self.level_id.clone(),
            tile: Some(position),
        });

        let response: tonic::Response<ClickResponse> = client.click(request).await?;
        let r = response.into_inner();

        self.board.set_value(row, column, r.value);

        Ok(())
    }

    async fn mark_mine(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
        row: usize,
        column: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.board.mark_mine(row, column);
        self.found_mines.push(minesweeper::Position {
            row: row as i32,
            column: column as i32,
        });

        if self.found_mines.len() == self.mines {
            self.send_solve(client).await?;
        }
        Ok(())
    }

    pub async fn solve(
        &mut self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut safe_stack = Vec::new();
        let mut unsafe_stack = Vec::new();

        safe_stack.push((0 as i32, 0 as i32));

        let mut row: i32;
        let mut column: i32;

        while let Some(top) = safe_stack.pop() {
            self.open_tile(client, top.0 as usize, top.1 as usize)
                .await?;

            row = top.0;
            column = top.1;

            let value = self.board.get_tile_value(row as usize, column as usize);
            if value == 0 {
                // No mines nearby
                for neighbour in self.board.get_neighbours(row as usize, column as usize) {
                    // Then go over all neighbours
                    if self
                        .board
                        .get_tile_value(neighbour.0 as usize, neighbour.1 as usize)
                        == -1
                        && !safe_stack.contains(&neighbour)
                    {
                        // Add to stack if unopened and not in stack
                        safe_stack.push(neighbour);
                    }
                }
            } else if value > 0 {
                // Not safe to click
                unsafe_stack.push((row, column));
            }
        }

        while let Some(top) = unsafe_stack.pop() {
            let mut i = 0;
            let mut index = (0, 0);
            //println!("Neighbours:");
            for neighbour in self.board.get_neighbours(top.0 as usize, top.1 as usize) {
                //println!("{}, {}", neighbour.0, neighbour.1);
                if self
                    .board
                    .get_tile_value(neighbour.0 as usize, neighbour.1 as usize)
                    == -1
                {
                    i += 1;
                    index = (neighbour.0, neighbour.1);
                }
            }
            println!("Tile: {}, {}, Adjacent mines: {}", top.0, top.1, i);
            if i == 1 {
                println!("Identified {:?} as mine", index);
                self.mark_mine(client, index.0 as usize, index.1 as usize)
                    .await?;
                unsafe_stack.clear();
            }
        }

        Ok(())
    }

    async fn send_solve(
        &self,
        client: &mut MinesweeperClient<tonic::transport::Channel>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(SolveLevelRequest {
            level_id: self.level_id.clone(),
            mines: self.found_mines.clone(),
        });

        let response: tonic::Response<SolveLevelResponse> = client.solve_level(request).await?;
        let r = response.into_inner();

        // create new game

        println!("{:?}", r);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MinesweeperClient::connect("http://minesweeper:1989").await?;
    let mut game = new_id(&mut client).await?;

    game.init(&mut client).await?;
    game.solve(&mut client).await?;

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
        mines: 0,
        found_mines: Vec::new(),
        board: Board::new(),
    };

    Ok(game)
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mines: {}, Found mines: {:?} Board:\n{}",
            self.mines, self.found_mines, self.board,
        )
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mines: {}\n Found mines: {:?}\nBoard:\n{:?}",
            self.mines, self.found_mines, self.board
        )
    }
}
