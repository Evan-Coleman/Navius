use crate::error::DatabaseResult;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

/// Sort direction for ORDER BY clauses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order (A-Z, 0-9)
    Ascending,
    /// Descending order (Z-A, 9-0)
    Descending,
}

impl SortDirection {
    /// Convert to SQL string
    pub fn as_sql(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        }
    }
}

/// Query builder trait
pub trait QueryBuilder {
    /// Add a WHERE clause to the query
    fn where_eq<T>(&mut self, column: &str, value: T) -> &mut Self
    where
        T: 'static + Send + Sync + Clone;

    /// Add an AND clause to the query
    fn and_eq<T>(&mut self, column: &str, value: T) -> &mut Self
    where
        T: 'static + Send + Sync + Clone;

    /// Add an OR clause to the query
    fn or_eq<T>(&mut self, column: &str, value: T) -> &mut Self
    where
        T: 'static + Send + Sync + Clone;

    /// Add an ORDER BY clause to the query
    fn order_by(&mut self, column: &str, direction: SortDirection) -> &mut Self;

    /// Add a LIMIT clause to the query
    fn limit(&mut self, limit: i64) -> &mut Self;

    /// Add an OFFSET clause to the query
    fn offset(&mut self, offset: i64) -> &mut Self;

    /// Build the SQL query string
    fn build_sql(&self) -> String;
}

/// Query executor trait
#[async_trait]
pub trait QueryExecutor<T: DeserializeOwned>: Send + Sync + 'static {
    /// Execute the query and return the results
    async fn execute(&self) -> DatabaseResult<Vec<T>>;

    /// Execute the query and return a single result
    async fn execute_one(&self) -> DatabaseResult<T>;

    /// Execute the query and return the number of affected rows
    async fn execute_update(&self) -> DatabaseResult<u64>;

    /// Count the number of records that would be returned by the query
    async fn count(&self) -> DatabaseResult<i64>;
}

/// Simple query builder for common database operations
pub struct Query<T> {
    table: String,
    columns: Vec<String>,
    where_clauses: Vec<String>,
    order_by: Option<(String, SortDirection)>,
    limit_value: Option<i64>,
    offset_value: Option<i64>,
    _phantom: PhantomData<T>,
    param_count: usize,
}

impl<T> Query<T> {
    /// Create a new query for a table
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: vec!["*".to_string()],
            where_clauses: Vec::new(),
            order_by: None,
            limit_value: None,
            offset_value: None,
            _phantom: PhantomData,
            param_count: 0,
        }
    }

    /// Select specific columns
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|c| c.to_string()).collect();
        self
    }
}

impl<T> QueryBuilder for Query<T> {
    fn where_eq<V>(&mut self, column: &str, _value: V) -> &mut Self
    where
        V: 'static + Send + Sync + Clone,
    {
        self.param_count += 1;
        let param_index = self.param_count;
        self.where_clauses
            .push(format!("{} = ${}", column, param_index));
        self
    }

    fn and_eq<V>(&mut self, column: &str, _value: V) -> &mut Self
    where
        V: 'static + Send + Sync + Clone,
    {
        if self.where_clauses.is_empty() {
            return self.where_eq(column, _value);
        }

        self.param_count += 1;
        let param_index = self.param_count;
        self.where_clauses
            .push(format!("AND {} = ${}", column, param_index));
        self
    }

    fn or_eq<V>(&mut self, column: &str, _value: V) -> &mut Self
    where
        V: 'static + Send + Sync + Clone,
    {
        if self.where_clauses.is_empty() {
            return self.where_eq(column, _value);
        }

        self.param_count += 1;
        let param_index = self.param_count;
        self.where_clauses
            .push(format!("OR {} = ${}", column, param_index));
        self
    }

    fn order_by(&mut self, column: &str, direction: SortDirection) -> &mut Self {
        self.order_by = Some((column.to_string(), direction));
        self
    }

    fn limit(&mut self, limit: i64) -> &mut Self {
        self.limit_value = Some(limit);
        self
    }

    fn offset(&mut self, offset: i64) -> &mut Self {
        self.offset_value = Some(offset);
        self
    }

    fn build_sql(&self) -> String {
        let columns = self.columns.join(", ");
        let mut query = format!("SELECT {} FROM {}", columns, self.table);

        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" "));
        }

        if let Some((column, direction)) = &self.order_by {
            query.push_str(&format!(" ORDER BY {} {}", column, direction.as_sql()));
        }

        if let Some(limit) = self.limit_value {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset_value {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestEntity {
        id: i32,
        name: String,
    }

    #[test]
    fn test_query_build() {
        let mut query = Query::<TestEntity>::new("test_table");

        query
            .where_eq("name", "test")
            .and_eq("age", 18)
            .order_by("name", SortDirection::Ascending)
            .limit(10)
            .offset(20);

        let sql = query.build_sql();

        assert_eq!(
            sql,
            "SELECT * FROM test_table WHERE name = $1 AND age = $2 ORDER BY name ASC LIMIT 10 OFFSET 20"
        );
    }

    #[test]
    fn test_query_select_columns() {
        let query = Query::<TestEntity>::new("test_table").select(&["id", "name", "age"]);

        let sql = query.build_sql();

        assert_eq!(sql, "SELECT id, name, age FROM test_table");
    }

    #[test]
    fn test_query_or_clause() {
        let mut query = Query::<TestEntity>::new("test_table");

        query.where_eq("name", "test1").or_eq("name", "test2");

        let sql = query.build_sql();

        assert_eq!(sql, "SELECT * FROM test_table WHERE name = $1 OR name = $2");
    }
}
