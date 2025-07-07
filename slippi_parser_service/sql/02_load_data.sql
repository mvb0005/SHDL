-- Load Slippi JSON data into DuckDB tables
-- This script reads all JSON files and populates the database

-- First, let's create a temporary view to read all JSON files
CREATE OR REPLACE VIEW json_files AS
SELECT 
    filename,
    json
FROM read_json_auto('/parsedgames/**/*.json', filename=true);

-- Insert games data
INSERT INTO games (game_id, stage, duration_frames, player_count, file_path)
SELECT 
    -- Generate game_id from filename
    regexp_replace(filename, '.*/([^/]+)\.json$', '\1') as game_id,
    json.stage,
    json.duration_frames,
    json.player_count,
    filename as file_path
FROM json_files;

-- Insert players data
INSERT INTO players (player_id, game_id, port, character, stocks, costume, team)
SELECT 
    -- Generate player_id as game_id + port
    regexp_replace(filename, '.*/([^/]+)\.json$', '\1') || '_' || player.port as player_id,
    regexp_replace(filename, '.*/([^/]+)\.json$', '\1') as game_id,
    player.port,
    player.character,
    player.stocks,
    player.costume,
    player.team
FROM json_files,
UNNEST(json.players) as t(player);

-- Refresh the materialized game_stats table
DROP TABLE IF EXISTS game_stats;
CREATE TABLE game_stats AS
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

-- Show summary of loaded data
SELECT 'Games loaded' as metric, COUNT(*) as count FROM games
UNION ALL
SELECT 'Players loaded' as metric, COUNT(*) as count FROM players
UNION ALL
SELECT 'Unique characters' as metric, COUNT(DISTINCT character) as count FROM players
UNION ALL
SELECT 'Unique stages' as metric, COUNT(DISTINCT stage) as count FROM games; 