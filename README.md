# SHDL - Short Hop Double Laser Analysis

A comprehensive Slippi replay analysis project focused on analyzing **Short Hop Double Laser (SHDL)** techniques in Super Smash Bros. Melee. This project processes Slippi replay files (.slp) and converts them to structured JSON data for detailed analysis of SHDL usage patterns, timing, and effectiveness.

## What is SHDL?

Short Hop Double Laser (SHDL) is an advanced technique in Super Smash Bros. Melee, primarily used by Fox and Falco players. It involves performing two laser shots during a single short hop, creating pressure and controlling stage space effectively.

## Project Overview

The SHDL project processes raw Slippi replay files and extracts comprehensive game data including:

- Player information and character choices
- Stage selection and game duration
- Frame-by-frame game state data
- Player actions and inputs
- Match outcomes and statistics

This data enables detailed analysis of SHDL technique usage, player performance patterns, and match dynamics.

## Project Structure

```
SHDL/
├── README.md                     # This file
├── .gitignore                    # Excludes games/* and parsedgames/*
├── games/                        # Raw Slippi replay files (excluded from repo)
│   ├── 2025-05/                 # May 2025 games
│   ├── 2025-06/                 # June 2025 games
│   └── 2025-07/                 # July 2025 games
├── parsedgames/                  # Parsed JSON files (excluded from repo)
│   └── Game_YYYYMMDDTHHMMSS.json # Parsed game data
├── slippi_parser_service/        # Rust-based parser service
│   ├── src/                     # Source code
│   ├── sql/                     # Database schemas and queries
│   ├── process_directory*.sh    # Batch processing scripts
│   └── README.md                # Parser service documentation
└── .github/                      # GitHub issue templates
    └── ISSUE_TEMPLATE/          # Bug reports, feature requests, etc.
```

### Directory Details

- **`games/`**: Raw Slippi replay files organized by month (2025-05/, 2025-06/, 2025-07/)
- **`parsedgames/`**: Contains 70+ parsed JSON files with format `Game_YYYYMMDDTHHMMSS.json`
- **`slippi_parser_service/`**: High-performance Rust-based parser for converting .slp files to JSON
- **`.github/ISSUE_TEMPLATE/`**: Issue templates for bug reports, feature requests, parser issues, and documentation

## Prerequisites

- **Rust** (1.70 or higher) - for the parser service
- **Docker** (optional) - for database analytics
- **Git** - for version control

## Installation & Setup

### 1. Clone the Repository

```bash
git clone https://github.com/mvb0005/SHDL.git
cd SHDL
```

### 2. Build the Parser Service

```bash
cd slippi_parser_service
cargo build --release
```

### 3. Verify Installation

```bash
# Run tests
cargo test

# Check parser help
./target/release/slippi_parser_service --help
```

## Usage Guide

### Adding New Game Files

1. Place your Slippi replay files (`.slp`) in the `games/` directory
2. Organize them by month: `games/2025-07/`, `games/2025-08/`, etc.
3. The parser will maintain this organization in the output

### Single File Processing

Parse a single Slippi replay file:

```bash
cd slippi_parser_service
./target/release/slippi_parser_service -f /path/to/your/game.slp
```

Output formats:
```bash
# JSON format (default)
./target/release/slippi_parser_service -f game.slp --format json

# Text format for quick viewing
./target/release/slippi_parser_service -f game.slp --format text
```

### Batch Directory Processing

Process all `.slp` files in a directory:

```bash
cd slippi_parser_service

# Process all files in games/ directory
./process_directory_simple.sh ../games/

# Specify custom output directory
./process_directory_simple.sh -o ../parsedgames ../games/

# Verbose output to see progress
./process_directory_simple.sh -v ../games/

# Dry run to preview what would be processed
./process_directory_simple.sh -d ../games/
```

### Working with Parsed Data

The parser generates JSON files with the following structure:

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

### Database Analytics (Optional)

For advanced analytics, set up the DuckDB database:

```bash
cd slippi_parser_service

# Start database container
docker-compose up -d

# Set up schema and load data
./setup_database.sh

# Run analytics queries
docker exec -i slippi_duckdb duckdb slippi.db < sql/03_analytics_queries.sql
```

This provides powerful analytics capabilities including:
- Character usage statistics
- Stage popularity analysis
- Monthly activity trends
- Performance metrics

## Example Workflows

### Complete Analysis Pipeline

1. **Add new replays**: Place `.slp` files in `games/YYYY-MM/` directory
2. **Process replays**: Run batch processing to generate JSON files
3. **Load into database**: Use database setup to enable analytics
4. **Analyze data**: Run SQL queries to extract insights

```bash
# 1. Process new games
cd slippi_parser_service
./process_directory_simple.sh -v ../games/2025-07/ -o ../parsedgames/

# 2. Update database with new data
./setup_database.sh

# 3. Run analytics
docker exec -i slippi_duckdb duckdb slippi.db < sql/03_analytics_queries.sql
```

### Quick Analysis of a Single Game

```bash
# Parse and view game summary
cd slippi_parser_service
./target/release/slippi_parser_service -f ../games/2025-07/game.slp --format text

# Generate JSON for detailed analysis
./target/release/slippi_parser_service -f ../games/2025-07/game.slp --format json > /tmp/game_analysis.json
```

## Parsed JSON File Format

Each parsed game file contains:

### Game Metadata
- **player_count**: Number of players (1-4)
- **duration_frames**: Total game length in frames
- **stage**: Stage where the game was played

### Player Data
For each player:
- **port**: Controller port (1-4)
- **character**: Character name (Fox, Falco, etc.)
- **stocks**: Starting stock count
- **costume**: Character costume/color
- **team**: Team assignment (if applicable)

### File Naming Convention
- Format: `Game_YYYYMMDDTHHMMSS.json`
- Example: `Game_20250707T143022.json` (July 7, 2025, 2:30:22 PM)

## Contributing

We welcome contributions! Please use our GitHub issue templates:

- [🐛 Bug Report](.github/ISSUE_TEMPLATE/bug_report.md)
- [✨ Feature Request](.github/ISSUE_TEMPLATE/feature_request.md)
- [🔧 Parser Issue](.github/ISSUE_TEMPLATE/parser_issue.md)
- [📚 Documentation](.github/ISSUE_TEMPLATE/documentation.md)

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run tests: `cd slippi_parser_service && cargo test`
5. Commit your changes: `git commit -m 'Add some feature'`
6. Push to the branch: `git push origin feature/your-feature`
7. Submit a pull request

### Code Style Guidelines

- Follow Rust conventions for the parser service
- Use meaningful variable names and comments
- Include tests for new functionality
- Update documentation for significant changes

## Performance

The Rust-based parser is optimized for high-performance batch processing:

- **Fast parsing**: Efficient .slp file processing
- **Low memory usage**: Minimal resource consumption
- **Batch processing**: Handle hundreds of files efficiently
- **Error handling**: Robust error reporting and recovery

## Troubleshooting

### Common Issues

**Parser not found**: Make sure to build the project first:
```bash
cd slippi_parser_service
cargo build --release
```

**Permission denied**: Ensure scripts are executable:
```bash
chmod +x process_directory_simple.sh
chmod +x setup_database.sh
```

**Database connection issues**: Check Docker is running:
```bash
docker ps
docker-compose up -d
```

### Getting Help

1. Check existing [GitHub issues](https://github.com/mvb0005/SHDL/issues)
2. Use the appropriate issue template to report problems
3. Include relevant error messages and system information

## License

This project is licensed under the MIT License - see the parser service documentation for details.

## Acknowledgments

- Built with [peppi](https://github.com/hohav/peppi) Slippi replay parser
- Uses [Slippi](https://slippi.gg/) replay format
- Inspired by the Super Smash Bros. Melee competitive community

---

For detailed parser service documentation, see [`slippi_parser_service/README.md`](slippi_parser_service/README.md).