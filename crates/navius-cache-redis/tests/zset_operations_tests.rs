use navius_cache::{Cache, CacheOperations};
use navius_cache_redis::{RedisCache, RedisCacheConfig};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Import common test utilities
mod common;
use common::start_redis_server;

// Test user for zset operations
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestUser {
    id: u64,
    username: String,
}

impl std::fmt::Display for TestUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User({})", self.username)
    }
}

// Create test configuration with a short default TTL
fn create_test_config(redis_url: &str) -> RedisCacheConfig {
    let mut config = RedisCacheConfig::new(redis_url);
    config.default_ttl = Some(Duration::from_secs(30));
    config.key_prefix = Some("test".to_string());
    config.connection_timeout = Duration::from_secs(2);
    config
}

#[tokio::test]
async fn test_zset_add_and_length() {
    // Start Redis server
    let redis_url = start_redis_server();

    // Create cache with test configuration
    let config = create_test_config(&redis_url);
    let cache = navius_cache_redis::new(config).await.unwrap();

    // Clear cache before test
    cache.clear().await.unwrap();

    // Create test data
    let users = vec![
        (
            100.5,
            TestUser {
                id: 1,
                username: "alice".to_string(),
            },
        ),
        (
            85.0,
            TestUser {
                id: 2,
                username: "bob".to_string(),
            },
        ),
        (
            120.0,
            TestUser {
                id: 3,
                username: "charlie".to_string(),
            },
        ),
    ];

    // Test adding members to a sorted set
    let added = cache.zset_add("leaderboard", users).await.unwrap();
    assert_eq!(added, 3, "Should add 3 new members");

    // Test getting the length of a sorted set
    let length = cache.zset_length("leaderboard").await.unwrap();
    assert_eq!(length, 3, "Sorted set should have 3 members");

    // Test adding duplicate members (should update scores)
    let updated_users = vec![
        (
            110.5,
            TestUser {
                id: 1,
                username: "alice".to_string(),
            },
        ),
        (
            95.0,
            TestUser {
                id: 2,
                username: "bob".to_string(),
            },
        ),
    ];
    let added = cache.zset_add("leaderboard", updated_users).await.unwrap();
    assert_eq!(added, 0, "Should not add new members, only update scores");

    // Length should still be 3
    let length = cache.zset_length("leaderboard").await.unwrap();
    assert_eq!(length, 3, "Sorted set should still have 3 members");
}

#[tokio::test]
async fn test_zset_range_by_score() {
    // Start Redis server
    let redis_url = start_redis_server();

    // Create cache with test configuration
    let config = create_test_config(&redis_url);
    let cache = navius_cache_redis::new(config).await.unwrap();

    // Clear cache before test
    cache.clear().await.unwrap();

    // Create test data
    let users = vec![
        (
            50.0,
            TestUser {
                id: 1,
                username: "alice".to_string(),
            },
        ),
        (
            75.0,
            TestUser {
                id: 2,
                username: "bob".to_string(),
            },
        ),
        (
            100.0,
            TestUser {
                id: 3,
                username: "charlie".to_string(),
            },
        ),
        (
            125.0,
            TestUser {
                id: 4,
                username: "dave".to_string(),
            },
        ),
        (
            150.0,
            TestUser {
                id: 5,
                username: "eve".to_string(),
            },
        ),
    ];

    // Add members to sorted set
    cache.zset_add("scores", users).await.unwrap();

    // Test zset_range_by_score
    let result: Vec<TestUser> = cache
        .zset_range_by_score("scores", 70.0, 130.0)
        .await
        .unwrap();
    assert_eq!(
        result.len(),
        3,
        "Should get 3 members in the score range 70-130"
    );
    assert_eq!(result[0].username, "bob", "First member should be bob");
    assert_eq!(
        result[1].username, "charlie",
        "Second member should be charlie"
    );
    assert_eq!(result[2].username, "dave", "Third member should be dave");

    // Test zset_range_by_score_with_scores
    let result_with_scores: Vec<(TestUser, f64)> = cache
        .zset_range_by_score_with_scores("scores", 70.0, 130.0)
        .await
        .unwrap();
    assert_eq!(
        result_with_scores.len(),
        3,
        "Should get 3 members with scores"
    );
    assert_eq!(
        result_with_scores[0].0.username, "bob",
        "First member should be bob"
    );
    assert_eq!(result_with_scores[0].1, 75.0, "Bob's score should be 75.0");
    assert_eq!(
        result_with_scores[1].0.username, "charlie",
        "Second member should be charlie"
    );
    assert_eq!(
        result_with_scores[1].1, 100.0,
        "Charlie's score should be 100.0"
    );
    assert_eq!(
        result_with_scores[2].0.username, "dave",
        "Third member should be dave"
    );
    assert_eq!(
        result_with_scores[2].1, 125.0,
        "Dave's score should be 125.0"
    );
}

#[tokio::test]
async fn test_zset_score_and_rank() {
    // Start Redis server
    let redis_url = start_redis_server();

    // Create cache with test configuration
    let config = create_test_config(&redis_url);
    let cache = navius_cache_redis::new(config).await.unwrap();

    // Clear cache before test
    cache.clear().await.unwrap();

    // Create test data
    let users = vec![
        (
            100.0,
            TestUser {
                id: 1,
                username: "alice".to_string(),
            },
        ),
        (
            200.0,
            TestUser {
                id: 2,
                username: "bob".to_string(),
            },
        ),
        (
            300.0,
            TestUser {
                id: 3,
                username: "charlie".to_string(),
            },
        ),
    ];

    // Add members to sorted set
    cache.zset_add("leaderboard", users).await.unwrap();

    // Test zset_score
    let alice = TestUser {
        id: 1,
        username: "alice".to_string(),
    };
    let score = cache.zset_score("leaderboard", &alice).await.unwrap();
    assert_eq!(score, Some(100.0), "Alice's score should be 100.0");

    // Test zset_score for non-existent member
    let dave = TestUser {
        id: 4,
        username: "dave".to_string(),
    };
    let score = cache.zset_score("leaderboard", &dave).await.unwrap();
    assert_eq!(score, None, "Dave should not have a score");

    // Test zset_rank
    let rank = cache.zset_rank("leaderboard", &alice).await.unwrap();
    assert_eq!(rank, Some(0), "Alice should be rank 0 (lowest score)");

    let bob = TestUser {
        id: 2,
        username: "bob".to_string(),
    };
    let rank = cache.zset_rank("leaderboard", &bob).await.unwrap();
    assert_eq!(rank, Some(1), "Bob should be rank 1");

    let charlie = TestUser {
        id: 3,
        username: "charlie".to_string(),
    };
    let rank = cache.zset_rank("leaderboard", &charlie).await.unwrap();
    assert_eq!(rank, Some(2), "Charlie should be rank 2 (highest score)");

    // Test zset_rank for non-existent member
    let rank = cache.zset_rank("leaderboard", &dave).await.unwrap();
    assert_eq!(rank, None, "Dave should not have a rank");
}

#[tokio::test]
async fn test_zset_increment() {
    // Start Redis server
    let redis_url = start_redis_server();

    // Create cache with test configuration
    let config = create_test_config(&redis_url);
    let cache = navius_cache_redis::new(config).await.unwrap();

    // Clear cache before test
    cache.clear().await.unwrap();

    // Add a single member to the sorted set
    let alice = TestUser {
        id: 1,
        username: "alice".to_string(),
    };
    let users = vec![(100.0, alice.clone())];
    cache.zset_add("scores", users).await.unwrap();

    // Test incrementing score
    let new_score = cache.zset_increment("scores", &alice, 25.5).await.unwrap();
    assert_eq!(new_score, 125.5, "Alice's new score should be 125.5");

    // Verify the new score
    let score = cache.zset_score("scores", &alice).await.unwrap();
    assert_eq!(
        score,
        Some(125.5),
        "Alice's score should be updated to 125.5"
    );

    // Test decrementing score
    let new_score = cache.zset_increment("scores", &alice, -50.0).await.unwrap();
    assert_eq!(new_score, 75.5, "Alice's new score should be 75.5");

    // Verify the new score
    let score = cache.zset_score("scores", &alice).await.unwrap();
    assert_eq!(score, Some(75.5), "Alice's score should be updated to 75.5");

    // Test incrementing score for non-existent member (should create member)
    let bob = TestUser {
        id: 2,
        username: "bob".to_string(),
    };
    let new_score = cache.zset_increment("scores", &bob, 50.0).await.unwrap();
    assert_eq!(new_score, 50.0, "Bob's score should be 50.0");

    // Verify Bob was added to the set
    let length = cache.zset_length("scores").await.unwrap();
    assert_eq!(length, 2, "Sorted set should now have 2 members");
}

#[tokio::test]
async fn test_edge_cases() {
    // Start Redis server
    let redis_url = start_redis_server();

    // Create cache with test configuration
    let config = create_test_config(&redis_url);
    let cache = navius_cache_redis::new(config).await.unwrap();

    // Clear cache before test
    cache.clear().await.unwrap();

    // Test empty zset
    let length = cache.zset_length("empty_set").await.unwrap();
    assert_eq!(length, 0, "Empty set should have 0 members");

    let members: Vec<String> = cache
        .zset_range_by_score("empty_set", 0.0, 100.0)
        .await
        .unwrap();
    assert_eq!(members.len(), 0, "Empty set should return no members");

    // Test adding empty list
    let empty_list: Vec<(f64, TestUser)> = vec![];
    let added = cache.zset_add("empty_add", empty_list).await.unwrap();
    assert_eq!(added, 0, "Adding empty list should add 0 members");

    // Test with complex objects
    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    struct ComplexUser {
        id: u64,
        username: String,
        tags: Vec<String>,
        metadata: std::collections::HashMap<String, String>,
    }

    let user = ComplexUser {
        id: 1,
        username: "complex_user".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        },
    };

    // Add complex object to sorted set
    let users = vec![(100.0, user.clone())];
    cache.zset_add("complex", users).await.unwrap();

    // Retrieve the complex object
    let result: Vec<ComplexUser> = cache
        .zset_range_by_score("complex", 0.0, 200.0)
        .await
        .unwrap();
    assert_eq!(result.len(), 1, "Should retrieve 1 complex object");
    assert_eq!(result[0], user, "Retrieved object should match original");
}

#[tokio::test]
async fn test_zset_count() {
    // Start Redis server
    let redis_url = start_redis_server();

    // Create cache with test configuration
    let config = create_test_config(&redis_url);
    let cache = navius_cache_redis::new(config).await.unwrap();

    // Clear cache before test
    cache.clear().await.unwrap();

    // Create test data
    let users = vec![
        (
            50.0,
            TestUser {
                id: 1,
                username: "alice".to_string(),
            },
        ),
        (
            75.0,
            TestUser {
                id: 2,
                username: "bob".to_string(),
            },
        ),
        (
            100.0,
            TestUser {
                id: 3,
                username: "charlie".to_string(),
            },
        ),
        (
            125.0,
            TestUser {
                id: 4,
                username: "dave".to_string(),
            },
        ),
        (
            150.0,
            TestUser {
                id: 5,
                username: "eve".to_string(),
            },
        ),
    ];

    // Add members to sorted set
    cache.zset_add("scores", users).await.unwrap();

    // Test zset_count
    let count = cache.zset_count("scores", 70.0, 130.0).await.unwrap();
    assert_eq!(count, 3, "Should find 3 members with scores between 70-130");

    // Test edge case - count at exact score points
    let count = cache.zset_count("scores", 75.0, 125.0).await.unwrap();
    assert_eq!(
        count, 3,
        "Should find 3 members with scores between 75-125 (inclusive)"
    );

    // Test edge case - count with out of range values
    let count = cache.zset_count("scores", 200.0, 300.0).await.unwrap();
    assert_eq!(
        count, 0,
        "Should find 0 members with scores between 200-300"
    );

    // Test edge case - count in non-existent key
    let count = cache.zset_count("nonexistent", 0.0, 100.0).await.unwrap();
    assert_eq!(count, 0, "Should return 0 for non-existent key");
}

#[tokio::test]
async fn test_zset_remove_range_by_rank() {
    // Start Redis server
    let redis_url = start_redis_server();

    // Create cache with test configuration
    let config = create_test_config(&redis_url);
    let cache = navius_cache_redis::new(config).await.unwrap();

    // Clear cache before test
    cache.clear().await.unwrap();

    // Create test data with 10 users with scores 10-100
    let mut users = Vec::new();
    for i in 1..=10 {
        users.push((
            i as f64 * 10.0,
            TestUser {
                id: i,
                username: format!("user_{}", i),
            },
        ));
    }

    // Add members to sorted set
    cache.zset_add("leaderboard", users).await.unwrap();

    // Verify initial length
    let length = cache.zset_length("leaderboard").await.unwrap();
    assert_eq!(length, 10, "Should have 10 members initially");

    // Remove members ranked 2-4 (0-based index)
    let removed = cache
        .zset_remove_range_by_rank("leaderboard", 2, 4)
        .await
        .unwrap();
    assert_eq!(removed, 3, "Should remove 3 members");

    // Verify new length
    let length = cache.zset_length("leaderboard").await.unwrap();
    assert_eq!(length, 7, "Should have 7 members after removal");

    // Get remaining members to verify correct removal
    let remaining: Vec<TestUser> = cache.zset_range("leaderboard", 0, -1).await.unwrap();
    assert_eq!(remaining.len(), 7, "Should have 7 members remaining");

    // Verify the correct members were removed (users 3, 4, and 5)
    // The remaining should be users 1, 2, 6, 7, 8, 9, 10
    let user_ids: Vec<u64> = remaining.iter().map(|u| u.id).collect();
    assert!(user_ids.contains(&1), "User 1 should remain");
    assert!(user_ids.contains(&2), "User 2 should remain");
    assert!(!user_ids.contains(&3), "User 3 should be removed");
    assert!(!user_ids.contains(&4), "User 4 should be removed");
    assert!(!user_ids.contains(&5), "User 5 should be removed");
    assert!(user_ids.contains(&6), "User 6 should remain");

    // Test edge case - remove from non-existent key
    let removed = cache
        .zset_remove_range_by_rank("nonexistent", 0, 5)
        .await
        .unwrap();
    assert_eq!(removed, 0, "Should remove 0 members from non-existent key");
}

#[tokio::test]
async fn test_zset_remove_range_by_score() {
    // Start Redis server
    let redis_url = start_redis_server();

    // Create cache with test configuration
    let config = create_test_config(&redis_url);
    let cache = navius_cache_redis::new(config).await.unwrap();

    // Clear cache before test
    cache.clear().await.unwrap();

    // Create test data with 10 users with scores 10-100
    let mut users = Vec::new();
    for i in 1..=10 {
        users.push((
            i as f64 * 10.0,
            TestUser {
                id: i,
                username: format!("user_{}", i),
            },
        ));
    }

    // Add members to sorted set
    cache.zset_add("leaderboard", users).await.unwrap();

    // Verify initial length
    let length = cache.zset_length("leaderboard").await.unwrap();
    assert_eq!(length, 10, "Should have 10 members initially");

    // Remove members with scores 30-60
    let removed = cache
        .zset_remove_range_by_score("leaderboard", 30.0, 60.0)
        .await
        .unwrap();
    assert_eq!(removed, 4, "Should remove 4 members");

    // Verify new length
    let length = cache.zset_length("leaderboard").await.unwrap();
    assert_eq!(length, 6, "Should have 6 members after removal");

    // Get remaining members to verify correct removal
    let remaining: Vec<TestUser> = cache.zset_range("leaderboard", 0, -1).await.unwrap();
    assert_eq!(remaining.len(), 6, "Should have 6 members remaining");

    // Verify the correct members were removed (users with scores 30, 40, 50, 60)
    let user_ids: Vec<u64> = remaining.iter().map(|u| u.id).collect();
    assert!(user_ids.contains(&1), "User 1 (score 10) should remain");
    assert!(user_ids.contains(&2), "User 2 (score 20) should remain");
    assert!(
        !user_ids.contains(&3),
        "User 3 (score 30) should be removed"
    );
    assert!(
        !user_ids.contains(&4),
        "User 4 (score 40) should be removed"
    );
    assert!(
        !user_ids.contains(&5),
        "User 5 (score 50) should be removed"
    );
    assert!(
        !user_ids.contains(&6),
        "User 6 (score 60) should be removed"
    );
    assert!(user_ids.contains(&7), "User 7 (score 70) should remain");

    // Test edge case - remove from non-existent key
    let removed = cache
        .zset_remove_range_by_score("nonexistent", 0.0, 100.0)
        .await
        .unwrap();
    assert_eq!(removed, 0, "Should remove 0 members from non-existent key");

    // Test edge case - remove with out of range scores
    let removed = cache
        .zset_remove_range_by_score("leaderboard", 200.0, 300.0)
        .await
        .unwrap();
    assert_eq!(
        removed, 0,
        "Should remove 0 members with out of range scores"
    );
}
