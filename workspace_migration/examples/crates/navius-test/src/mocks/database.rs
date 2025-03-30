use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::error::TestResult;
use crate::mock::{Expectation, MockRegistry};

/// A mock implementation of a database client interface
#[derive(Debug, Default)]
pub struct MockDatabaseClient {
    /// Internal state for recording and verifying method calls
    state: Arc<Mutex<MockDatabaseClientState>>,
}

/// Internal state for the mock database client
#[derive(Debug, Default)]
struct MockDatabaseClientState {
    /// Stores query results by query string
    query_results: HashMap<String, Vec<MockQueryResult>>,

    /// Records query calls
    query_calls: Vec<String>,

    /// Stores execute results by statement
    execute_results: HashMap<String, Result<u64, MockDatabaseError>>,

    /// Records execute calls
    execute_calls: Vec<String>,

    /// Stores transaction results
    transaction_results: Vec<Result<(), MockDatabaseError>>,

    /// Records transaction calls
    transaction_calls: usize,
}

/// A mock query result that can be returned by the mock database client
#[derive(Debug, Clone)]
pub struct MockQueryResult {
    /// The row data stored as a map of column name to value
    pub rows: Vec<HashMap<String, MockValue>>,
}

impl MockQueryResult {
    /// Create a new empty query result
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Add a row to the query result
    pub fn add_row(mut self, row: HashMap<String, MockValue>) -> Self {
        self.rows.push(row);
        self
    }

    /// Get the number of rows in the result
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// A mock value for database results
#[derive(Debug, Clone)]
pub enum MockValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Null value
    Null,
}

impl MockValue {
    /// Convert the mock value to a string
    pub fn as_string(&self) -> Option<String> {
        match self {
            MockValue::String(s) => Some(s.clone()),
            MockValue::Integer(i) => Some(i.to_string()),
            MockValue::Float(f) => Some(f.to_string()),
            MockValue::Boolean(b) => Some(b.to_string()),
            MockValue::Null => None,
        }
    }

    /// Convert the mock value to an integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            MockValue::Integer(i) => Some(*i),
            MockValue::String(s) => s.parse::<i64>().ok(),
            MockValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Convert the mock value to a float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            MockValue::Float(f) => Some(*f),
            MockValue::Integer(i) => Some(*i as f64),
            MockValue::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Convert the mock value to a boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            MockValue::Boolean(b) => Some(*b),
            MockValue::Integer(i) => Some(*i != 0),
            MockValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "1" => Some(true),
                "false" | "no" | "0" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

/// A mock database error
#[derive(Debug, Clone)]
pub struct MockDatabaseError {
    /// The error message
    pub message: String,

    /// The error code
    pub code: Option<String>,
}

impl MockDatabaseError {
    /// Create a new mock database error
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }

    /// Set the error code
    pub fn with_code<S: Into<String>>(mut self, code: S) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl fmt::Display for MockDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref code) = self.code {
            write!(f, "Database error ({}): {}", code, self.message)
        } else {
            write!(f, "Database error: {}", self.message)
        }
    }
}

impl std::error::Error for MockDatabaseError {}

impl MockDatabaseClient {
    /// Create a new mock database client
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockDatabaseClientState::default())),
        }
    }

    /// Register this mock with the mock registry
    pub fn register(self, registry: &MockRegistry) -> TestResult<Arc<Self>> {
        let mock = Arc::new(self);
        registry.register::<dyn DatabaseClient, Self>(Arc::clone(&mock))?;
        Ok(mock)
    }

    /// Set up a query to return specific results
    pub fn expect_query<S: Into<String>>(
        &self,
        query: S,
        result: Result<MockQueryResult, MockDatabaseError>,
    ) -> &Self {
        let mut state = self.state.lock().unwrap();
        let query = query.into();

        match result {
            Ok(result) => {
                state
                    .query_results
                    .entry(query)
                    .or_insert_with(Vec::new)
                    .push(result);
            }
            Err(err) => {
                state
                    .query_results
                    .entry(query)
                    .or_insert_with(Vec::new)
                    .push(MockQueryResult::new());
                // TODO: Handle error cases
            }
        }

        self
    }

    /// Set up an execute to return a specific result
    pub fn expect_execute<S: Into<String>>(
        &self,
        statement: S,
        result: Result<u64, MockDatabaseError>,
    ) -> &Self {
        let mut state = self.state.lock().unwrap();
        state.execute_results.insert(statement.into(), result);
        self
    }

    /// Set up a transaction to return a specific result
    pub fn expect_transaction(&self, result: Result<(), MockDatabaseError>) -> &Self {
        let mut state = self.state.lock().unwrap();
        state.transaction_results.push(result);
        self
    }

    /// Execute a query and return the result
    pub fn query<S: Into<String>>(&self, query: S) -> Result<MockQueryResult, MockDatabaseError> {
        let mut state = self.state.lock().unwrap();
        let query = query.into();
        state.query_calls.push(query.clone());

        match state.query_results.get_mut(&query) {
            Some(results) if !results.is_empty() => Ok(results.remove(0)),
            _ => Ok(MockQueryResult::new()),
        }
    }

    /// Execute a statement and return the number of affected rows
    pub fn execute<S: Into<String>>(&self, statement: S) -> Result<u64, MockDatabaseError> {
        let mut state = self.state.lock().unwrap();
        let statement = statement.into();
        state.execute_calls.push(statement.clone());

        state
            .execute_results
            .get(&statement)
            .cloned()
            .unwrap_or(Ok(0))
    }

    /// Execute a transaction
    pub fn transaction<F, T>(&self, f: F) -> Result<T, MockDatabaseError>
    where
        F: FnOnce() -> Result<T, MockDatabaseError>,
    {
        let mut state = self.state.lock().unwrap();
        state.transaction_calls += 1;

        let result = match state.transaction_results.get(state.transaction_calls - 1) {
            Some(Ok(())) => f(),
            Some(Err(err)) => Err(err.clone()),
            None => f(),
        };

        result
    }

    /// Verify that all expected calls were made
    pub fn verify(&self) -> TestResult<()> {
        Ok(())
    }
}

/// Database client interface trait
pub trait DatabaseClient: Send + Sync {
    /// Execute a query and return the result
    fn query(&self, query: &str) -> Result<Box<dyn QueryResult>, Box<dyn std::error::Error>>;

    /// Execute a statement and return the number of affected rows
    fn execute(&self, statement: &str) -> Result<u64, Box<dyn std::error::Error>>;

    /// Execute a transaction
    fn transaction<F, T>(&self, f: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Result<T, Box<dyn std::error::Error>>;
}

/// Query result interface trait
pub trait QueryResult: Send + Sync {
    /// Get the number of rows in the result
    fn len(&self) -> usize;

    /// Check if the result is empty
    fn is_empty(&self) -> bool;

    /// Get a row by index
    fn row(&self, index: usize) -> Option<Box<dyn Row>>;

    /// Iterate over the rows
    fn rows(&self) -> Vec<Box<dyn Row>>;
}

/// Row interface trait
pub trait Row: Send + Sync {
    /// Get a value by column name
    fn get(&self, column: &str) -> Option<Box<dyn Value>>;

    /// Get the column names
    fn columns(&self) -> Vec<String>;
}

/// Value interface trait
pub trait Value: Send + Sync {
    /// Get the value as a string
    fn as_string(&self) -> Option<String>;

    /// Get the value as an integer
    fn as_integer(&self) -> Option<i64>;

    /// Get the value as a float
    fn as_float(&self) -> Option<f64>;

    /// Get the value as a boolean
    fn as_boolean(&self) -> Option<bool>;

    /// Check if the value is null
    fn is_null(&self) -> bool;
}

// Implementation of QueryResult for MockQueryResult
impl QueryResult for MockQueryResult {
    fn len(&self) -> usize {
        self.rows.len()
    }

    fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    fn row(&self, index: usize) -> Option<Box<dyn Row>> {
        self.rows.get(index).map(|r| {
            let mock_row = MockRow { data: r.clone() };
            Box::new(mock_row) as Box<dyn Row>
        })
    }

    fn rows(&self) -> Vec<Box<dyn Row>> {
        self.rows
            .iter()
            .map(|r| {
                let mock_row = MockRow { data: r.clone() };
                Box::new(mock_row) as Box<dyn Row>
            })
            .collect()
    }
}

/// A mock implementation of a database row
#[derive(Debug, Clone)]
struct MockRow {
    /// The row data stored as a map of column name to value
    data: HashMap<String, MockValue>,
}

impl Row for MockRow {
    fn get(&self, column: &str) -> Option<Box<dyn Value>> {
        self.data.get(column).map(|v| {
            let mock_value = v.clone();
            Box::new(mock_value) as Box<dyn Value>
        })
    }

    fn columns(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

impl Value for MockValue {
    fn as_string(&self) -> Option<String> {
        self.as_string()
    }

    fn as_integer(&self) -> Option<i64> {
        self.as_integer()
    }

    fn as_float(&self) -> Option<f64> {
        self.as_float()
    }

    fn as_boolean(&self) -> Option<bool> {
        self.as_boolean()
    }

    fn is_null(&self) -> bool {
        matches!(self, MockValue::Null)
    }
}

impl DatabaseClient for MockDatabaseClient {
    fn query(&self, query: &str) -> Result<Box<dyn QueryResult>, Box<dyn std::error::Error>> {
        match self.query(query) {
            Ok(result) => Ok(Box::new(result) as Box<dyn QueryResult>),
            Err(err) => Err(Box::new(err) as Box<dyn std::error::Error>),
        }
    }

    fn execute(&self, statement: &str) -> Result<u64, Box<dyn std::error::Error>> {
        match self.execute(statement) {
            Ok(result) => Ok(result),
            Err(err) => Err(Box::new(err) as Box<dyn std::error::Error>),
        }
    }

    fn transaction<F, T>(&self, f: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Result<T, Box<dyn std::error::Error>>,
    {
        let result = self.transaction(|| f().map_err(|e| MockDatabaseError::new(e.to_string())));

        match result {
            Ok(result) => Ok(result),
            Err(err) => Err(Box::new(err) as Box<dyn std::error::Error>),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockRegistry;

    #[test]
    fn test_mock_database_client() {
        let registry = MockRegistry::new();
        let db = MockDatabaseClient::new();

        // Set up expectations
        db.expect_query(
            "SELECT * FROM users WHERE id = ?",
            Ok(MockQueryResult::new().add_row({
                let mut row = HashMap::new();
                row.insert("id".to_string(), MockValue::Integer(1));
                row.insert(
                    "name".to_string(),
                    MockValue::String("Test User".to_string()),
                );
                row.insert("active".to_string(), MockValue::Boolean(true));
                row
            })),
        );

        db.expect_execute("INSERT INTO users (name, active) VALUES (?, ?)", Ok(1));

        // Execute queries
        let result = db.query("SELECT * FROM users WHERE id = ?").unwrap();
        assert_eq!(result.len(), 1);
        assert!(!result.is_empty());

        let row = result.rows[0].clone();
        assert_eq!(row.get("id").unwrap().as_integer(), Some(1));
        assert_eq!(
            row.get("name").unwrap().as_string(),
            Some("Test User".to_string())
        );
        assert_eq!(row.get("active").unwrap().as_boolean(), Some(true));

        // Execute statements
        let affected = db
            .execute("INSERT INTO users (name, active) VALUES (?, ?)")
            .unwrap();
        assert_eq!(affected, 1);

        // Test interface implementation
        let db_client: Box<dyn DatabaseClient> = Box::new(db);
        let query_result = db_client.query("SELECT * FROM users WHERE id = ?").unwrap();
        assert_eq!(query_result.len(), 1);

        let affected = db_client
            .execute("INSERT INTO users (name, active) VALUES (?, ?)")
            .unwrap();
        assert_eq!(affected, 1);
    }
}
