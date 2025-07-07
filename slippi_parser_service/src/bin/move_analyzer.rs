use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "move_analyzer")]
#[command(about = "Analyze moves from parsed Slippi game files")]
struct Args {
    /// Path to the directory containing JSON files
    #[arg(short, long)]
    directory: PathBuf,
    
    /// Output format (json, csv, text)
    #[arg(long, default_value = "json")]
    format: String,
    
    /// Output file path (optional, defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct GameData {
    player_count: usize,
    duration_frames: u32,
    stage: String,
    players: Vec<PlayerData>,
    moves: Option<Vec<PlayerMoveData>>,
}

#[derive(Serialize, Deserialize)]
struct PlayerData {
    port: u8,
    character: String,
    stocks: u8,
    costume: u8,
    team: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PlayerMoveData {
    port: u8,
    character: String,
    moves: HashMap<String, u32>,
}

#[derive(Serialize)]
struct MoveStats {
    total_games: u32,
    players: Vec<PlayerMoveData>,
    aggregated_stats: HashMap<String, serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("Starting move analyzer");
    info!("Processing directory: {:?}", args.directory);
    
    match process_directory_for_moves(&args.directory).await {
        Ok(stats) => {
            let output = match args.format.as_str() {
                "json" => {
                    serde_json::to_string_pretty(&stats)?
                }
                "csv" => {
                    generate_csv_output(&stats)?
                }
                "text" => {
                    generate_text_output(&stats)
                }
                _ => {
                    error!("Unknown format: {}", args.format);
                    return Err(anyhow::anyhow!("Unknown format"));
                }
            };
            
            // Output to file or stdout
            if let Some(output_path) = args.output {
                fs::write(output_path, output)?;
                info!("Output saved to file");
            } else {
                println!("{}", output);
            }
        }
        Err(e) => {
            error!("Failed to process directory: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn process_directory_for_moves(directory: &PathBuf) -> Result<MoveStats> {
    let mut total_games = 0;
    let mut all_players: Vec<PlayerMoveData> = Vec::new();
    let mut aggregated_moves: HashMap<String, u32> = HashMap::new();
    
    // Read all JSON files in the directory
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(game_data) = serde_json::from_str::<GameData>(&content) {
                    total_games += 1;
                    
                    if let Some(moves) = game_data.moves {
                        for player_moves in moves {
                            // Aggregate moves
                            for (move_name, count) in &player_moves.moves {
                                let total_count = aggregated_moves.entry(move_name.clone()).or_insert(0);
                                *total_count += count;
                            }
                            
                            // Store player data
                            all_players.push(player_moves);
                        }
                    }
                }
            }
        }
    }
    
    // Create aggregated statistics
    let mut stats_map = HashMap::new();
    if let Some(most_common) = aggregated_moves.iter().max_by_key(|(_, count)| *count) {
        stats_map.insert("most_common_move".to_string(), serde_json::Value::String(most_common.0.clone()));
    }
    
    let total_moves: u32 = aggregated_moves.values().sum();
    let avg_moves_per_game = if total_games > 0 { total_moves / total_games } else { 0 };
    stats_map.insert("average_moves_per_game".to_string(), serde_json::Value::Number(avg_moves_per_game.into()));
    
    Ok(MoveStats {
        total_games,
        players: all_players,
        aggregated_stats: stats_map,
    })
}

fn generate_csv_output(stats: &MoveStats) -> Result<String> {
    let mut output = String::new();
    output.push_str("port,character,move,count\n");
    
    for player in &stats.players {
        for (move_name, count) in &player.moves {
            output.push_str(&format!("{},{},{},{}\n", player.port, player.character, move_name, count));
        }
    }
    
    Ok(output)
}

fn generate_text_output(stats: &MoveStats) -> String {
    let mut output = String::new();
    output.push_str(&format!("Move Statistics Summary\n"));
    output.push_str(&format!("======================\n"));
    output.push_str(&format!("Total games processed: {}\n", stats.total_games));
    output.push_str(&format!("Total players analyzed: {}\n", stats.players.len()));
    output.push_str(&format!("\n"));
    
    // Show aggregated stats
    if let Some(most_common) = stats.aggregated_stats.get("most_common_move") {
        output.push_str(&format!("Most common move: {}\n", most_common.as_str().unwrap_or("unknown")));
    }
    if let Some(avg_moves) = stats.aggregated_stats.get("average_moves_per_game") {
        output.push_str(&format!("Average moves per game: {}\n", avg_moves.as_u64().unwrap_or(0)));
    }
    
    output.push_str(&format!("\nPlayer breakdown:\n"));
    for player in &stats.players {
        let total_moves: u32 = player.moves.values().sum();
        output.push_str(&format!("Port {}: {} - {} total moves\n", player.port, player.character, total_moves));
        
        // Show top 5 moves for each player
        let mut moves_vec: Vec<_> = player.moves.iter().collect();
        moves_vec.sort_by(|a, b| b.1.cmp(a.1));
        for (i, (move_name, count)) in moves_vec.iter().take(5).enumerate() {
            output.push_str(&format!("  {}. {}: {}\n", i + 1, move_name, count));
        }
        output.push_str(&format!("\n"));
    }
    
    output
}