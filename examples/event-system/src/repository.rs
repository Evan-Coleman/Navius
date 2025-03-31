//! Event System Repository
//!
//! This file contains the repository implementation for the Event System integration example.

use navius_core::error::Result;
use navius_db::{connection::ConnectionManager, repository::Repository};
use navius_db_postgres::{connection::PostgresConnectionManager, query::PostgresQuery};
use tracing::info;

use crate::models::EventRecord;

/// Event Repository for persisting events to the database
pub struct EventRepository {
    connection_manager: PostgresConnectionManager,
}

impl EventRepository {
    pub fn new(connection_manager: PostgresConnectionManager) -> Self {
        Self { connection_manager }
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection_manager.get_connection().await?;

        // Create events table
        let create_table_query = PostgresQuery::new(
            "CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY,
                event_type VARCHAR(255) NOT NULL,
                payload TEXT NOT NULL,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                source VARCHAR(255) NOT NULL
            )",
        );

        create_table_query.execute_update(&conn, &[]).await?;
        info!("Event database schema initialized successfully");

        Ok(())
    }

    pub async fn save_event(&self, record: &EventRecord) -> Result<()> {
        let conn = self.connection_manager.get_connection().await?;

        let query = PostgresQuery::new(
            "INSERT INTO events (id, event_type, payload, timestamp, source) 
             VALUES ($1, $2, $3, $4, $5)",
        );

        query
            .execute_update(
                &conn,
                &[
                    &record.id,
                    &record.event_type,
                    &record.payload,
                    &record.timestamp,
                    &record.source,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn get_events(&self, limit: i32) -> Result<Vec<EventRecord>> {
        let conn = self.connection_manager.get_connection().await?;

        let query = PostgresQuery::new(
            "SELECT id, event_type, payload, timestamp, source 
             FROM events 
             ORDER BY timestamp DESC 
             LIMIT $1",
        );

        let rows = query.execute_query(&conn, &[&limit]).await?;

        let events = rows
            .into_iter()
            .map(|row| EventRecord {
                id: row.get("id"),
                event_type: row.get("event_type"),
                payload: row.get("payload"),
                timestamp: row.get("timestamp"),
                source: row.get("source"),
            })
            .collect();

        Ok(events)
    }

    pub async fn get_events_by_type(
        &self,
        event_type: &str,
        limit: i32,
    ) -> Result<Vec<EventRecord>> {
        let conn = self.connection_manager.get_connection().await?;

        let query = PostgresQuery::new(
            "SELECT id, event_type, payload, timestamp, source 
             FROM events 
             WHERE event_type = $1
             ORDER BY timestamp DESC 
             LIMIT $2",
        );

        let rows = query.execute_query(&conn, &[&event_type, &limit]).await?;

        let events = rows
            .into_iter()
            .map(|row| EventRecord {
                id: row.get("id"),
                event_type: row.get("event_type"),
                payload: row.get("payload"),
                timestamp: row.get("timestamp"),
                source: row.get("source"),
            })
            .collect();

        Ok(events)
    }
}
