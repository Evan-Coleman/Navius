# Example Entity Models

This directory contains example implementations of the entity pattern used in the Navius framework.

## File Naming Convention

Files in this directory use the `example_` prefix to indicate that they are reference implementations
that can be removed in production environments. A script can be created to remove all example code
by filtering for the `example_` prefix.

## Available Examples

- `example_user_entity.rs`: Example implementation of the `Entity` trait for a User domain object.

## Usage Guidelines

These examples demonstrate best practices for implementing domain entities in the Navius framework:

1. Implement the `Entity` trait from `core::models::Entity`
2. Use appropriate validation logic
3. Separate domain models from DTOs (Data Transfer Objects)
4. Implement helper methods for entity construction and modification

## Creating Your Own Entities

When creating your own entity implementations, follow these steps:

1. Create a new file for your entity (without the `example_` prefix)
2. Implement the `Entity` trait
3. Add validation logic
4. Add appropriate constructors and helper methods
5. Write comprehensive tests

## Testing

Each example includes unit tests that demonstrate how to properly test entity implementations,
including validation logic and helper methods. 