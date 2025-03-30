use navius_db::error::{DatabaseError, DatabaseResult};
use navius_db::query::{Filter, Order, Query, QueryExecutor, SortField};
use sqlx::postgres::PgArguments;
use sqlx::{Arguments, Executor, Postgres};
use std::fmt;
use std::sync::Arc;
use tracing::{debug, instrument};

/// PostgreSQL query implementation
pub struct PgQuery {
    /// Table name
    table: String,
    /// Query filters
    filters: Vec<Filter>,
    /// Sort fields
    sort: Vec<SortField>,
    /// Pagination offset
    offset: Option<u64>,
    /// Pagination limit
    limit: Option<u64>,
    /// Selected columns
    columns: Vec<String>,
}

impl PgQuery {
    /// Create a new query for the given table
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            filters: Vec::new(),
            sort: Vec::new(),
            offset: None,
            limit: None,
            columns: Vec::new(),
        }
    }

    /// Build a SQL WHERE clause from the filters
    fn build_where_clause(
        &self,
        args: &mut PgArguments,
        param_offset: &mut u32,
    ) -> DatabaseResult<String> {
        if self.filters.is_empty() {
            return Ok(String::new());
        }

        let mut where_clause = String::from("WHERE ");
        let mut conditions = Vec::new();

        for filter in &self.filters {
            let condition = self.build_filter(filter, args, param_offset)?;
            conditions.push(condition);
        }

        where_clause.push_str(&conditions.join(" AND "));
        Ok(where_clause)
    }

    /// Build a SQL filter condition from a filter
    fn build_filter(
        &self,
        filter: &Filter,
        args: &mut PgArguments,
        param_offset: &mut u32,
    ) -> DatabaseResult<String> {
        match filter {
            Filter::Eq(field, value) => {
                let placeholder = format!("${}", *param_offset);
                *param_offset += 1;
                args.add(value);
                Ok(format!("{} = {}", field, placeholder))
            }
            Filter::Ne(field, value) => {
                let placeholder = format!("${}", *param_offset);
                *param_offset += 1;
                args.add(value);
                Ok(format!("{} <> {}", field, placeholder))
            }
            Filter::Gt(field, value) => {
                let placeholder = format!("${}", *param_offset);
                *param_offset += 1;
                args.add(value);
                Ok(format!("{} > {}", field, placeholder))
            }
            Filter::Lt(field, value) => {
                let placeholder = format!("${}", *param_offset);
                *param_offset += 1;
                args.add(value);
                Ok(format!("{} < {}", field, placeholder))
            }
            Filter::Gte(field, value) => {
                let placeholder = format!("${}", *param_offset);
                *param_offset += 1;
                args.add(value);
                Ok(format!("{} >= {}", field, placeholder))
            }
            Filter::Lte(field, value) => {
                let placeholder = format!("${}", *param_offset);
                *param_offset += 1;
                args.add(value);
                Ok(format!("{} <= {}", field, placeholder))
            }
            Filter::In(field, values) => {
                let placeholders: Vec<String> = values
                    .iter()
                    .map(|_| {
                        let placeholder = format!("${}", *param_offset);
                        *param_offset += 1;
                        placeholder
                    })
                    .collect();

                for value in values {
                    args.add(value);
                }

                Ok(format!("{} IN ({})", field, placeholders.join(", ")))
            }
            Filter::Like(field, pattern) => {
                let placeholder = format!("${}", *param_offset);
                *param_offset += 1;
                args.add(pattern);
                Ok(format!("{} LIKE {}", field, placeholder))
            }
            Filter::Between(field, start, end) => {
                let placeholder1 = format!("${}", *param_offset);
                *param_offset += 1;
                let placeholder2 = format!("${}", *param_offset);
                *param_offset += 1;
                args.add(start);
                args.add(end);
                Ok(format!(
                    "{} BETWEEN {} AND {}",
                    field, placeholder1, placeholder2
                ))
            }
            Filter::IsNull(field) => Ok(format!("{} IS NULL", field)),
            Filter::IsNotNull(field) => Ok(format!("{} IS NOT NULL", field)),
            Filter::And(filters) => {
                let mut conditions = Vec::new();
                for filter in filters {
                    let condition = self.build_filter(filter, args, param_offset)?;
                    conditions.push(condition);
                }
                Ok(format!("({})", conditions.join(" AND ")))
            }
            Filter::Or(filters) => {
                let mut conditions = Vec::new();
                for filter in filters {
                    let condition = self.build_filter(filter, args, param_offset)?;
                    conditions.push(condition);
                }
                Ok(format!("({})", conditions.join(" OR ")))
            }
            Filter::Not(filter) => {
                let condition = self.build_filter(filter, args, param_offset)?;
                Ok(format!("NOT ({})", condition))
            }
            Filter::Raw(raw_sql) => Ok(raw_sql.clone()),
        }
    }

    /// Build a SQL ORDER BY clause from the sort fields
    fn build_order_by_clause(&self) -> String {
        if self.sort.is_empty() {
            return String::new();
        }

        let clauses: Vec<String> = self
            .sort
            .iter()
            .map(|sort_field| {
                let direction = match sort_field.order {
                    Order::Asc => "ASC",
                    Order::Desc => "DESC",
                };
                format!("{} {}", sort_field.field, direction)
            })
            .collect();

        format!("ORDER BY {}", clauses.join(", "))
    }

    /// Build a SQL LIMIT clause
    fn build_limit_clause(&self) -> String {
        if let Some(limit) = self.limit {
            format!("LIMIT {}", limit)
        } else {
            String::new()
        }
    }

    /// Build a SQL OFFSET clause
    fn build_offset_clause(&self) -> String {
        if let Some(offset) = self.offset {
            format!("OFFSET {}", offset)
        } else {
            String::new()
        }
    }
}

impl Query for PgQuery {
    fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    fn sort(mut self, field: impl Into<String>, order: Order) -> Self {
        self.sort.push(SortField {
            field: field.into(),
            order,
        });
        self
    }

    fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    fn select(mut self, columns: Vec<String>) -> Self {
        self.columns = columns;
        self
    }
}

impl fmt::Debug for PgQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgQuery")
            .field("table", &self.table)
            .field("filters", &self.filters)
            .field("sort", &self.sort)
            .field("offset", &self.offset)
            .field("limit", &self.limit)
            .field("columns", &self.columns)
            .finish()
    }
}

/// PostgreSQL query executor
pub struct PgQueryExecutor<'a, E> {
    executor: E,
    _lifetime: std::marker::PhantomData<&'a ()>,
}

impl<'a, E> PgQueryExecutor<'a, E>
where
    E: Executor<'a, Database = Postgres> + Send + Sync,
{
    /// Create a new query executor with the given SQLx executor
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            _lifetime: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<'a, E> QueryExecutor for PgQueryExecutor<'a, E>
where
    E: Executor<'a, Database = Postgres> + Send + Sync,
{
    /// Execute a query and return all rows
    #[instrument(skip(self, query), level = "debug")]
    async fn execute_query<T>(&mut self, query: PgQuery) -> DatabaseResult<Vec<T>>
    where
        T: for<'row> sqlx::FromRow<'row, sqlx::postgres::PgRow> + Send + Unpin + 'static,
    {
        // Build the columns part
        let columns_part = if query.columns.is_empty() {
            "*".to_string()
        } else {
            query.columns.join(", ")
        };

        // Create arguments collection
        let mut args = PgArguments::default();
        let mut param_offset = 1;

        // Build the WHERE clause
        let where_part = query.build_where_clause(&mut args, &mut param_offset)?;

        // Build the ORDER BY clause
        let order_part = query.build_order_by_clause();

        // Build the LIMIT clause
        let limit_part = query.build_limit_clause();

        // Build the OFFSET clause
        let offset_part = query.build_offset_clause();

        // Build the complete SQL query
        let sql = format!(
            "SELECT {} FROM {} {} {} {} {}",
            columns_part, query.table, where_part, order_part, limit_part, offset_part
        );

        debug!("Executing SQL: {}", sql);

        // Execute the query
        let rows = sqlx::query_as_with::<_, T, _>(&sql, args)
            .fetch_all(&mut self.executor)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Failed to execute query: {}", e)))?;

        Ok(rows)
    }

    /// Execute a count query and return the number of rows
    #[instrument(skip(self, query), level = "debug")]
    async fn count(&mut self, query: PgQuery) -> DatabaseResult<i64> {
        // Create arguments collection
        let mut args = PgArguments::default();
        let mut param_offset = 1;

        // Build the WHERE clause
        let where_part = query.build_where_clause(&mut args, &mut param_offset)?;

        // Build the complete SQL query
        let sql = format!(
            "SELECT COUNT(*) as count FROM {} {}",
            query.table, where_part
        );

        debug!("Executing count SQL: {}", sql);

        // Execute the query
        let row = sqlx::query_with(&sql, args)
            .fetch_one(&mut self.executor)
            .await
            .map_err(|e| {
                DatabaseError::QueryError(format!("Failed to execute count query: {}", e))
            })?;

        let count: i64 = row.try_get("count").map_err(|e| {
            DatabaseError::DataError(format!("Failed to retrieve count from result: {}", e))
        })?;

        Ok(count)
    }

    /// Execute a query and return a single row
    #[instrument(skip(self, query), level = "debug")]
    async fn execute_query_one<T>(&mut self, query: PgQuery) -> DatabaseResult<Option<T>>
    where
        T: for<'row> sqlx::FromRow<'row, sqlx::postgres::PgRow> + Send + Unpin + 'static,
    {
        // Build the columns part
        let columns_part = if query.columns.is_empty() {
            "*".to_string()
        } else {
            query.columns.join(", ")
        };

        // Create arguments collection
        let mut args = PgArguments::default();
        let mut param_offset = 1;

        // Build the WHERE clause
        let where_part = query.build_where_clause(&mut args, &mut param_offset)?;

        // Build the ORDER BY clause
        let order_part = query.build_order_by_clause();

        // Build the LIMIT clause
        let limit_part = "LIMIT 1";

        // Build the complete SQL query
        let sql = format!(
            "SELECT {} FROM {} {} {} {}",
            columns_part, query.table, where_part, order_part, limit_part
        );

        debug!("Executing SQL (one): {}", sql);

        // Execute the query
        let row = sqlx::query_as_with::<_, T, _>(&sql, args)
            .fetch_optional(&mut self.executor)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Failed to execute query: {}", e)))?;

        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navius_db::query::Filter;
    use sqlx::postgres::PgArguments;

    #[test]
    fn test_query_builder_basics() {
        let query = PgQuery::new("users")
            .filter(Filter::Eq("name".to_string(), "Alice".into()))
            .sort("created_at".to_string(), Order::Desc)
            .limit(10)
            .offset(20);

        assert_eq!(query.table, "users");
        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.sort.len(), 1);
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(20));
    }

    #[test]
    fn test_build_where_clause() {
        let query = PgQuery::new("users")
            .filter(Filter::Eq("name".to_string(), "Alice".into()))
            .filter(Filter::Gt("age".to_string(), 18.into()));

        let mut args = PgArguments::default();
        let mut param_offset = 1;
        let where_clause = query
            .build_where_clause(&mut args, &mut param_offset)
            .unwrap();

        assert_eq!(where_clause, "WHERE name = $1 AND age > $2");
        assert_eq!(param_offset, 3); // Incremented twice
    }

    #[test]
    fn test_build_order_by_clause() {
        let query = PgQuery::new("users")
            .sort("name".to_string(), Order::Asc)
            .sort("created_at".to_string(), Order::Desc);

        let order_clause = query.build_order_by_clause();
        assert_eq!(order_clause, "ORDER BY name ASC, created_at DESC");
    }

    #[test]
    fn test_build_limit_clause() {
        let query = PgQuery::new("users").limit(10);
        assert_eq!(query.build_limit_clause(), "LIMIT 10");

        let query_no_limit = PgQuery::new("users");
        assert_eq!(query_no_limit.build_limit_clause(), "");
    }

    #[test]
    fn test_build_offset_clause() {
        let query = PgQuery::new("users").offset(20);
        assert_eq!(query.build_offset_clause(), "OFFSET 20");

        let query_no_offset = PgQuery::new("users");
        assert_eq!(query_no_offset.build_offset_clause(), "");
    }

    #[test]
    fn test_complex_filter() {
        let query = PgQuery::new("users").filter(Filter::Or(vec![
            Filter::Eq("name".to_string(), "Alice".into()),
            Filter::And(vec![
                Filter::Gt("age".to_string(), 18.into()),
                Filter::Lt("age".to_string(), 65.into()),
            ]),
        ]));

        let mut args = PgArguments::default();
        let mut param_offset = 1;
        let where_clause = query
            .build_where_clause(&mut args, &mut param_offset)
            .unwrap();

        assert_eq!(where_clause, "WHERE (name = $1 OR (age > $2 AND age < $3))");
        assert_eq!(param_offset, 4); // Incremented three times
    }
}
