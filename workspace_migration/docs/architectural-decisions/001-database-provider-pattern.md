# Architectural Decision Record: Database Provider Pattern

**Date**: March 29, 2025  
**Status**: Accepted  
**Deciders**: Navius Core Team  
**Context**: Database implementation strategy for the Navius workspace migration  

## Decision

We will implement a provider pattern for database access in the Navius framework by:

1. Creating a `navius-db` crate that contains:
   - Database interfaces and abstractions
   - Repository patterns
   - Query building interfaces
   - Transaction interfaces
   - Common database utilities

2. Creating separate implementation crates for specific database providers:
   - `navius-db-postgres`: PostgreSQL implementation using SQLx
   - Future implementations for MySQL, SQLite, and other databases as needed

## Problem Statement

The original Navius codebase had database functionality tightly coupled with PostgreSQL-specific code. This created several issues:

1. Applications were forced to include PostgreSQL dependencies even if they used a different database
2. Testing required PostgreSQL setup, complicating the test process
3. Supporting multiple database backends would require extensive refactoring
4. Code organization made it difficult to maintain clear boundaries

## Considered Alternatives

1. **Single Database Crate with Feature Flags**: Using a single `navius-db` crate with feature flags for different database backends.
   - Pros: Simpler organization, fewer crates to manage
   - Cons: Less clear separation of concerns, potential for "leaky abstractions", harder to test in isolation

2. **Database-Specific Crates Only**: Having only database-specific crates without a common interface crate.
   - Pros: Maximum flexibility for each database implementation
   - Cons: Inconsistent APIs, duplication across implementations, harder for application developers

3. **Adapter/Bridge Pattern**: Using a different type of abstraction pattern than the provider pattern.
   - Pros: Well-established pattern in some contexts
   - Cons: More complexity than needed for our use case

## Decision Rationale

The provider pattern was selected because it:

1. Creates a clean separation between the interface and implementations
2. Allows applications to depend only on the interfaces they need
3. Simplifies testing through mock implementations
4. Provides a consistent pattern that can be applied to other parts of the framework
5. Follows Rust's trait-based abstraction patterns
6. Balances flexibility with consistency across implementations

## Implementation Details

- The `navius-db` crate defines traits like `DatabaseProvider`, `Repository<T>`, `QueryBuilder`, and `Transaction`
- Each provider implementation crate implements these traits for a specific database
- Applications can select which provider to use at compile-time through dependencies
- A common pattern for feature-gated re-exports allows for simplified imports

## Consequences

**Positive:**
- Clear separation of concerns
- Support for multiple database backends
- Reduced dependencies for applications
- Improved testability
- Consistent pattern that can be applied elsewhere

**Negative:**
- More crates to maintain
- Potential for version synchronization issues between crates
- Additional abstraction layer may have some performance cost
- May require more complex documentation

## Adoption Strategy

1. First fully implement the `navius-db` interface crate
2. Implement the `navius-db-postgres` provider as a reference implementation
3. Update application code to use the new pattern
4. Document the pattern for future maintainers in the DATABASE_PROVIDER_GUIDE.md
5. Apply the same pattern to other parts of the framework where appropriate

## Related Decisions

- This pattern will be extended to other areas such as caching, template engines, and job processing
- We will maintain documentation of implementation patterns in a centralized guide

## References

- [DATABASE_PROVIDER_GUIDE.md](../../DATABASE_PROVIDER_GUIDE.md)
- [navius-db crate README](../../../crates/navius-db/README.md)
- [navius-db-postgres crate README](../../../crates/navius-db-postgres/README.md) 