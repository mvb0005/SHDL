-- Slippi Game Data Schema for DuckDB
-- This script creates tables optimized for analytics queries

-- Main games table
CREATE TABLE IF NOT EXISTS games (
    game_id VARCHAR PRIMARY KEY,
    stage VARCHAR,
    duration_frames INTEGER,
    player_count INTEGER,
    file_path VARCHAR,
    parsed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Players table with game details
CREATE TABLE IF NOT EXISTS players (
    player_id VARCHAR PRIMARY KEY,
    game_id VARCHAR,
    port INTEGER,
    character VARCHAR,
    stocks INTEGER,
    costume INTEGER,
    team VARCHAR,
    FOREIGN KEY (game_id) REFERENCES games(game_id)
);

-- Denormalized view for fast analytics
CREATE TABLE IF NOT EXISTS game_stats AS
SELECT 
    g.game_id,
    g.stage,
    g.duration_frames,
    g.player_count,
    p.port,
    p.character,
    p.stocks,
    p.costume,
    p.team,
    g.parsed_at,
    -- Extract date from file path for time-based analysis
    CASE 
        WHEN g.file_path LIKE '%2025-05%' THEN '2025-05-01'::DATE
        WHEN g.file_path LIKE '%2025-06%' THEN '2025-06-01'::DATE  
        WHEN g.file_path LIKE '%2025-07%' THEN '2025-07-01'::DATE
        ELSE '2025-01-01'::DATE
    END as game_month
FROM games g
JOIN players p ON g.game_id = p.game_id;

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_character ON players(character);
CREATE INDEX IF NOT EXISTS idx_stage ON games(stage);
CREATE INDEX IF NOT EXISTS idx_game_month ON game_stats(game_month);

-- Views for common analytics
CREATE VIEW IF NOT EXISTS character_usage AS
SELECT 
    character,
    COUNT(*) as games_played,
    COUNT(*) * 100.0 / SUM(COUNT(*)) OVER() as usage_percentage
FROM players 
GROUP BY character
ORDER BY games_played DESC;

CREATE VIEW IF NOT EXISTS stage_popularity AS
SELECT 
    stage,
    COUNT(*) as games_played,
    AVG(duration_frames) as avg_duration,
    COUNT(*) * 100.0 / SUM(COUNT(*)) OVER() as stage_percentage
FROM games
GROUP BY stage
ORDER BY games_played DESC;

CREATE VIEW IF NOT EXISTS monthly_activity AS
SELECT 
    game_month,
    COUNT(DISTINCT game_id) as total_games,
    COUNT(*) as total_player_games,
    COUNT(DISTINCT character) as unique_characters_used
FROM game_stats
GROUP BY game_month
ORDER BY game_month DESC; 