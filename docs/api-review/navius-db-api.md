# navius-db API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 95%  
**Status:** ✅ Good

## Dependencies

- navius-core
- async-trait
- serde
- serde_json
- tokio
- tracing
- uuid
- thiserror
- sqlx

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| Transaction<'a> | transaction.rs | 235 | ✅ Complete |
| ErrorContext | error.rs | 102 | ✅ Complete |
| DatabaseSpecificInfo | error.rs | 121 | ✅ Complete |
| DatabaseConfig | config.rs | 7 | ⚠️ Partial |
| Query<T> | query.rs | 215 | ✅ Complete |
| PoolOptions | pool.rs | 15 | ⚠️ Partial |
| PgPool | pool.rs | 77 | ⚠️ Partial |
| PgRow | pool.rs | 306 | ✅ Complete |
| PgTransaction | pool.rs | 383 | ✅ Complete |
| PgConnection | pool.rs | 516 | ⚠️ Partial |
| PgTransaction | pool.rs | 530 | ⚠️ Partial |
| DatabaseConnectionManager | pool.rs | 689 | ❌ Missing |
| BaseRepository<T: | repository.rs | 51 | ⚠️ Partial |
| SqlRepository<T: | repository.rs | 86 | ⚠️ Partial |
| DatabaseConnectionManager | connection.rs | 9 | ⚠️ Partial |
| DatabaseConnectionHandle | connection.rs | 68 | ⚠️ Partial |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| DatabaseError | error.rs | 7 | ⚠️ Partial |
| SortDirection | query.rs | 8 | ⚠️ Partial |
| ComparisonOperator | query.rs | 27 | ⚠️ Partial |
| LogicalOperator | query.rs | 76 | ⚠️ Partial |
| PaginationStrategy | query.rs | 98 | ⚠️ Partial |
| FilterCondition | query.rs | 121 | ✅ Complete |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| DatabaseProvider: | lib.rs | 29 | ⚠️ Partial |
| QueryBuilder | query.rs | 143 | ✅ Complete |
| QueryExecutor<T: | query.rs | 200 | ✅ Complete |
| DatabasePool: | pool.rs | 63 | ⚠️ Partial |
| DatabaseConnection: | pool.rs | 276 | ⚠️ Partial |
| DatabaseRowSet: | pool.rs | 298 | ⚠️ Partial |
| DatabaseTransaction: | pool.rs | 342 | ⚠️ Partial |
| Entity: | repository.rs | 12 | ⚠️ Partial |
| Repository<T: | repository.rs | 25 | ✅ Complete |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| fn | transaction.rs | 268 | ⚠️ Partial |
| fn | transaction.rs | 282 | ⚠️ Partial |
| fn | transaction.rs | 303 | ⚠️ Partial |
| fn | transaction.rs | 320 | ⚠️ Partial |
| fn | transaction.rs | 345 | ✅ Complete |
| fn | transaction.rs | 376 | ✅ Complete |
| fn | transaction.rs | 389 | ✅ Complete |
| fn | transaction.rs | 427 | ✅ Complete |
| fn | transaction.rs | 486 | ✅ Complete |
| fn | transaction.rs | 606 | ✅ Complete |
| fn | transaction.rs | 677 | ⚠️ Partial |
| fn | transaction.rs | 703 | ⚠️ Partial |
| nesting_level(&self) | transaction.rs | 719 | ⚠️ Partial |
| active_savepoints(&self) | transaction.rs | 724 | ⚠️ Partial |
| fn | transaction.rs | 753 | ✅ Complete |
| fn | transaction.rs | 913 | ✅ Complete |
| new() | error.rs | 219 | ⚠️ Partial |
| with_query(mut | error.rs | 230 | ⚠️ Partial |
| with_entity_type(mut | error.rs | 236 | ⚠️ Partial |
| with_operation(mut | error.rs | 242 | ⚠️ Partial |
| with_additional_info(mut | error.rs | 248 | ⚠️ Partial |
| with_db_specific(mut | error.rs | 254 | ⚠️ Partial |
| from_sqlx_error(error: | error.rs | 260 | ⚠️ Partial |
| with_context<C: | error.rs | 305 | ⚠️ Partial |
| with_query_context(self, | error.rs | 313 | ⚠️ Partial |
| with_detailed_context(self, | error.rs | 323 | ⚠️ Partial |
| detailed_db_error<S: | error.rs | 352 | ⚠️ Partial |
| chain_error<M: | error.rs | 367 | ⚠️ Partial |
| savepoint_error<S: | error.rs | 406 | ⚠️ Partial |
| error_code(&self) | error.rs | 411 | ⚠️ Partial |
| status_code(&self) | error.rs | 439 | ⚠️ Partial |
| unwrap_context(&self) | error.rs | 459 | ⚠️ Partial |
| error_chain(&self) | error.rs | 467 | ⚠️ Partial |
| root_cause(&self) | error.rs | 494 | ⚠️ Partial |
| db_specific_info(&self) | error.rs | 506 | ⚠️ Partial |
| connection_error<S: | error.rs | 550 | ⚠️ Partial |
| transaction_error<S: | error.rs | 555 | ⚠️ Partial |
| query_error<S: | error.rs | 560 | ⚠️ Partial |
| parameter_error<S: | error.rs | 565 | ⚠️ Partial |
| row_access_error<S: | error.rs | 570 | ⚠️ Partial |
| pool_error<S: | error.rs | 575 | ⚠️ Partial |
| configuration_error<S: | error.rs | 580 | ⚠️ Partial |
| migration_error<S: | error.rs | 585 | ⚠️ Partial |
| is_transient(&self) | error.rs | 590 | ⚠️ Partial |
| from_sqlx_with_context( | error.rs | 622 | ⚠️ Partial |
| unwrap_query_error(&self) | error.rs | 642 | ⚠️ Partial |
| new(url: | config.rs | 50 | ⚠️ Partial |
| validate(&self) | config.rs | 66 | ⚠️ Partial |
| connect_timeout(&self) | config.rs | 89 | ⚠️ Partial |
| idle_timeout(&self) | config.rs | 94 | ⚠️ Partial |
| max_lifetime(&self) | config.rs | 99 | ⚠️ Partial |
| as_sql(&self) | query.rs | 17 | ✅ Complete |
| as_sql(&self) | query.rs | 56 | ✅ Complete |
| as_sql(&self) | query.rs | 87 | ✅ Complete |
| new(table: | query.rs | 230 | ⚠️ Partial |
| select(mut | query.rs | 246 | ⚠️ Partial |
| fn | pool.rs | 85 | ⚠️ Partial |
| inner(&self) | pool.rs | 112 | ⚠️ Partial |
| new_mock(options: | pool.rs | 118 | ⚠️ Partial |
| new(row: | pool.rs | 313 | ⚠️ Partial |
| get<T>(&self, | pool.rs | 318 | ⚠️ Partial |
| get_by_index<T>(&self, | pool.rs | 329 | ⚠️ Partial |
| new(tx: | pool.rs | 537 | ⚠️ Partial |
| new(pool: | pool.rs | 694 | ❌ Missing |
| fn | pool.rs | 700 | ❌ Missing |
| fn | pool.rs | 731 | ❌ Missing |
| fn | pool.rs | 774 | ❌ Missing |
| new(db: | repository.rs | 60 | ✅ Complete |
| db(&self) | repository.rs | 68 | ⚠️ Partial |
| fn | repository.rs | 73 | ⚠️ Partial |
| new(db: | repository.rs | 93 | ⚠️ Partial |
| new<P: | connection.rs | 15 | ⚠️ Partial |
| fn | connection.rs | 23 | ⚠️ Partial |
| fn | connection.rs | 31 | ⚠️ Partial |
| fn | connection.rs | 61 | ⚠️ Partial |
| fn | connection.rs | 74 | ⚠️ Partial |
| fn | connection.rs | 80 | ⚠️ Partial |
| fn | connection.rs | 89 | ⚠️ Partial |
| fn | connection.rs | 99 | ⚠️ Partial |
| fn | connection.rs | 105 | ⚠️ Partial |

