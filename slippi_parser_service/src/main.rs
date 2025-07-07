use anyhow::Result;
use clap::Parser;
use peppi::io::slippi::read;
use peppi::game::Player;
use peppi::frame::immutable::Frame;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "slippi_parser_service")]
#[command(about = "A fast Slippi replay file parser using peppi")]
struct Args {
    /// Path to the Slippi replay file (.slp) or directory containing JSON files
    #[arg(short, long)]
    file: PathBuf,
    
    /// Output format (json, text)
    #[arg(long, default_value = "json")]
    format: String,
    
    /// Enable move extraction and counting
    #[arg(long)]
    extract_moves: bool,
    
    /// Process directory of JSON files for move statistics
    #[arg(long)]
    process_directory: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("Starting Slippi parser service");
    
    if args.process_directory {
        info!("Processing directory for move statistics: {:?}", args.file);
        match process_directory_for_moves(&args.file).await {
            Ok(stats) => {
                match args.format.as_str() {
                    "json" => {
                        let json = serde_json::to_string_pretty(&stats)?;
                        println!("{}", json);
                    }
                    "text" => {
                        println!("Move Statistics:");
                        println!("  Total games: {}", stats.total_games);
                        println!("  Players analyzed: {}", stats.players.len());
                        for player in &stats.players {
                            println!("    Port {}: {} moves", player.port, player.moves.len());
                        }
                    }
                    _ => {
                        error!("Unknown format: {}", args.format);
                        return Err(anyhow::anyhow!("Unknown format"));
                    }
                }
            }
            Err(e) => {
                error!("Failed to process directory: {}", e);
                return Err(e);
            }
        }
    } else {
        info!("Parsing file: {:?}", args.file);
        
        // Parse the Slippi file
        match parse_slippi_file(&args.file, args.extract_moves).await {
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
                        if let Some(moves) = &game_data.moves {
                            println!("  Move data extracted for {} players", moves.len());
                        }
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
    }
    
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GameData {
    player_count: usize,
    duration_frames: u32,
    stage: String,
    players: Vec<PlayerData>,
    moves: Option<Vec<PlayerMoveData>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PlayerData {
    port: u8,
    character: String,
    stocks: u8,
    costume: u8,
    team: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PlayerMoveData {
    port: u8,
    character: String,
    moves: HashMap<String, u32>,
}

#[derive(serde::Serialize)]
struct MoveStats {
    total_games: u32,
    players: Vec<PlayerMoveData>,
    aggregated_stats: HashMap<String, serde_json::Value>,
}

async fn parse_slippi_file(file_path: &PathBuf, extract_moves: bool) -> Result<GameData> {
    info!("Reading Slippi file from: {:?}", file_path);
    
    // Parse with peppi using the correct API
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let game = read(&mut reader, None)?;
    
    info!("Successfully parsed Slippi replay");
    
    // Extract move data if requested
    let move_data = if extract_moves {
        info!("Extracting move data from {} frames", game.frames.len());
        Some(extract_moves_from_frames(&game.frames, &game.start.players)?)
    } else {
        None
    };
    
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
        moves: move_data,
    };
    
    info!("Extracted game data: {} players, {} frames", 
          game_data.player_count, game_data.duration_frames);
    
    Ok(game_data)
}

// Extract moves from frame data
fn extract_moves_from_frames(frames: &Frame, players: &[Player]) -> Result<Vec<PlayerMoveData>> {
    let mut player_moves: Vec<PlayerMoveData> = Vec::new();
    
    // Initialize move counters for each player
    for player in players {
        player_moves.push(PlayerMoveData {
            port: player.port.into(),
            character: format!("{:?}", player.character),
            moves: HashMap::new(),
        });
    }
    
    // Iterate through all frames to extract moves
    for frame_idx in 0..frames.len() {
        let frame = frames.transpose_one(frame_idx, peppi::io::slippi::Version(3, 0, 0));
        
        for (port_idx, port_data) in frame.ports.iter().enumerate() {
            if let Some(player_data) = player_moves.get_mut(port_idx) {
                // Analyze pre-frame data for inputs and action states
                analyze_frame_for_moves(port_data, player_data, frame_idx);
            }
        }
    }
    
    Ok(player_moves)
}

// Analyze a single frame for move detection
fn analyze_frame_for_moves(port_data: &peppi::frame::transpose::PortData, player_data: &mut PlayerMoveData, frame_idx: usize) {
    let leader = &port_data.leader;
    
    // Get action state
    let action_state = leader.pre.state;
    let buttons = leader.pre.buttons;
    
    // Identify moves based on action state
    if let Some(move_name) = identify_move_from_action_state(action_state, buttons) {
        let counter = player_data.moves.entry(move_name).or_insert(0);
        *counter += 1;
    }
    
    // Additional analysis for special moves and techniques
    analyze_special_techniques(port_data, player_data, frame_idx);
}

// Map action states to move names
fn identify_move_from_action_state(action_state: u16, _buttons: u32) -> Option<String> {
    match action_state {
        // Aerial attacks
        13 => Some("nair".to_string()),
        14 => Some("fair".to_string()),
        15 => Some("bair".to_string()),
        16 => Some("uair".to_string()),
        17 => Some("dair".to_string()),
        
        // Ground attacks
        18 => Some("jab".to_string()),
        19 => Some("ftilt".to_string()),
        20 => Some("utilt".to_string()),
        21 => Some("dtilt".to_string()),
        22 => Some("fsmash".to_string()),
        23 => Some("usmash".to_string()),
        24 => Some("dsmash".to_string()),
        
        // Special moves
        25 => Some("neutral_b".to_string()),
        26 => Some("side_b".to_string()),
        27 => Some("up_b".to_string()),
        28 => Some("down_b".to_string()),
        
        // Grabs
        29 => Some("grab".to_string()),
        30 => Some("dash_attack".to_string()),
        
        // Movement
        31 => Some("jump".to_string()),
        32 => Some("double_jump".to_string()),
        
        _ => None,
    }
}

// Analyze special techniques like wavedash, L-cancel, etc.
fn analyze_special_techniques(port_data: &peppi::frame::transpose::PortData, player_data: &mut PlayerMoveData, _frame_idx: usize) {
    let leader = &port_data.leader;
    
    // Check for wavedash (air dodge into ground within short timeframe)
    if leader.pre.state == 39 && leader.post.airborne == Some(0) { // Air dodge that ends on ground
        let counter = player_data.moves.entry("wavedash".to_string()).or_insert(0);
        *counter += 1;
    }
    
    // Check for L-cancel (shield press during landing lag)
    if leader.pre.buttons & 0x40 != 0 && leader.pre.state >= 40 && leader.pre.state <= 43 { // Shield during landing states
        let counter = player_data.moves.entry("l_cancel".to_string()).or_insert(0);
        *counter += 1;
    }
    
    // Check for shine (down-B for spacies)
    if leader.pre.state == 28 && (player_data.character == "Fox" || player_data.character == "Falco") {
        let counter = player_data.moves.entry("shine".to_string()).or_insert(0);
        *counter += 1;
    }
    
    // Check for laser (neutral-B for Falco)
    if leader.pre.state == 25 && player_data.character == "Falco" {
        let counter = player_data.moves.entry("laser".to_string()).or_insert(0);
        *counter += 1;
    }
}

// Process directory of JSON files for aggregated statistics
async fn process_directory_for_moves(directory: &PathBuf) -> Result<MoveStats> {
    use std::fs;
    
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
            moves: None,
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

    #[test]
    fn test_move_identification() {
        // Test action state to move name mapping
        assert_eq!(identify_move_from_action_state(13, 0), Some("nair".to_string()));
        assert_eq!(identify_move_from_action_state(14, 0), Some("fair".to_string()));
        assert_eq!(identify_move_from_action_state(15, 0), Some("bair".to_string()));
        assert_eq!(identify_move_from_action_state(16, 0), Some("uair".to_string()));
        assert_eq!(identify_move_from_action_state(17, 0), Some("dair".to_string()));
        assert_eq!(identify_move_from_action_state(18, 0), Some("jab".to_string()));
        assert_eq!(identify_move_from_action_state(25, 0), Some("neutral_b".to_string()));
        assert_eq!(identify_move_from_action_state(999, 0), None);
    }

    #[test]
    fn test_move_data_serialization() {
        let mut moves = HashMap::new();
        moves.insert("nair".to_string(), 10);
        moves.insert("fair".to_string(), 5);
        moves.insert("laser".to_string(), 20);

        let player_moves = PlayerMoveData {
            port: 1,
            character: "Falco".to_string(),
            moves,
        };

        let json = serde_json::to_string(&player_moves).unwrap();
        assert!(json.contains("Falco"));
        assert!(json.contains("nair"));
        assert!(json.contains("laser"));
        assert!(json.contains("10"));
        assert!(json.contains("20"));
    }

    #[test]
    fn test_move_stats_structure() {
        let mut stats_map = HashMap::new();
        stats_map.insert("most_common_move".to_string(), serde_json::Value::String("laser".to_string()));
        stats_map.insert("average_moves_per_game".to_string(), serde_json::Value::Number(150.into()));

        let stats = MoveStats {
            total_games: 3,
            players: vec![],
            aggregated_stats: stats_map,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("total_games"));
        assert!(json.contains("most_common_move"));
        assert!(json.contains("laser"));
        assert!(json.contains("150"));
    }
}
