-- Slippi Analytics Queries
-- Collection of useful queries for analyzing game data

-- =============================================================================
-- CHARACTER ANALYSIS
-- =============================================================================

-- Most popular characters
SELECT * FROM character_usage;

-- Character performance by stocks remaining
SELECT 
    character,
    AVG(stocks) as avg_stocks_remaining,
    COUNT(*) as games_played
FROM players 
GROUP BY character
HAVING COUNT(*) >= 10  -- Only characters with 10+ games
ORDER BY avg_stocks_remaining DESC;

-- Character matchup analysis
SELECT 
    p1.character as char1,
    p2.character as char2,
    COUNT(*) as matchup_count
FROM players p1
JOIN players p2 ON p1.game_id = p2.game_id AND p1.port != p2.port
WHERE p1.port < p2.port  -- Avoid duplicates
GROUP BY p1.character, p2.character
ORDER BY matchup_count DESC
LIMIT 20;

-- =============================================================================
-- STAGE ANALYSIS  
-- =============================================================================

-- Stage popularity and average game length
SELECT * FROM stage_popularity;

-- Character performance by stage
SELECT 
    stage,
    character,
    COUNT(*) as games_on_stage,
    AVG(stocks) as avg_stocks
FROM game_stats
GROUP BY stage, character
HAVING COUNT(*) >= 5
ORDER BY stage, avg_stocks DESC;

-- =============================================================================
-- TEMPORAL ANALYSIS
-- =============================================================================

-- Monthly gaming activity
SELECT * FROM monthly_activity;

-- Games per day (approximation based on file timestamps)
SELECT 
    game_month,
    total_games,
    total_games / 30.0 as approx_games_per_day
FROM monthly_activity;

-- Character usage trends over time
SELECT 
    game_month,
    character,
    COUNT(*) as usage_count,
    COUNT(*) * 100.0 / SUM(COUNT(*)) OVER (PARTITION BY game_month) as monthly_percentage
FROM game_stats
GROUP BY game_month, character
ORDER BY game_month DESC, usage_count DESC;

-- =============================================================================
-- GAME LENGTH ANALYSIS
-- =============================================================================

-- Game duration statistics
SELECT 
    AVG(duration_frames) as avg_frames,
    MIN(duration_frames) as shortest_game,
    MAX(duration_frames) as longest_game,
    PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY duration_frames) as median_frames,
    -- Convert frames to approximate seconds (60 FPS)
    AVG(duration_frames) / 60.0 as avg_seconds
FROM games;

-- Game length by stage
SELECT 
    stage,
    COUNT(*) as game_count,
    AVG(duration_frames) as avg_frames,
    AVG(duration_frames) / 60.0 as avg_seconds,
    MIN(duration_frames) / 60.0 as shortest_seconds,
    MAX(duration_frames) / 60.0 as longest_seconds
FROM games
GROUP BY stage
ORDER BY avg_frames DESC;

-- Game length by character matchup
SELECT 
    p1.character as char1,
    p2.character as char2,
    COUNT(*) as games,
    AVG(g.duration_frames) / 60.0 as avg_duration_seconds
FROM games g
JOIN players p1 ON g.game_id = p1.game_id AND p1.port = 0
JOIN players p2 ON g.game_id = p2.game_id AND p2.port = 1
GROUP BY p1.character, p2.character
HAVING COUNT(*) >= 5
ORDER BY avg_duration_seconds DESC;

-- =============================================================================
-- PLAYER PERFORMANCE
-- =============================================================================

-- Stock differential analysis (2-player games only)
WITH stock_diff AS (
    SELECT 
        game_id,
        MAX(CASE WHEN port = 0 THEN stocks END) - 
        MAX(CASE WHEN port = 1 THEN stocks END) as stock_difference
    FROM players 
    WHERE game_id IN (SELECT game_id FROM games WHERE player_count = 2)
    GROUP BY game_id
)
SELECT 
    stock_difference,
    COUNT(*) as game_count,
    COUNT(*) * 100.0 / SUM(COUNT(*)) OVER() as percentage
FROM stock_diff
GROUP BY stock_difference
ORDER BY stock_difference;

-- Port advantage analysis
SELECT 
    port,
    COUNT(*) as games_played,
    AVG(stocks) as avg_stocks_remaining
FROM players
GROUP BY port
ORDER BY port;

-- =============================================================================
-- QUICK STATS DASHBOARD
-- =============================================================================

-- Overall statistics summary
SELECT 
    (SELECT COUNT(*) FROM games) as total_games,
    (SELECT COUNT(DISTINCT character) FROM players) as unique_characters,
    (SELECT COUNT(DISTINCT stage) FROM games) as unique_stages,
    (SELECT AVG(duration_frames) / 60.0 FROM games) as avg_game_duration_seconds,
    (SELECT MAX(parsed_at) FROM games) as last_updated; 