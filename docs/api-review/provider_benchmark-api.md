# provider_benchmark API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- navius-db
- navius-core
- async-trait
- sqlx
- thiserror
- tracing
- tokio
- futures
- serde
- serde_json
- chrono
- crypto-hash
- regex

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| PgTransaction | transaction.rs | 16 | ⚠️ Partial |
| PgTransactionManager | transaction.rs | 164 | ⚠️ Partial |
| PgQuery | query.rs | 6 | ⚠️ Partial |
| PostgresProvider | provider.rs | 21 | ⚠️ Partial |
| PostgresProviderOptions | provider.rs | 30 | ✅ Complete |
| PgPoolConfig | pool.rs | 10 | ⚠️ Partial |
| MigrationVersion | version.rs | 50 | ⚠️ Partial |
| VersionManager | version.rs | 68 | ✅ Complete |
| Migration | runner.rs | 50 | ⚠️ Partial |
| MigrationOptions | runner.rs | 130 | ⚠️ Partial |
| MigrationRunner | runner.rs | 152 | ⚠️ Partial |
| MigrationStatus | runner.rs | 479 | ✅ Complete |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| PgError | error.rs | 6 | ⚠️ Partial |
| MigrationVersionError | version.rs | 10 | ⚠️ Partial |
| MigrationError | runner.rs | 16 | ⚠️ Partial |
| MigrationState | runner.rs | 456 | ⚠️ Partial |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new(tx: | transaction.rs | 29 | ✅ Complete |
| inner_mut(&mut | transaction.rs | 39 | ⚠️ Partial |
| is_committed(&self) | transaction.rs | 44 | ⚠️ Partial |
| is_rolled_back(&self) | transaction.rs | 49 | ⚠️ Partial |
| nesting_level(&self) | transaction.rs | 54 | ⚠️ Partial |
| fn | transaction.rs | 59 | ⚠️ Partial |
| fn | transaction.rs | 69 | ⚠️ Partial |
| fn | transaction.rs | 79 | ⚠️ Partial |
| new(pool: | transaction.rs | 171 | ✅ Complete |
| fn | provider.rs | 121 | ⚠️ Partial |
| fn | provider.rs | 156 | ⚠️ Partial |
| fn | provider.rs | 175 | ⚠️ Partial |
| fn | provider.rs | 198 | ⚠️ Partial |
| pool(&self) | provider.rs | 215 | ⚠️ Partial |
| transaction_manager(&self) | provider.rs | 220 | ⚠️ Partial |
| new(pool: | version.rs | 75 | ⚠️ Partial |
| fn | version.rs | 83 | ⚠️ Partial |
| fn | version.rs | 103 | ⚠️ Partial |
| fn | version.rs | 119 | ⚠️ Partial |
| fn | version.rs | 152 | ⚠️ Partial |
| fn | version.rs | 171 | ⚠️ Partial |
| fn | version.rs | 198 | ⚠️ Partial |
| fn | version.rs | 226 | ⚠️ Partial |
| fn | version.rs | 253 | ⚠️ Partial |
| from_file(path: | runner.rs | 69 | ✅ Complete |
| new(pool: | runner.rs | 160 | ⚠️ Partial |
| fn | runner.rs | 171 | ⚠️ Partial |
| fn | runner.rs | 177 | ⚠️ Partial |
| fn | runner.rs | 215 | ⚠️ Partial |
| fn | runner.rs | 312 | ⚠️ Partial |
| fn | runner.rs | 335 | ⚠️ Partial |
| fn | runner.rs | 396 | ⚠️ Partial |

