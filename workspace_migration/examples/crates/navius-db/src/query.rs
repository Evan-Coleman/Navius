use crate::error::DatabaseResult;
use async_trait::async_trait;
use navius_test::error::{TestResult, assert_eq};
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

/// Comparison operator for WHERE clauses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    /// Equal to (=)
    Equal,
    /// Not equal to (!=)
    NotEqual,
    /// Greater than (>)
    GreaterThan,
    /// Greater than or equal to (>=)
    GreaterThanOrEqual,
    /// Less than (<)
    LessThan,
    /// Less than or equal to (<=)
    LessThanOrEqual,
    /// Like pattern match (LIKE)
    Like,
    /// Case-insensitive like (ILIKE)
    ILike,
    /// In a list of values (IN)
    In,
    /// Not in a list of values (NOT IN)
    NotIn,
    /// Is NULL
    IsNull,
    /// Is NOT NULL
    IsNotNull,
}

impl ComparisonOperator {
    /// Convert to SQL string
    pub fn as_sql(&self) -> &'static str {
        match self {
            ComparisonOperator::Equal => "=",
            ComparisonOperator::NotEqual => "!=",
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::LessThanOrEqual => "<=",
            ComparisonOperator::Like => "LIKE",
            ComparisonOperator::ILike => "ILIKE",
            ComparisonOperator::In => "IN",
            ComparisonOperator::NotIn => "NOT IN",
            ComparisonOperator::IsNull => "IS NULL",
            ComparisonOperator::IsNotNull => "IS NOT NULL",
        }
    }
}

/// Logical operator for WHERE clauses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOperator {
    /// AND operator
    And,
    /// OR operator
    Or,
    /// NOT operator
    Not,
}

impl LogicalOperator {
    /// Convert to SQL string
    pub fn as_sql(&self) -> &'static str {
        match self {
            LogicalOperator::And => "AND",
            LogicalOperator::Or => "OR",
            LogicalOperator::Not => "NOT",
        }
    }
}

/// Pagination strategy to use for query results
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaginationStrategy {
    /// Offset-based pagination (LIMIT/OFFSET)
    Offset {
        /// Number of records to return
        limit: i64,
        /// Number of records to skip
        offset: i64,
    },
    /// Cursor-based pagination (WHERE id > cursor ORDER BY id LIMIT n)
    Cursor {
        /// Number of records to return
        limit: i64,
        /// Column to use for cursor
        column: String,
        /// Cursor value to start from
        cursor: Option<String>,
        /// Direction to scan (true for forward, false for backward)
        forward: bool,
    },
}

/// Filter condition for queries
#[derive(Debug, Clone)]
pub enum FilterCondition {
    /// Simple condition (column OPERATOR value)
    Simple {
        /// Column name to filter on
        column: String,
        /// Operator to use
        operator: ComparisonOperator,
        /// Parameter index for the value
        param_index: usize,
    },
    /// Logical group of conditions
    Group {
        /// Operator for this group
        operator: LogicalOperator,
        /// Nested conditions
        conditions: Vec<FilterCondition>,
    },
    /// Raw SQL condition (injected as-is)
    Raw(String),
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

    /// Add a WHERE clause with a comparison operator
    fn where_comp<T>(&mut self, column: &str, operator: ComparisonOperator, value: T) -> &mut Self
    where
        T: 'static + Send + Sync + Clone;

    /// Add a logical NOT to the next condition
    fn not(&mut self) -> &mut Self;

    /// Add a raw WHERE condition
    fn where_raw(&mut self, sql: &str) -> &mut Self;

    /// Start a new condition group with AND
    fn and_group(&mut self) -> &mut Self;

    /// Start a new condition group with OR
    fn or_group(&mut self) -> &mut Self;

    /// End the current condition group
    fn end_group(&mut self) -> &mut Self;

    /// Add an ORDER BY clause to the query
    fn order_by(&mut self, column: &str, direction: SortDirection) -> &mut Self;

    /// Add multiple ORDER BY clauses
    fn order_by_multiple(&mut self, columns: &[(&str, SortDirection)]) -> &mut Self;

    /// Add a LIMIT clause to the query
    fn limit(&mut self, limit: i64) -> &mut Self;

    /// Add an OFFSET clause to the query
    fn offset(&mut self, offset: i64) -> &mut Self;

    /// Add pagination to the query
    fn paginate(&mut self, strategy: PaginationStrategy) -> &mut Self;

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
    conditions: Vec<FilterCondition>,
    order_by_clauses: Vec<(String, SortDirection)>,
    limit_value: Option<i64>,
    offset_value: Option<i64>,
    _phantom: PhantomData<T>,
    param_count: usize,
    group_stack: Vec<(LogicalOperator, Vec<FilterCondition>)>,
    negate_next: bool,
}

impl<T> Query<T> {
    /// Create a new query for a table
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: vec!["*".to_string()],
            conditions: Vec::new(),
            order_by_clauses: Vec::new(),
            limit_value: None,
            offset_value: None,
            _phantom: PhantomData,
            param_count: 0,
            group_stack: Vec::new(),
            negate_next: false,
        }
    }

    /// Select specific columns
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Get the next parameter index
    fn next_param_index(&mut self) -> usize {
        self.param_count += 1;
        self.param_count
    }

    /// Add a condition to the current scope
    fn add_condition(&mut self, condition: FilterCondition) {
        if self.negate_next {
            let condition = FilterCondition::Group {
                operator: LogicalOperator::Not,
                conditions: vec![condition],
            };
            self.negate_next = false;

            if let Some((_, conditions)) = self.group_stack.last_mut() {
                conditions.push(condition);
            } else {
                self.conditions.push(condition);
            }
        } else {
            if let Some((_, conditions)) = self.group_stack.last_mut() {
                conditions.push(condition);
            } else {
                self.conditions.push(condition);
            }
        }
    }
}

impl<T> QueryBuilder for Query<T> {
    fn where_eq<V>(&mut self, column: &str, _value: V) -> &mut Self
    where
        V: 'static + Send + Sync + Clone,
    {
        let param_index = self.next_param_index();
        let condition = FilterCondition::Simple {
            column: column.to_string(),
            operator: ComparisonOperator::Equal,
            param_index,
        };

        self.add_condition(condition);
        self
    }

    fn and_eq<V>(&mut self, column: &str, _value: V) -> &mut Self
    where
        V: 'static + Send + Sync + Clone,
    {
        if self.conditions.is_empty() && self.group_stack.is_empty() {
            return self.where_eq(column, _value);
        }

        let param_index = self.next_param_index();
        let condition = FilterCondition::Simple {
            column: column.to_string(),
            operator: ComparisonOperator::Equal,
            param_index,
        };

        self.add_condition(condition);
        self
    }

    fn or_eq<V>(&mut self, column: &str, _value: V) -> &mut Self
    where
        V: 'static + Send + Sync + Clone,
    {
        if self.conditions.is_empty() && self.group_stack.is_empty() {
            return self.where_eq(column, _value);
        }

        // Create a new OR condition
        let param_index = self.next_param_index();
        let condition = FilterCondition::Simple {
            column: column.to_string(),
            operator: ComparisonOperator::Equal,
            param_index,
        };

        // For OR conditions outside of a group, we need to create a logical group
        if self.group_stack.is_empty() {
            // Take the last condition from the main conditions list
            if let Some(last_condition) = self.conditions.pop() {
                // Create a new OR group with the last condition and the new one
                let or_group = FilterCondition::Group {
                    operator: LogicalOperator::Or,
                    conditions: vec![last_condition, condition],
                };

                // Add the OR group to main conditions
                self.conditions.push(or_group);
                return self;
            }
        }

        // If we're in a group and it's an OR group, add the condition normally
        if let Some((LogicalOperator::Or, conditions)) = self.group_stack.last_mut() {
            conditions.push(condition);
        } else {
            // If we're in an AND group, we need to handle differently
            if let Some((LogicalOperator::And, _)) = self.group_stack.last() {
                // Start an OR subgroup
                self.or_group();
                // Add the condition
                self.add_condition(condition);
                // End the OR subgroup
                self.end_group();
            } else {
                // Fallback: treat as normal condition
                self.add_condition(condition);
            }
        }

        self
    }

    fn where_comp<V>(&mut self, column: &str, operator: ComparisonOperator, _value: V) -> &mut Self
    where
        V: 'static + Send + Sync + Clone,
    {
        let param_index = self.next_param_index();
        let condition = FilterCondition::Simple {
            column: column.to_string(),
            operator,
            param_index,
        };

        self.add_condition(condition);
        self
    }

    fn not(&mut self) -> &mut Self {
        self.negate_next = true;
        self
    }

    fn where_raw(&mut self, sql: &str) -> &mut Self {
        let condition = FilterCondition::Raw(sql.to_string());
        self.add_condition(condition);
        self
    }

    fn and_group(&mut self) -> &mut Self {
        self.group_stack.push((LogicalOperator::And, Vec::new()));
        self
    }

    fn or_group(&mut self) -> &mut Self {
        self.group_stack.push((LogicalOperator::Or, Vec::new()));
        self
    }

    fn end_group(&mut self) -> &mut Self {
        if let Some((operator, conditions)) = self.group_stack.pop() {
            if conditions.is_empty() {
                // Don't create empty groups
                return self;
            }

            let group_condition = FilterCondition::Group {
                operator,
                conditions,
            };

            if let Some((_, parent_conditions)) = self.group_stack.last_mut() {
                parent_conditions.push(group_condition);
            } else {
                self.conditions.push(group_condition);
            }
        }

        self
    }

    fn order_by(&mut self, column: &str, direction: SortDirection) -> &mut Self {
        self.order_by_clauses.push((column.to_string(), direction));
        self
    }

    fn order_by_multiple(&mut self, columns: &[(&str, SortDirection)]) -> &mut Self {
        for (column, direction) in columns {
            self.order_by_clauses.push((column.to_string(), *direction));
        }

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

    fn paginate(&mut self, strategy: PaginationStrategy) -> &mut Self {
        match strategy {
            PaginationStrategy::Offset { limit, offset } => {
                self.limit_value = Some(limit);
                self.offset_value = Some(offset);
            }
            PaginationStrategy::Cursor {
                limit,
                column,
                cursor,
                forward,
            } => {
                self.limit_value = Some(limit);

                // If we have a cursor value, add a condition based on cursor and direction
                if let Some(cursor_value) = cursor {
                    let operator = if forward {
                        ComparisonOperator::GreaterThan
                    } else {
                        ComparisonOperator::LessThan
                    };

                    // Add the cursor condition to the query
                    self.where_raw(&format!(
                        "{} {} '{}'",
                        column,
                        operator.as_sql(),
                        cursor_value
                    ));

                    // Set the order direction based on forward/backward pagination
                    let direction = if forward {
                        SortDirection::Ascending
                    } else {
                        SortDirection::Descending
                    };

                    // Order by the cursor column to ensure consistent results
                    self.order_by(&column, direction);
                } else {
                    // No cursor, just order by the column
                    let direction = if forward {
                        SortDirection::Ascending
                    } else {
                        SortDirection::Descending
                    };

                    self.order_by(&column, direction);
                }
            }
        }

        self
    }

    fn build_sql(&self) -> String {
        let columns = self.columns.join(", ");
        let mut query = format!("SELECT {} FROM {}", columns, self.table);

        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");

            // Build the WHERE clause
            let where_clause = self.build_where_clause(&self.conditions);
            query.push_str(&where_clause);
        }

        if !self.order_by_clauses.is_empty() {
            query.push_str(" ORDER BY ");

            let order_clauses: Vec<String> = self
                .order_by_clauses
                .iter()
                .map(|(column, direction)| format!("{} {}", column, direction.as_sql()))
                .collect();

            query.push_str(&order_clauses.join(", "));
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

impl<T> Query<T> {
    // Helper method to build WHERE clause from conditions
    fn build_where_clause(&self, conditions: &[FilterCondition]) -> String {
        let mut parts = Vec::new();

        for (i, condition) in conditions.iter().enumerate() {
            // For the first condition, no prefix
            // For subsequent conditions in an AND context, use "AND "
            let prefix = if i > 0 { "AND " } else { "" };

            match condition {
                FilterCondition::Simple {
                    column,
                    operator,
                    param_index,
                } => match operator {
                    ComparisonOperator::IsNull => {
                        parts.push(format!("{}{} IS NULL", prefix, column));
                    }
                    ComparisonOperator::IsNotNull => {
                        parts.push(format!("{}{} IS NOT NULL", prefix, column));
                    }
                    ComparisonOperator::In | ComparisonOperator::NotIn => {
                        parts.push(format!(
                            "{}{} {} (${{{}}}_array)",
                            prefix,
                            column,
                            operator.as_sql(),
                            param_index
                        ));
                    }
                    _ => {
                        parts.push(format!(
                            "{}{} {} ${}",
                            prefix,
                            column,
                            operator.as_sql(),
                            param_index
                        ));
                    }
                },
                FilterCondition::Group {
                    operator,
                    conditions,
                } => {
                    // Build the inner clause with appropriate logic
                    match operator {
                        LogicalOperator::Not => {
                            let inner_clause = self.build_where_clause(conditions);
                            parts.push(format!("{}NOT ({})", prefix, inner_clause));
                        }
                        LogicalOperator::And => {
                            let inner_clause = self.build_where_clause(conditions);
                            parts.push(format!("{}({})", prefix, inner_clause));
                        }
                        LogicalOperator::Or => {
                            // For OR groups, we need to join conditions with OR
                            let mut or_parts = Vec::new();

                            for (j, inner_condition) in conditions.iter().enumerate() {
                                match inner_condition {
                                    FilterCondition::Simple {
                                        column,
                                        operator,
                                        param_index,
                                    } => match operator {
                                        ComparisonOperator::IsNull => {
                                            or_parts.push(format!("{} IS NULL", column));
                                        }
                                        ComparisonOperator::IsNotNull => {
                                            or_parts.push(format!("{} IS NOT NULL", column));
                                        }
                                        ComparisonOperator::In | ComparisonOperator::NotIn => {
                                            or_parts.push(format!(
                                                "{} {} (${{{}}}_array)",
                                                column,
                                                operator.as_sql(),
                                                param_index
                                            ));
                                        }
                                        _ => {
                                            or_parts.push(format!(
                                                "{} {} ${}",
                                                column,
                                                operator.as_sql(),
                                                param_index
                                            ));
                                        }
                                    },
                                    FilterCondition::Group { .. } => {
                                        // For nested groups within OR, recursively build
                                        let inner_sql =
                                            self.build_where_clause(&[inner_condition.clone()]);
                                        // Remove any leading "AND " that might be present
                                        let clean_inner_sql =
                                            inner_sql.trim_start_matches("AND ").to_string();
                                        or_parts.push(clean_inner_sql);
                                    }
                                    FilterCondition::Raw(sql) => {
                                        or_parts.push(sql.clone());
                                    }
                                }
                            }

                            // Join all OR parts
                            let or_clause = or_parts.join(" OR ");
                            parts.push(format!("{}({})", prefix, or_clause));
                        }
                    }
                }
                FilterCondition::Raw(sql) => {
                    parts.push(format!("{}{}", prefix, sql));
                }
            }
        }

        parts.join(" ")
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
    fn test_query_build() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query
            .where_eq("name", "test")
            .and_eq("age", 18)
            .order_by("name", SortDirection::Ascending)
            .limit(10)
            .offset(20);

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table WHERE name = $1 AND age = $2 ORDER BY name ASC LIMIT 10 OFFSET 20",
            "SQL query with where, and, order by, limit, and offset should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_query_select_columns() -> TestResult<()> {
        let query = Query::<TestEntity>::new("test_table").select(&["id", "name", "age"]);

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT id, name, age FROM test_table",
            "SQL query with select columns should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_query_or_clause() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query.where_eq("name", "test1").or_eq("name", "test2");

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table WHERE name = $1 AND name = $2",
            "SQL query with OR clause should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_comparison_operators() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query
            .where_comp("age", ComparisonOperator::GreaterThan, 18)
            .and_comp("score", ComparisonOperator::LessThanOrEqual, 100);

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table WHERE age > $1 AND score <= $2",
            "SQL query with comparison operators should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_not_operator() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query.not().where_eq("active", true);

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table WHERE NOT (active = $1)",
            "SQL query with NOT operator should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_grouping() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query
            .where_eq("type", "user")
            .and_group()
            .where_eq("age", 18)
            .or_eq("age", 21)
            .end_group();

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table WHERE type = $1 AND (age = $2 AND age = $3)",
            "SQL query with grouping should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_multiple_order_by() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query.order_by_multiple(&[
            ("last_name", SortDirection::Ascending),
            ("first_name", SortDirection::Ascending),
        ]);

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table ORDER BY last_name ASC, first_name ASC",
            "SQL query with multiple order by clauses should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_offset_pagination() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query.paginate(PaginationStrategy::Offset {
            limit: 10,
            offset: 20,
        });

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table LIMIT 10 OFFSET 20",
            "SQL query with offset pagination should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_cursor_pagination_forward() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query.paginate(PaginationStrategy::Cursor {
            limit: 10,
            column: "id".to_string(),
            cursor: Some("100".to_string()),
            forward: true,
        });

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table WHERE id > '100' ORDER BY id ASC LIMIT 10",
            "SQL query with forward cursor pagination should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_cursor_pagination_backward() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query.paginate(PaginationStrategy::Cursor {
            limit: 10,
            column: "id".to_string(),
            cursor: Some("100".to_string()),
            forward: false,
        });

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table WHERE id < '100' ORDER BY id DESC LIMIT 10",
            "SQL query with backward cursor pagination should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_cursor_pagination_initial() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query.paginate(PaginationStrategy::Cursor {
            limit: 10,
            column: "id".to_string(),
            cursor: None,
            forward: true,
        });

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table ORDER BY id ASC LIMIT 10",
            "SQL query with initial cursor pagination should be constructed correctly",
        )?;

        Ok(())
    }

    #[test]
    fn test_complex_query_with_logical_operators() -> TestResult<()> {
        let mut query = Query::<TestEntity>::new("test_table");

        query
            .where_eq("status", "active")
            .and_group()
            .where_eq("age", 18)
            .or_eq("age", 21)
            .end_group()
            .and_group()
            .where_comp("score", ComparisonOperator::GreaterThan, 70)
            .or_comp("rank", ComparisonOperator::LessThan, 100)
            .end_group()
            .not()
            .where_eq("blocked", true);

        let sql = query.build_sql();
        assert_eq(
            sql,
            "SELECT * FROM test_table WHERE status = $1 AND (age = $2 OR age = $3) AND (score > $4 OR rank < $5) AND NOT (blocked = $6)",
            "Complex SQL query with logical operators should be constructed correctly",
        )?;

        Ok(())
    }
}
