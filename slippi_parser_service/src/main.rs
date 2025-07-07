use anyhow::Result;
use clap::Parser;
use peppi::io::slippi::read;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "slippi_parser_service")]
#[command(about = "A fast Slippi replay file parser using peppi")]
struct Args {
    /// Path to the Slippi replay file (.slp)
    #[arg(short, long)]
    file: PathBuf,
    
    /// Output format (json, text)
    #[arg(short, long, default_value = "json")]
    format: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("Starting Slippi parser service");
    info!("Parsing file: {:?}", args.file);
    
    // Parse the Slippi file
    match parse_slippi_file(&args.file).await {
        Ok(game_data) => {
            match args.format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&game_data)?;
                    println!("{}", json);
                }
                "text" => {
                    println!("Game Data:");
                    println!("  Players: {}", game_data.player_count);
                    println!("  Duration: {} frames", game_data.duration_frames);
                    println!("  Stage: {:?}", game_data.stage);
                }
                _ => {
                    error!("Unknown format: {}", args.format);
                    return Err(anyhow::anyhow!("Unknown format"));
                }
            }
        }
        Err(e) => {
            error!("Failed to parse Slippi file: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

#[derive(serde::Serialize)]
struct GameData {
    player_count: usize,
    duration_frames: u32,
    stage: String,
    players: Vec<PlayerData>,
}

#[derive(serde::Serialize)]
struct PlayerData {
    port: u8,
    character: String,
    stocks: u8,
    costume: u8,
    team: Option<String>,
}

async fn parse_slippi_file(file_path: &PathBuf) -> Result<GameData> {
    info!("Reading Slippi file from: {:?}", file_path);
    
    // Parse with peppi using the correct API
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let game = read(&mut reader, None)?;
    
    info!("Successfully parsed Slippi replay");
    
    // Debug: examine frame structure
    info!("Frame count: {}", game.frames.len());
    
    // Get first frame to examine structure
    if let Some(first_frame) = game.frames.get(0) {
        info!("Examining first frame structure...");
        info!("Frame has {} ports", first_frame.ports.len());
        
        for (port_idx, port) in first_frame.ports.iter().enumerate() {
            if let Some(port_data) = port {
                info!("Port {} has data available", port_idx);
                
                // Look at leader data (main character)
                if let Some(leader) = &port_data.leader {
                    info!("Port {} leader action state: {:?}", port_idx, leader.action_state);
                    info!("Port {} leader buttons: {:?}", port_idx, leader.buttons);
                }
            }
        }
    }
    
    // Extract basic game information
    let game_data = GameData {
        player_count: game.start.players.len(),
        duration_frames: game.frames.len() as u32,
        stage: format!("{:?}", game.start.stage),
        players: game.start.players.iter().map(|player| {
            PlayerData {
                port: player.port.into(),
                character: format!("{:?}", player.character),
                stocks: player.stocks,
                costume: player.costume,
                team: player.team.map(|t| format!("{:?}", t)),
            }
        }).collect(),
    };
    
    info!("Extracted game data: {} players, {} frames", 
          game_data.player_count, game_data.duration_frames);
    
    Ok(game_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slippi_file_structure() {
        // This test verifies that our data structures are correctly defined
        let game_data = GameData {
            player_count: 2,
            duration_frames: 1000,
            stage: "Battlefield".to_string(),
            players: vec![
                PlayerData {
                    port: 1,
                    character: "Fox".to_string(),
                    stocks: 4,
                    costume: 0,
                    team: None,
                },
                PlayerData {
                    port: 2,
                    character: "Falco".to_string(),
                    stocks: 4,
                    costume: 1,
                    team: None,
                },
            ],
        };

        // Test serialization
        let json = serde_json::to_string(&game_data).unwrap();
        assert!(json.contains("Fox"));
        assert!(json.contains("Falco"));
        assert!(json.contains("Battlefield"));
        assert_eq!(game_data.player_count, 2);
        assert_eq!(game_data.duration_frames, 1000);
    }
}
