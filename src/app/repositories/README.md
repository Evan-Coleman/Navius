# Example Repositories

This directory contains example implementations of the repository pattern used in the Navius framework.

## File Naming Convention

Files in this directory use the `example_` prefix to indicate that they are reference implementations
that can be removed in production environments. A script can be created to remove all example code
by filtering for the `example_` prefix.

## Available Examples

- `example_user_repository.rs`: Example implementation of a custom repository for User entities that extends the base repository with additional query methods.

## Usage Guidelines

These examples demonstrate best practices for implementing repositories in the Navius framework:

1. Create a custom repository that wraps a core Repository
2. Add entity-specific query methods 
3. Use dependency injection for repository creation
4. Implement proper error handling
5. Delegate standard CRUD operations to the underlying repository

## Creating Your Own Repositories

When creating your own repository implementations, follow these steps:

1. Create a new file for your repository (without the `example_` prefix)
2. Create a struct that holds an inner repository
3. Implement the `Repository<E>` trait from core, delegating to the inner repository
4. Add custom query methods specific to your entity type
5. Write comprehensive tests

## Integration with Services

The repositories are designed to be consumed by services that handle business logic.
See the corresponding service examples in the `services` directory for how to use repositories
in service implementations. 