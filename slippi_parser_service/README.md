# Slippi Parser Service

A fast, high-performance Slippi replay file parser built in Rust using the `peppi` library.

## Features

- **Fast parsing**: Built in Rust for maximum performance
- **Multiple output formats**: JSON and text output
- **Comprehensive data extraction**: Player info, stage, frame count, and more
- **Command-line interface**: Easy to use CLI with argument parsing
- **Batch processing**: Recursive directory processing script

## Installation

1. Ensure you have Rust installed (1.70+)
2. Clone this repository
3. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

### Single File Processing

Parse a Slippi replay file and output as JSON:
```bash
./target/release/slippi_parser_service -f path/to/your/game.slp
```

### Output Formats

**JSON format (default):**
```bash
./target/release/slippi_parser_service -f game.slp --format json
```

**Text format:**
```bash
./target/release/slippi_parser_service -f game.slp --format text
```

### Batch Directory Processing

Process all `.slp` files in a directory recursively:

```bash
# Process all files in a directory (outputs to input_dir/processed)
./process_directory_simple.sh /path/to/slippi/replays

# Specify custom output directory
./process_directory_simple.sh -o /path/to/output /path/to/slippi/replays

# Output in text format
./process_directory_simple.sh -f text /path/to/slippi/replays

# Verbose output to see each file being processed
./process_directory_simple.sh -v /path/to/slippi/replays

# Dry run to see what would be processed
./process_directory_simple.sh -d /path/to/slippi/replays
```

### Command Line Options

#### Single File Parser
- `-f, --file <FILE>`: Path to the Slippi replay file (.slp)
- `-o, --format <FORMAT>`: Output format (json, text) [default: json]

#### Directory Processor
- `-o, --output <dir>`: Output directory (default: input_dir/processed)
- `-f, --format <format>`: Output format: json or text (default: json)
- `-v, --verbose`: Verbose output
- `-d, --dry-run`: Show what would be processed without actually processing
- `-h, --help`: Show help message

## Output Data

The parser extracts the following information:

- **Player count**: Number of players in the game
- **Duration**: Total number of frames in the replay
- **Stage**: The stage where the game was played
- **Player details**: For each player:
  - Port number
  - Character
  - Starting stocks
  - Costume
  - Team (if applicable)

## Example Output

### JSON Format
```json
{
  "player_count": 2,
  "duration_frames": 12345,
  "stage": "Battlefield",
  "players": [
    {
      "port": 1,
      "character": "Fox",
      "stocks": 4,
      "costume": 0,
      "team": null
    },
    {
      "port": 2,
      "character": "Falco",
      "stocks": 4,
      "costume": 1,
      "team": null
    }
  ]
}
```

### Text Format
```
Game Data:
  Players: 2
  Duration: 12345 frames
  Stage: Battlefield
```

## Performance

This parser is designed for high-performance batch processing of Slippi replay files. The Rust implementation provides:

- Fast file parsing
- Low memory usage
- Efficient data extraction
- Optimized for large replay files

## Dependencies

- `peppi`: Slippi replay file parser
- `tokio`: Async runtime
- `serde`: Serialization
- `clap`: Command-line argument parsing
- `anyhow`: Error handling
- `tracing`: Logging

## Development

To run in development mode:
```bash
cargo run -- -f path/to/game.slp
```

To run tests:
```bash
cargo test
```

## License

MIT License 