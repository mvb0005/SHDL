# Move Extraction and Analysis

This document describes the move extraction and counting system for Slippi replay files.

## Overview

The move extraction system analyzes frame-by-frame data from Slippi replay files to identify and count individual moves (attacks, actions, inputs). This enables statistical analysis of move usage patterns in competitive Melee matches.

## Tools

### 1. Slippi Parser Service (Enhanced)

The main parser now supports move extraction from `.slp` files:

```bash
# Parse a single .slp file with move extraction
./target/release/slippi_parser_service -f game.slp --extract-moves

# Process directory of JSON files for aggregated statistics
./target/release/slippi_parser_service --process-directory -f parsedgames/
```

### 2. Move Analyzer (New)

Dedicated tool for analyzing move statistics from parsed JSON files:

```bash
# Analyze moves from JSON files in directory
./target/release/move_analyzer -d parsedgames/

# Output in different formats
./target/release/move_analyzer -d parsedgames/ --format text
./target/release/move_analyzer -d parsedgames/ --format csv
./target/release/move_analyzer -d parsedgames/ --format json

# Save to file
./target/release/move_analyzer -d parsedgames/ --format csv -o moves.csv
```

## Move Categories

The system tracks the following move categories:

### Aerials
- `nair` - Neutral Air
- `fair` - Forward Air  
- `bair` - Back Air
- `uair` - Up Air
- `dair` - Down Air

### Ground Attacks
- `jab` - Jab
- `dtilt` - Down Tilt
- `utilt` - Up Tilt
- `ftilt` - Forward Tilt
- `fsmash` - Forward Smash
- `usmash` - Up Smash
- `dsmash` - Down Smash
- `grab` - Grab
- `dash_attack` - Dash Attack

### Specials
- `neutral_b` - Neutral B
- `side_b` - Side B
- `up_b` - Up B
- `down_b` - Down B
- `shine` - Shine (Fox/Falco specific)
- `laser` - Laser (Falco specific)

### Movement
- `jump` - Jump
- `double_jump` - Double Jump
- `wavedash` - Wavedash
- `waveland` - Waveland
- `l_cancel` - L-Cancel

## Output Formats

### JSON Format
```json
{
  "total_games": 3,
  "players": [
    {
      "port": 1,
      "character": "Fox",
      "moves": {
        "nair": 15,
        "fair": 8,
        "uair": 12,
        "shine": 25,
        "jump": 45
      }
    }
  ],
  "aggregated_stats": {
    "most_common_move": "jump",
    "average_moves_per_game": 279
  }
}
```

### CSV Format
```csv
port,character,move,count
1,Fox,nair,15
1,Fox,fair,8
1,Fox,uair,12
1,Fox,shine,25
1,Fox,jump,45
```

### Text Format
```
Move Statistics Summary
======================
Total games processed: 3
Total players analyzed: 6

Most common move: jump
Average moves per game: 279

Player breakdown:
Port 1: Fox - 110 total moves
  1. jump: 45
  2. shine: 25
  3. nair: 15
  4. uair: 12
  5. fair: 8
```

## Technical Implementation

### Move Detection Logic

The system identifies moves by analyzing:

1. **Action States**: Frame-by-frame character action states from peppi's Pre/Post data
2. **Button Inputs**: Physical button presses and combinations
3. **Context Analysis**: Character-specific move detection (e.g., Falco laser vs Fox shine)
4. **Special Techniques**: Advanced techniques like wavedashing and L-canceling

### Action State Mapping

Key action states mapped to moves:
- 13-17: Aerial attacks (nair, fair, bair, uair, dair)
- 18-24: Ground attacks (jab, tilts, smashes)
- 25-28: Special moves (neutral-B, side-B, up-B, down-B)
- 29-32: Grabs and movement (grab, dash attack, jumps)

### Performance

The system is optimized for:
- Processing large batches of games (70+ files tested)
- Efficient frame-by-frame analysis
- Memory-efficient aggregation of statistics
- Fast JSON parsing and output generation

## File Structure

Expected directory structure:
```
parsedgames/
├── Game_20250701T120000.json
├── Game_20250701T130000.json
├── Game_20250701T140000.json
└── ...
```

Each JSON file should contain game data with move information in the format shown above.

## Building and Testing

```bash
# Build all tools
cargo build --release

# Run tests
cargo test

# Test with sample data
./target/release/move_analyzer -d parsedgames/ --format text
```

## Use Cases

This system enables:
- **Player Profile Analysis**: Generate statistics showing most-used moves per player
- **Character Comparison**: Compare move usage patterns across different characters
- **SHDL Frequency**: Track Short Hop Double Laser frequency for Falco players
- **Improvement Tracking**: Analyze move efficiency changes over time
- **Heat Map Generation**: Create visualizations of move usage by stage position
- **Habit Analysis**: Identify player tendencies and common patterns

## Error Handling

The system handles:
- Invalid JSON files (skipped with warning)
- Missing move data (games without extracted moves)
- Empty directories
- Corrupted or incomplete game files
- Large datasets (tested with 70+ files)

## Future Enhancements

Potential improvements:
- Real-time move extraction from live replays
- Advanced combo detection and classification
- Stage-specific move analysis
- Time-based move frequency analysis
- Integration with replay databases
- Machine learning-based move prediction