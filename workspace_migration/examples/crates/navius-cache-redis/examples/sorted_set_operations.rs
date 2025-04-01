use navius_cache::{
    config::CacheConfig,
    error::CacheResult,
    key::CacheKey,
    operations::Cache,
};
use navius_cache_redis::{
    config::RedisCacheConfig,
    connection::RedisConnectionManager,
    operations::{RedisCache, SortedSetOperations},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

/**
 * This example demonstrates the use of Sorted Set (ZSet) operations in the Redis cache.
 * 
 * Sorted Sets allow you to:
 * - Store items with associated scores
 * - Retrieve items by score range
 * - Retrieve items by rank (position in the sorted order)
 * - Perform complex operations like unions and intersections with score aggregation
 * 
 * Sorted Sets are useful for:
 * - Leaderboards and rankings
 * - Time-series data with timestamps as scores
 * - Priority queues
 * - Rate limiting with timestamps
 * - Weighted recommendations
 * - Range queries
 */

// Example data structure for demonstrating serialization with sorted sets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct Player {
    id: String,
    name: String,
    team: String,
}

impl Player {
    fn new(id: &str, name: &str, team: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            team: team.to_string(),
        }
    }
}

// Implement CacheKey for Player so it can be used as a key in the cache
impl CacheKey for Player {
    fn to_cache_key(&self) -> String {
        format!("player:{}", self.id)
    }
}

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Configure Redis cache
    let config = RedisCacheConfig::builder()
        .with_url("redis://127.0.0.1:6379")
        .with_key_prefix("navius:example:zsets")
        .with_default_ttl(Duration::from_secs(300))
        .with_pool_size(5)
        .build();

    // Create connection manager
    let conn_manager = Arc::new(
        RedisConnectionManager::new(config)
            .await
            .expect("Failed to create Redis connection manager"),
    );

    // Create cache instance
    let cache = RedisCache::new(conn_manager)
        .expect("Failed to create Redis cache");

    println!("== Redis Sorted Set Operations Example ==");

    // Basic Sorted Set Operations Example
    println!("\n1. Basic Sorted Set Operations");
    await basic_sorted_set_operations(&cache).await?;

    // Score Range Example
    println!("\n2. Score Range Operations");
    await score_range_example(&cache).await?;

    // Rank Operations Example
    println!("\n3. Rank Operations");
    await rank_operations_example(&cache).await?;

    // Leaderboard Example
    println!("\n4. Leaderboard Example");
    await leaderboard_example(&cache).await?;

    // Complex Object Example
    println!("\n5. Complex Object Example");
    await complex_object_example(&cache).await?;

    // Union and Intersection Example
    println!("\n6. Union and Intersection Example");
    await union_intersection_example(&cache).await?;

    // Clean up
    cleanup(&cache).await?;

    println!("\nSorted Set operations example completed successfully!");
    Ok(())
}

async fn basic_sorted_set_operations(cache: &RedisCache) -> CacheResult<()> {
    let zset_key = "simple_zset";
    
    // Add items to a sorted set
    println!("Adding items to sorted set '{}'", zset_key);
    
    // Create a HashMap of members with scores
    let mut scores = HashMap::new();
    scores.insert("apple".to_string(), 5.0);
    scores.insert("banana".to_string(), 8.5);
    scores.insert("cherry".to_string(), 7.0);
    
    // Add items with scores
    let added = cache.zset_add(zset_key, scores).await?;
    println!("Added {} new items to the sorted set", added);
    
    // Add more items, including an update to an existing item
    let mut more_scores = HashMap::new();
    more_scores.insert("banana".to_string(), 9.0);  // Update score
    more_scores.insert("dragonfruit".to_string(), 6.5);
    more_scores.insert("elderberry".to_string(), 4.0);
    
    let added = cache.zset_add(zset_key, more_scores).await?;
    println!("Added {} more unique items to the sorted set (plus updated 1 item)", added);
    
    // Check the sorted set's size
    let count = cache.zset_size(zset_key).await?;
    println!("Sorted set '{}' has {} items", zset_key, count);
    
    // Get all members of the sorted set (sorted by score, ascending)
    let members_with_scores = cache.zset_range_with_scores::<String, _>(zset_key, 0, -1).await?;
    println!("Sorted set members with scores (low to high):");
    for (member, score) in &members_with_scores {
        println!("- {}: {}", member, score);
    }
    
    // Get all members in reverse order (sorted by score, descending)
    let members_with_scores = cache.zset_rev_range_with_scores::<String, _>(zset_key, 0, -1).await?;
    println!("\nSorted set members with scores (high to low):");
    for (member, score) in &members_with_scores {
        println!("- {}: {}", member, score);
    }
    
    // Get the score of a specific member
    let banana_score = cache.zset_score::<String, _>(zset_key, "banana").await?;
    println!("\nScore of 'banana': {:?}", banana_score);
    
    // Get the score of a non-existent member
    let nonexistent_score = cache.zset_score::<String, _>(zset_key, "nonexistent").await?;
    println!("Score of 'nonexistent': {:?}", nonexistent_score);
    
    // Remove an item
    let removed = cache.zset_remove::<String, _>(zset_key, vec!["banana"]).await?;
    println!("\nRemoved {} item(s) from the sorted set", removed);
    
    // Get the updated set
    let updated_members = cache.zset_range::<String, _>(zset_key, 0, -1).await?;
    println!("Updated sorted set members: {:?}", updated_members);
    
    Ok(())
}

async fn score_range_example(cache: &RedisCache) -> CacheResult<()> {
    let zset_key = "temperature_readings";
    
    // Add temperature readings with timestamps as scores
    println!("Adding temperature readings with timestamps as scores");
    
    // Let's simulate hourly temperature readings for a day (24 readings)
    // We'll use Unix timestamps for a day, with each reading 1 hour apart
    let base_timestamp = 1617235200.0;  // April 1, 2025, 00:00:00 UTC
    let hour_in_seconds = 3600.0;
    
    let mut temperature_readings = HashMap::new();
    
    // Create 24 hourly readings with varying temperatures
    for hour in 0..24 {
        let timestamp = base_timestamp + (hour as f64 * hour_in_seconds);
        let temperature = 15.0 + (10.0 * (hour as f64 / 24.0 * std::f64::consts::PI).sin());
        temperature_readings.insert(format!("temp:{}", hour), timestamp);
        println!("Hour {}: {:.1}°C at timestamp {}", hour, temperature, timestamp);
    }
    
    // Add all readings to the sorted set
    cache.zset_add(zset_key, temperature_readings).await?;
    
    // Query temperature readings by time range (e.g., between 6 AM and 12 PM)
    let morning_start = base_timestamp + (6.0 * hour_in_seconds); // 6 AM
    let noon = base_timestamp + (12.0 * hour_in_seconds); // 12 PM
    
    println!("\nTemperature readings between 6 AM and 12 PM:");
    let morning_readings = cache.zset_range_by_score::<String, _>(
        zset_key,
        morning_start,
        noon,
    ).await?;
    
    for reading in morning_readings {
        println!("- Reading: {}", reading);
    }
    
    // Query with scores to see the actual timestamps
    println!("\nTemperature readings with timestamps between 6 AM and 12 PM:");
    let morning_readings_with_times = cache.zset_range_by_score_with_scores::<String, _>(
        zset_key,
        morning_start,
        noon,
    ).await?;
    
    for (reading, timestamp) in morning_readings_with_times {
        let hour = ((timestamp - base_timestamp) / hour_in_seconds) as i64;
        println!("- Reading: {} at hour {}", reading, hour);
    }
    
    // Count readings in afternoon hours (12 PM to 6 PM)
    let noon = base_timestamp + (12.0 * hour_in_seconds);
    let evening = base_timestamp + (18.0 * hour_in_seconds);
    
    let afternoon_count = cache.zset_count(zset_key, noon, evening).await?;
    println!("\nNumber of readings between 12 PM and 6 PM: {}", afternoon_count);
    
    Ok(())
}

async fn rank_operations_example(cache: &RedisCache) -> CacheResult<()> {
    let zset_key = "student_scores";
    
    // Add student scores
    println!("Adding student scores to sorted set");
    
    let mut scores = HashMap::new();
    scores.insert("Alice".to_string(), 92.5);
    scores.insert("Bob".to_string(), 85.0);
    scores.insert("Charlie".to_string(), 91.0);
    scores.insert("Dave".to_string(), 78.5);
    scores.insert("Eve".to_string(), 95.0);
    scores.insert("Frank".to_string(), 79.0);
    scores.insert("Grace".to_string(), 88.0);
    scores.insert("Hannah".to_string(), 94.0);
    
    cache.zset_add(zset_key, scores).await?;
    
    // Get ranks (0-based) for some students
    // Lower ranks mean lower scores (ascending order)
    println!("\nStudent ranks (0-based, ascending by score):");
    
    let students = vec!["Alice", "Bob", "Eve"];
    for student in &students {
        let rank = cache.zset_rank::<String, _>(zset_key, student).await?;
        println!("- {}'s rank: {:?}", student, rank);
    }
    
    // Get reverse ranks (0-based) for some students
    // Lower reverse ranks mean higher scores (descending order)
    println!("\nStudent reverse ranks (0-based, descending by score):");
    
    for student in &students {
        let rev_rank = cache.zset_rev_rank::<String, _>(zset_key, student).await?;
        println!("- {}'s reverse rank: {:?}", student, rev_rank);
    }
    
    // Get top 3 students by score
    println!("\nTop 3 students:");
    let top_students = cache.zset_rev_range_with_scores::<String, _>(zset_key, 0, 2).await?;
    
    for (i, (student, score)) in top_students.iter().enumerate() {
        println!("{}. {}: {:.1}", i + 1, student, score);
    }
    
    // Get bottom 3 students by score
    println!("\nBottom 3 students:");
    let bottom_students = cache.zset_range_with_scores::<String, _>(zset_key, 0, 2).await?;
    
    for (i, (student, score)) in bottom_students.iter().enumerate() {
        println!("{}. {}: {:.1}", i + 1, student, score);
    }
    
    Ok(())
}

async fn leaderboard_example(cache: &RedisCache) -> CacheResult<()> {
    let leaderboard_key = "game_leaderboard";
    
    // Add player scores
    println!("Creating game leaderboard");
    
    let mut player_scores = HashMap::new();
    player_scores.insert("player:1001".to_string(), 12750.0);
    player_scores.insert("player:1002".to_string(), 9500.0);
    player_scores.insert("player:1003".to_string(), 14250.0);
    player_scores.insert("player:1004".to_string(), 8750.0);
    player_scores.insert("player:1005".to_string(), 15500.0);
    player_scores.insert("player:1006".to_string(), 10250.0);
    player_scores.insert("player:1007".to_string(), 11000.0);
    player_scores.insert("player:1008".to_string(), 13500.0);
    
    cache.zset_add(leaderboard_key, player_scores).await?;
    
    // Display top 5 players
    println!("\nTop 5 players:");
    let top_players = cache.zset_rev_range_with_scores::<String, _>(leaderboard_key, 0, 4).await?;
    
    for (i, (player_id, score)) in top_players.iter().enumerate() {
        println!("{}. {}: {:,.0} pts", i + 1, player_id, score);
    }
    
    // Update a player's score after they complete a new level
    println!("\nUpdating player:1007's score after completing a level");
    let mut update = HashMap::new();
    update.insert("player:1007".to_string(), 13250.0);  // New total score
    cache.zset_add(leaderboard_key, update).await?;
    
    // Check player:1007's new rank
    let new_rank = cache.zset_rev_rank::<String, _>(leaderboard_key, "player:1007").await?;
    println!("player:1007's new rank: {}", new_rank.map_or("Not found".to_string(), |r| (r + 1).to_string()));
    
    // Get nearby players (2 above and 2 below) for player:1007
    println!("\nNearby players around player:1007:");
    
    if let Some(rank) = new_rank {
        // Get the 2 players above (better scores)
        let start_above = if rank >= 2 { rank - 2 } else { 0 };
        let above_players = cache.zset_rev_range_with_scores::<String, _>(leaderboard_key, start_above, rank - 1).await?;
        
        // Get the 2 players below (worse scores)
        let below_players = cache.zset_rev_range_with_scores::<String, _>(leaderboard_key, rank + 1, rank + 2).await?;
        
        // Get the target player
        let target_player = cache.zset_rev_range_with_scores::<String, _>(leaderboard_key, rank, rank).await?;
        
        // Display nearby players
        for (player_id, score) in above_players {
            let player_rank = cache.zset_rev_rank::<String, _>(leaderboard_key, &player_id).await?;
            println!("{}. {}: {:,.0} pts", player_rank.map_or(0, |r| r + 1), player_id, score);
        }
        
        for (player_id, score) in target_player {
            let player_rank = cache.zset_rev_rank::<String, _>(leaderboard_key, &player_id).await?;
            println!("{}. {} (YOU): {:,.0} pts", player_rank.map_or(0, |r| r + 1), player_id, score);
        }
        
        for (player_id, score) in below_players {
            let player_rank = cache.zset_rev_rank::<String, _>(leaderboard_key, &player_id).await?;
            println!("{}. {}: {:,.0} pts", player_rank.map_or(0, |r| r + 1), player_id, score);
        }
    }
    
    // Calculate percentile for a player
    let target_player = "player:1004";
    if let Some(rank) = cache.zset_rev_rank::<String, _>(leaderboard_key, target_player).await? {
        let total_players = cache.zset_size(leaderboard_key).await?;
        let percentile = 100.0 * (1.0 - (rank as f64 / total_players as f64));
        let score = cache.zset_score::<String, _>(leaderboard_key, target_player).await?;
        
        println!("\nPlayer {} statistics:", target_player);
        println!("- Score: {:,.0} pts", score.unwrap_or(0.0));
        println!("- Rank: {} of {}", rank + 1, total_players);
        println!("- Percentile: {:.1}%", percentile);
    }
    
    Ok(())
}

async fn complex_object_example(cache: &RedisCache) -> CacheResult<()> {
    // Create player objects
    let players = vec![
        Player::new("p1", "Michael Jordan", "Chicago Bulls"),
        Player::new("p2", "LeBron James", "Los Angeles Lakers"),
        Player::new("p3", "Stephen Curry", "Golden State Warriors"),
        Player::new("p4", "Kevin Durant", "Phoenix Suns"),
        Player::new("p5", "Giannis Antetokounmpo", "Milwaukee Bucks"),
        Player::new("p6", "Nikola Jokic", "Denver Nuggets"),
    ];
    
    // Season statistics (points per game)
    println!("Creating player statistics sorted set");
    let ppg_key = "stats:ppg:2024-25";
    
    let mut ppg_scores = HashMap::new();
    ppg_scores.insert(&players[0], 30.1);  // Michael Jordan
    ppg_scores.insert(&players[1], 28.7);  // LeBron James
    ppg_scores.insert(&players[2], 27.2);  // Stephen Curry
    ppg_scores.insert(&players[3], 29.3);  // Kevin Durant
    ppg_scores.insert(&players[4], 27.8);  // Giannis Antetokounmpo
    ppg_scores.insert(&players[5], 26.4);  // Nikola Jokic
    
    cache.zset_add(ppg_key, ppg_scores).await?;
    
    // MVP voting points
    println!("Creating MVP voting sorted set");
    let mvp_key = "award:mvp:2024-25";
    
    let mut mvp_scores = HashMap::new();
    mvp_scores.insert(&players[0], 687.0);  // Michael Jordan
    mvp_scores.insert(&players[1], 743.0);  // LeBron James
    mvp_scores.insert(&players[2], 512.0);  // Stephen Curry
    mvp_scores.insert(&players[3], 486.0);  // Kevin Durant
    mvp_scores.insert(&players[4], 924.0);  // Giannis Antetokounmpo
    mvp_scores.insert(&players[5], 823.0);  // Nikola Jokic
    
    cache.zset_add(mvp_key, mvp_scores).await?;
    
    // Get top scorers
    println!("\nTop 3 scorers by points per game:");
    let top_scorers = cache.zset_rev_range_with_scores::<Player, _>(ppg_key, 0, 2).await?;
    
    for (i, (player, ppg)) in top_scorers.iter().enumerate() {
        println!("{}. {} ({}) - {:.1} PPG", i + 1, player.name, player.team, ppg);
    }
    
    // Get MVP voting results
    println!("\nMVP Voting Results:");
    let mvp_results = cache.zset_rev_range_with_scores::<Player, _>(mvp_key, 0, -1).await?;
    
    for (i, (player, points)) in mvp_results.iter().enumerate() {
        println!("{}. {} ({}) - {:.0} points", i + 1, player.name, player.team, points);
    }
    
    // Check if a player is a top scorer
    let player_to_check = &players[5];  // Nikola Jokic
    let rank = cache.zset_rev_rank::<Player, _>(ppg_key, player_to_check).await?;
    let ppg = cache.zset_score::<Player, _>(ppg_key, player_to_check).await?;
    
    println!("\n{} statistics:", player_to_check.name);
    println!("- PPG: {:.1}", ppg.unwrap_or(0.0));
    println!("- PPG Rank: {} of 6", rank.map_or("Not ranked".to_string(), |r| (r + 1).to_string()));
    
    // Get a player's MVP voting points
    let mvp_points = cache.zset_score::<Player, _>(mvp_key, player_to_check).await?;
    let mvp_rank = cache.zset_rev_rank::<Player, _>(mvp_key, player_to_check).await?;
    
    println!("- MVP Voting Points: {:.0}", mvp_points.unwrap_or(0.0));
    println!("- MVP Rank: {} of 6", mvp_rank.map_or("Not ranked".to_string(), |r| (r + 1).to_string()));
    
    Ok(())
}

async fn union_intersection_example(cache: &RedisCache) -> CacheResult<()> {
    // Create sets to demonstrate aggregation operations
    println!("Creating player stats for aggregation operations");
    
    // Points per game
    let ppg_key = "player:stats:ppg";
    let mut ppg_scores = HashMap::new();
    ppg_scores.insert("player1".to_string(), 28.5);
    ppg_scores.insert("player2".to_string(), 22.0);
    ppg_scores.insert("player3".to_string(), 25.0);
    ppg_scores.insert("player4".to_string(), 18.5);
    cache.zset_add(ppg_key, ppg_scores).await?;
    
    // Rebounds per game
    let rpg_key = "player:stats:rpg";
    let mut rpg_scores = HashMap::new();
    rpg_scores.insert("player1".to_string(), 6.2);
    rpg_scores.insert("player2".to_string(), 11.5);
    rpg_scores.insert("player3".to_string(), 8.0);
    rpg_scores.insert("player5".to_string(), 9.5);
    cache.zset_add(rpg_key, rpg_scores).await?;
    
    // Assists per game
    let apg_key = "player:stats:apg";
    let mut apg_scores = HashMap::new();
    apg_scores.insert("player1".to_string(), 7.3);
    apg_scores.insert("player3".to_string(), 5.5);
    apg_scores.insert("player4".to_string(), 9.0);
    apg_scores.insert("player5".to_string(), 4.2);
    cache.zset_add(apg_key, apg_scores).await?;
    
    println!("\nPlayer Stats:");
    println!("PPG: {:?}", cache.zset_range_with_scores::<String, _>(ppg_key, 0, -1).await?);
    println!("RPG: {:?}", cache.zset_range_with_scores::<String, _>(rpg_key, 0, -1).await?);
    println!("APG: {:?}", cache.zset_range_with_scores::<String, _>(apg_key, 0, -1).await?);
    
    // Union operation: Combine PPG and RPG, taking max score
    println!("\nPerforming union operation (MAX) between PPG and RPG");
    let dest_key_max = "player:stats:ppg_or_rpg_max";
    let count = cache.zset_union_store(
        dest_key_max,
        vec![ppg_key, rpg_key],
        &[1.0, 1.0],
        SortedSetOperations::AggregateMax
    ).await?;
    
    println!("Union (MAX) result with {} members:", count);
    let result = cache.zset_range_with_scores::<String, _>(dest_key_max, 0, -1).await?;
    for (member, score) in result {
        println!("- {}: {}", member, score);
    }
    
    // Union operation: Combine PPG and RPG, summing scores
    println!("\nPerforming union operation (SUM) between PPG and RPG");
    let dest_key_sum = "player:stats:ppg_or_rpg_sum";
    cache.zset_union_store(
        dest_key_sum,
        vec![ppg_key, rpg_key],
        &[1.0, 1.0],
        SortedSetOperations::AggregateSum
    ).await?;
    
    println!("Union (SUM) result:");
    let result = cache.zset_range_with_scores::<String, _>(dest_key_sum, 0, -1).await?;
    for (member, score) in result {
        println!("- {}: {}", member, score);
    }
    
    // Weighted union: Combine PPG (weighted 0.7) and APG (weighted 0.3)
    println!("\nPerforming weighted union between PPG (weight 0.7) and APG (weight 0.3)");
    let dest_key_weighted = "player:stats:ppg_apg_weighted";
    cache.zset_union_store(
        dest_key_weighted,
        vec![ppg_key, apg_key],
        &[0.7, 0.3],
        SortedSetOperations::AggregateSum
    ).await?;
    
    println!("Weighted union result (offensive rating):");
    let result = cache.zset_rev_range_with_scores::<String, _>(dest_key_weighted, 0, -1).await?;
    for (i, (member, score)) in result.iter().enumerate() {
        println!("{}. {}: {:.1}", i + 1, member, score);
    }
    
    // Intersection: Players who have stats in all three categories
    println!("\nPerforming intersection between PPG, RPG, and APG");
    let dest_key_intersect = "player:stats:all_categories";
    let count = cache.zset_intersection_store(
        dest_key_intersect,
        vec![ppg_key, rpg_key, apg_key],
        &[1.0, 1.0, 1.0],
        SortedSetOperations::AggregateSum
    ).await?;
    
    println!("Players with stats in all categories ({}), with sum of stats:", count);
    let result = cache.zset_rev_range_with_scores::<String, _>(dest_key_intersect, 0, -1).await?;
    for (member, score) in result {
        println!("- {}: {:.1}", member, score);
    }
    
    Ok(())
}

async fn cleanup(cache: &RedisCache) -> CacheResult<()> {
    // Clean up all the keys created in this example
    println!("\nCleaning up example keys...");
    
    let keys = vec![
        "simple_zset",
        "temperature_readings",
        "student_scores",
        "game_leaderboard",
        "stats:ppg:2024-25",
        "award:mvp:2024-25",
        "player:stats:ppg",
        "player:stats:rpg",
        "player:stats:apg",
        "player:stats:ppg_or_rpg_max",
        "player:stats:ppg_or_rpg_sum",
        "player:stats:ppg_apg_weighted",
        "player:stats:all_categories",
    ];
    
    for key in keys {
        let _ = cache.delete(key).await;
    }
    
    println!("Cleanup complete.");
    Ok(())
} 