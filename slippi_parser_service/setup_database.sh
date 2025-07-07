#!/bin/bash

# Slippi DuckDB Setup Script
# Sets up DuckDB container and loads all parsed game data

set -e

echo "🦆 Setting up Slippi DuckDB Analytics Database"
echo "=============================================="

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Start DuckDB container
echo "🚀 Starting DuckDB container..."
docker-compose up -d

# Wait for container to be ready
echo "⏳ Waiting for DuckDB to be ready..."
sleep 3

# Check if container is running
if ! docker ps | grep -q slippi_duckdb; then
    echo "❌ Failed to start DuckDB container"
    exit 1
fi

echo "✅ DuckDB container is running"

# Create database schema
echo "📊 Creating database schema..."
docker exec -i slippi_duckdb duckdb slippi.db < sql/01_create_tables.sql

# Load data from JSON files
echo "📥 Loading Slippi game data..."
docker exec -i slippi_duckdb duckdb slippi.db < sql/02_load_data.sql

echo ""
echo "🎉 Database setup complete!"
echo ""
echo "📈 Quick Stats:"
docker exec -i slippi_duckdb duckdb slippi.db -c "
SELECT 
    (SELECT COUNT(*) FROM games) as total_games,
    (SELECT COUNT(*) FROM players) as total_players,
    (SELECT COUNT(DISTINCT character) FROM players) as unique_characters,
    (SELECT COUNT(DISTINCT stage) FROM games) as unique_stages;
"

echo ""
echo "🔧 Usage:"
echo "  Connect to database: docker exec -it slippi_duckdb duckdb slippi.db"
echo "  Run analytics: docker exec -i slippi_duckdb duckdb slippi.db < sql/03_analytics_queries.sql"
echo "  Stop container: docker-compose down"
echo ""
echo "📁 Files:"
echo "  Database: ./data/slippi.db"
echo "  SQL Scripts: ./sql/"
echo "  Data Source: ../parsedgames/" 