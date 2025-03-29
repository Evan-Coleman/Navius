---
title: "Building GraphQL APIs with Navius"
description: "Comprehensive guide to implementing GraphQL APIs using Navius, including schema definition, resolvers, queries, mutations, and error handling"
category: examples
tags:
  - graphql
  - api
  - async-graphql
  - queries
  - mutations
  - resolvers
  - schema
related:
  - 02_examples/rest-api-example.md
  - 02_examples/dependency-injection-example.md
  - 04_guides/api-design.md
last_updated: March 27, 2025
version: 1.1
status: stable
---

# GraphQL API Example

This example demonstrates how to implement a GraphQL API using Navius, including schema definition, resolvers, queries, mutations, and error handling.

## Overview

GraphQL provides a powerful alternative to REST APIs, allowing clients to request exactly the data they need. Navius makes it easy to integrate GraphQL into your application using the async-graphql crate, providing a type-safe and elegant way to build GraphQL servers in Rust.

This example builds a complete GraphQL API for a book management system, showing how to:
- Define GraphQL schemas and types
- Implement queries and mutations
- Handle errors gracefully
- Organize your code for maintainability
- Test GraphQL endpoints

## Quick Navigation

- [Project Structure](#project-structure)
- [Implementation](#implementation)
  - [Cargo.toml](#cargotoml)
  - [Domain Models](#src/models/bookrs)
  - [Repositories](#src/repositories/book_repositoryrs)
  - [GraphQL Schema](#src/graphql/schemars)
  - [GraphQL Types](#src/graphql/typesrs)
  - [Queries](#src/graphql/queryrs)
  - [Mutations](#src/graphql/mutationrs)
  - [Error Handling](#src/errorrs)
  - [Application Entry Point](#src/mainrs)
- [Running the Example](#running-the-example)
- [Testing the API](#testing-the-api)
- [Advanced Topics](#advanced-topics)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)

## Prerequisites

Before working with this example, you should be familiar with:

- Rust programming basics
- Async programming with Tokio
- GraphQL fundamentals (queries, mutations, schemas)
- Navius framework basics

Required dependencies:
- Rust 1.70 or newer
- Navius 0.1.0 or newer
- async-graphql 5.0 or newer

## Project Structure

```
graphql-example/
├── Cargo.toml
├── config/
│   └── default.yaml
└── src/
    ├── main.rs                   # Application entry point
    ├── graphql/                  # GraphQL implementation
    │   ├── mod.rs
    │   ├── schema.rs             # GraphQL schema
    │   ├── query.rs              # Query resolvers
    │   ├── mutation.rs           # Mutation resolvers
    │   └── types.rs              # GraphQL types
    ├── models/                   # Domain models
    │   ├── mod.rs
    │   └── book.rs               # Book model
    ├── repositories/             # Data access
    │   ├── mod.rs
    │   └── book_repository.rs    # Book repository
    └── error.rs                  # Error handling
```

## Implementation

### Cargo.toml

```
[package]
name = "graphql-example"
version = "0.1.0"
edition = "2021"

[dependencies]
navius = "0.1.0"
tokio = { version = "1.28", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-graphql = "5.0"
async-graphql-axum = "5.0"
uuid = { version = "1.3", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
thiserror = "1.0"
log = "0.4"
```

### src/models/book.rs

```
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub genre: String,
    pub published_date: Option<DateTime<Utc>>,
    pub rating: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BookInput {
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub genre: String,
    pub published_date: Option<DateTime<Utc>>,
    pub rating: Option<f32>,
}

impl From<BookInput> for Book {
    fn from(input: BookInput) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: input.title,
            author: input.author,
            description: input.description,
            genre: input.genre,
            published_date: input.published_date,
            rating: input.rating,
            created_at: now,
            updated_at: now,
        }
    }
}
```

### src/repositories/book_repository.rs

```
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::book::Book;
use crate::error::AppError;

#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Book>, AppError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Book, AppError>;
    async fn find_by_author(&self, author: &str) -> Result<Vec<Book>, AppError>;
    async fn find_by_genre(&self, genre: &str) -> Result<Vec<Book>, AppError>;
    async fn create(&self, book: Book) -> Result<Book, AppError>;
    async fn update(&self, id: &Uuid, book: Book) -> Result<Book, AppError>;
    async fn delete(&self, id: &Uuid) -> Result<(), AppError>;
}

// In-memory implementation for the example
pub struct InMemoryBookRepository {
    books: RwLock<HashMap<Uuid, Book>>,
}

impl InMemoryBookRepository {
    pub fn new() -> Self {
        let mut books = HashMap::new();
        
        // Add some sample books
        let sample_books = vec![
            Book::from(crate::models::book::BookInput {
                title: "The Rust Programming Language".to_string(),
                author: "Steve Klabnik and Carol Nichols".to_string(),
                description: Some("The official book on the Rust programming language".to_string()),
                genre: "Programming".to_string(),
                published_date: None,
                rating: Some(4.8),
            }),
            Book::from(crate::models::book::BookInput {
                title: "Designing Data-Intensive Applications".to_string(),
                author: "Martin Kleppmann".to_string(),
                description: Some("The big ideas behind reliable, scalable, and maintainable systems".to_string()),
                genre: "Software Architecture".to_string(),
                published_date: None,
                rating: Some(4.9),
            }),
        ];
        
        for book in sample_books {
            books.insert(book.id, book);
        }
        
        Self {
            books: RwLock::new(books),
        }
    }
}

#[async_trait]
impl BookRepository for InMemoryBookRepository {
    async fn find_all(&self) -> Result<Vec<Book>, AppError> {
        let books = self.books.read().await;
        Ok(books.values().cloned().collect())
    }
    
    async fn find_by_id(&self, id: &Uuid) -> Result<Book, AppError> {
        let books = self.books.read().await;
        
        books.get(id)
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("Book with ID {} not found", id)))
    }
    
    async fn find_by_author(&self, author: &str) -> Result<Vec<Book>, AppError> {
        let books = self.books.read().await;
        
        let mut result = Vec::new();
        for book in books.values() {
            if book.author.to_lowercase().contains(&author.to_lowercase()) {
                result.push(book.clone());
            }
        }
        
        Ok(result)
    }
    
    async fn find_by_genre(&self, genre: &str) -> Result<Vec<Book>, AppError> {
        let books = self.books.read().await;
        
        let mut result = Vec::new();
        for book in books.values() {
            if book.genre.to_lowercase().contains(&genre.to_lowercase()) {
                result.push(book.clone());
            }
        }
        
        Ok(result)
    }
    
    async fn create(&self, book: Book) -> Result<Book, AppError> {
        let mut books = self.books.write().await;
        
        let book_clone = book.clone();
        books.insert(book.id, book);
        
        Ok(book_clone)
    }
    
    async fn update(&self, id: &Uuid, book: Book) -> Result<Book, AppError> {
        let mut books = self.books.write().await;
        
        if !books.contains_key(id) {
            return Err(AppError::not_found(format!("Book with ID {} not found", id)));
        }
        
        let book_clone = book.clone();
        books.insert(*id, book);
        
        Ok(book_clone)
    }
    
    async fn delete(&self, id: &Uuid) -> Result<(), AppError> {
        let mut books = self.books.write().await;
        
        if books.remove(id).is_none() {
            return Err(AppError::not_found(format!("Book with ID {} not found", id)));
        }
        
        Ok(())
    }
}
```

### src/error.rs

```
use async_graphql::ErrorExtensions;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }
    
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }
    
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::InternalServerError(message.into())
    }
}

// Extension to convert AppError to GraphQL error
impl ErrorExtensions for AppError {
    fn extend(&self) -> async_graphql::Error {
        let mut err = async_graphql::Error::new(self.to_string());
        
        match self {
            AppError::BadRequest(_) => {
                err = err.extend_with(|_, e| {
                    e.set("code", "BAD_REQUEST");
                    e.set("status", 400);
                });
            }
            AppError::NotFound(_) => {
                err = err.extend_with(|_, e| {
                    e.set("code", "NOT_FOUND");
                    e.set("status", 404);
                });
            }
            AppError::InternalServerError(_) => {
                err = err.extend_with(|_, e| {
                    e.set("code", "INTERNAL_SERVER_ERROR");
                    e.set("status", 500);
                });
            }
        }
        
        err
    }
}
```

### src/graphql/types.rs

```
use async_graphql::{Object, InputObject, SimpleObject, ID, Context};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::book::{Book, BookInput};
use crate::repositories::book_repository::BookRepository;
use crate::error::AppError;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct BookType {
    pub id: ID,
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub genre: String,
    pub published_date: Option<DateTime<Utc>>,
    pub rating: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Book> for BookType {
    fn from(book: Book) -> Self {
        Self {
            id: ID(book.id.to_string()),
            title: book.title,
            author: book.author,
            description: book.description,
            genre: book.genre,
            published_date: book.published_date,
            rating: book.rating,
            created_at: book.created_at,
            updated_at: book.updated_at,
        }
    }
}

#[derive(InputObject)]
pub struct BookInputType {
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub genre: String,
    pub published_date: Option<DateTime<Utc>>,
    pub rating: Option<f32>,
}

impl From<BookInputType> for BookInput {
    fn from(input: BookInputType) -> Self {
        Self {
            title: input.title,
            author: input.author,
            description: input.description,
            genre: input.genre,
            published_date: input.published_date,
            rating: input.rating,
        }
    }
}

#[derive(InputObject)]
pub struct BookUpdateInputType {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub published_date: Option<DateTime<Utc>>,
    pub rating: Option<f32>,
}
```

### src/graphql/query.rs

```
use async_graphql::{Context, Object, Result, ID};
use std::sync::Arc;
use uuid::Uuid;

use crate::graphql::types::BookType;
use crate::repositories::book_repository::BookRepository;
use crate::error::AppError;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn books(&self, ctx: &Context<'_>) -> Result<Vec<BookType>> {
        let repo = ctx.data::<Arc<dyn BookRepository>>().unwrap();
        
        let books = repo.find_all().await
            .map_err(|e| e.extend())?;
        
        Ok(books.into_iter().map(BookType::from).collect())
    }
    
    async fn book(&self, ctx: &Context<'_>, id: ID) -> Result<BookType> {
        let repo = ctx.data::<Arc<dyn BookRepository>>().unwrap();
        
        let uuid = Uuid::parse_str(&id.to_string())
            .map_err(|_| AppError::bad_request("Invalid UUID format").extend())?;
        
        let book = repo.find_by_id(&uuid).await
            .map_err(|e| e.extend())?;
        
        Ok(BookType::from(book))
    }
    
    async fn books_by_author(&self, ctx: &Context<'_>, author: String) -> Result<Vec<BookType>> {
        let repo = ctx.data::<Arc<dyn BookRepository>>().unwrap();
        
        let books = repo.find_by_author(&author).await
            .map_err(|e| e.extend())?;
        
        Ok(books.into_iter().map(BookType::from).collect())
    }
    
    async fn books_by_genre(&self, ctx: &Context<'_>, genre: String) -> Result<Vec<BookType>> {
        let repo = ctx.data::<Arc<dyn BookRepository>>().unwrap();
        
        let books = repo.find_by_genre(&genre).await
            .map_err(|e| e.extend())?;
        
        Ok(books.into_iter().map(BookType::from).collect())
    }
}
```

### src/graphql/mutation.rs

```
use async_graphql::{Context, Object, Result, ID};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::graphql::types::{BookType, BookInputType, BookUpdateInputType};
use crate::repositories::book_repository::BookRepository;
use crate::models::book::Book;
use crate::error::AppError;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_book(&self, ctx: &Context<'_>, input: BookInputType) -> Result<BookType> {
        let repo = ctx.data::<Arc<dyn BookRepository>>().unwrap();
        
        let book_input = input.into();
        let book = Book::from(book_input);
        
        let created_book = repo.create(book).await
            .map_err(|e| e.extend())?;
        
        Ok(BookType::from(created_book))
    }
    
    async fn update_book(&self, ctx: &Context<'_>, id: ID, input: BookUpdateInputType) -> Result<BookType> {
        let repo = ctx.data::<Arc<dyn BookRepository>>().unwrap();
        
        let uuid = Uuid::parse_str(&id.to_string())
            .map_err(|_| AppError::bad_request("Invalid UUID format").extend())?;
        
        // Get existing book
        let mut book = repo.find_by_id(&uuid).await
            .map_err(|e| e.extend())?;
        
        // Update fields if provided
        if let Some(title) = input.title {
            book.title = title;
        }
        
        if let Some(author) = input.author {
            book.author = author;
        }
        
        if let Some(description) = input.description {
            book.description = Some(description);
        }
        
        if let Some(genre) = input.genre {
            book.genre = genre;
        }
        
        if let Some(published_date) = input.published_date {
            book.published_date = Some(published_date);
        }
        
        if let Some(rating) = input.rating {
            book.rating = Some(rating);
        }
        
        // Update timestamp
        book.updated_at = Utc::now();
        
        // Save updated book
        let updated_book = repo.update(&uuid, book).await
            .map_err(|e| e.extend())?;
        
        Ok(BookType::from(updated_book))
    }
    
    async fn delete_book(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let repo = ctx.data::<Arc<dyn BookRepository>>().unwrap();
        
        let uuid = Uuid::parse_str(&id.to_string())
            .map_err(|_| AppError::bad_request("Invalid UUID format").extend())?;
        
        repo.delete(&uuid).await
            .map_err(|e| e.extend())?;
        
        Ok(true)
    }
}
```

### src/graphql/schema.rs

```
use async_graphql::{Schema, EmptySubscription};
use std::sync::Arc;

use crate::graphql::query::QueryRoot;
use crate::graphql::mutation::MutationRoot;
use crate::repositories::book_repository::BookRepository;

pub type BookSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(
    book_repository: Arc<dyn BookRepository>,
) -> BookSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(book_repository)
        .finish()
}
```

### src/graphql/mod.rs

```
pub mod types;
pub mod query;
pub mod mutation;
pub mod schema;

pub use schema::{BookSchema, create_schema};
```

### src/main.rs

```
mod error;
mod models;
mod repositories;
mod graphql;

use std::sync::Arc;
use navius::http::header;
use navius::routing::{get, post, Router};
use navius::http::StatusCode;
use navius::web::{handler, Response};
use async_graphql::http::{GraphiQLSource, playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

use repositories::book_repository::InMemoryBookRepository;
use graphql::{BookSchema, create_schema};

// GraphQL handler
async fn graphql_handler(
    schema: navius::Extension<BookSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// GraphiQL playground handler
async fn graphiql_handler() -> impl Response {
    navius::http::Response::builder()
        .header(header::CONTENT_TYPE, "text/html")
        .body(GraphiQLSource::build().endpoint("/graphql").finish())
        .unwrap()
}

// GraphQL playground handler
async fn playground_handler() -> impl Response {
    navius::http::Response::builder()
        .header(header::CONTENT_TYPE, "text/html")
        .body(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create repository
    let book_repository = Arc::new(InMemoryBookRepository::new());
    
    // Create GraphQL schema
    let schema = create_schema(book_repository);
    
    // Create router
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", get(graphiql_handler))
        .route("/playground", get(playground_handler))
        .layer(navius::Extension(schema));
    
    // Run the server
    let addr = "127.0.0.1:8080";
    println!("GraphQL server running at http://{}/graphql", addr);
    println!("GraphiQL interface available at http://{}/graphiql", addr);
    println!("GraphQL Playground available at http://{}/playground", addr);
    
    navius::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
```

## GraphQL Schema

The example implements a GraphQL schema for managing books with the following operations:

### Queries

```
type Query {
  # Get all books
  books: [BookType!]!
  
  # Get a book by ID
  book(id: ID!): BookType!
  
  # Get books by author name (partial match)
  booksByAuthor(author: String!): [BookType!]!
  
  # Get books by genre (partial match)
  booksByGenre(genre: String!): [BookType!]!
}
```

### Mutations

```
type Mutation {
  # Create a new book
  createBook(input: BookInputType!): BookType!
  
  # Update an existing book
  updateBook(id: ID!, input: BookUpdateInputType!): BookType!
  
  # Delete a book
  deleteBook(id: ID!): Boolean!
}
```

### Types

```
type BookType {
  id: ID!
  title: String!
  author: String!
  description: String
  genre: String!
  publishedDate: DateTime
  rating: Float
  createdAt: DateTime!
  updatedAt: DateTime!
}

input BookInputType {
  title: String!
  author: String!
  description: String
  genre: String!
  publishedDate: DateTime
  rating: Float
}

input BookUpdateInputType {
  title: String
  author: String
  description: String
  genre: String
  publishedDate: DateTime
  rating: Float
}
```

## Running the Example

After setting up the project with all the components shown above, you can run the application using:

```
cargo run
```

By default, the GraphQL server will be accessible at `http://localhost:8080/graphql`. You can also access the GraphQL Playground at `http://localhost:8080/graphql/playground` for an interactive interface to test your API.

### Configuration

The application loads configuration from `config/default.yaml`. You can customize settings like the server port:

```
server:
  port: 8080
  host: "127.0.0.1"

graphql:
  path: "/graphql"
  playground: true
  introspection: true
```

## Testing the API

### Using GraphQL Playground

1. Start the application with `cargo run`
2. Navigate to `http://localhost:8080/graphql/playground` in your browser
3. Try running some queries and mutations:

**Query all books:**

```
query GetAllBooks {
  books {
    id
    title
    author
    genre
    rating
  }
}
```

**Query a specific book:**

```
query GetBookById {
  book(id: "paste-a-valid-uuid-here") {
    id
    title
    author
    description
    genre
    rating
    publishedDate
    createdAt
    updatedAt
  }
}
```

**Create a new book:**

```
mutation CreateBook {
  createBook(input: {
    title: "GraphQL in Action"
    author: "Samer Buna"
    genre: "Programming"
    description: "A practical guide to GraphQL APIs"
    rating: 4.7
  }) {
    id
    title
    author
    genre
    rating
  }
}
```

**Update a book:**

```
mutation UpdateBook {
  updateBook(
    id: "paste-a-valid-uuid-here"
    input: {
      title: "Updated Title"
      author: "Same Author"
      genre: "Programming"
      description: "Updated description"
      rating: 4.9
    }
  ) {
    id
    title
    description
    rating
  }
}
```

**Delete a book:**

```
mutation DeleteBook {
  deleteBook(id: "paste-a-valid-uuid-here")
}
```

### Automated Testing

You can also write automated tests for your GraphQL API. Here's an example of a test for the book queries:

```
#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::Request;
    use navius::core::test::TestApp;

    #[tokio::test]
    async fn test_get_all_books() {
        // Create a test app with the book repository
        let app = TestApp::new()
            .with_service(|registry| {
                let repo = Arc::new(InMemoryBookRepository::new());
                registry.register::<dyn BookRepository>(repo);
            })
            .build();

        // Prepare the GraphQL query
        let query = r#"
            query {
                books {
                    id
                    title
                    author
                }
            }
        "#;

        // Execute the query
        let request = Request::new(query);
        let response = app.execute_graphql(request).await;

        // Assert that there are no errors
        assert!(response.errors.is_empty());
        
        // Check the response data
        let data = response.data.into_json().unwrap();
        let books = data["books"].as_array().unwrap();
        
        // Should have our sample books
        assert_eq!(books.len(), 2);
    }
}
```

## Advanced Topics

### Adding Authentication

To add authentication to your GraphQL API, you can:

1. Create a custom context with the authenticated user:

```
pub struct GraphQLContext {
    pub user_id: Option<Uuid>,
    pub book_repository: Arc<dyn BookRepository>,
}

impl From<&RequestContext> for GraphQLContext {
    fn from(ctx: &RequestContext) -> Self {
        // Extract user ID from the request context (e.g., from JWT)
        let user_id = ctx.get_user_id();
        
        // Resolve the repository from the service registry
        let book_repository = ctx.service_registry().resolve::<dyn BookRepository>().unwrap();
        
        Self {
            user_id,
            book_repository,
        }
    }
}
```

2. Update your resolvers to check for authentication:

```
#[Object]
impl QueryRoot {
    async fn books(&self, ctx: &Context<'_>) -> Result<Vec<Book>, Error> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check if user is authenticated
        if gql_ctx.user_id.is_none() {
            return Err("Unauthorized".into());
        }
        
        // Proceed with the authorized query
        let books = gql_ctx.book_repository.find_all().await?;
        Ok(books)
    }
}
```

### Implementing Pagination

For larger datasets, implement pagination with cursor-based or offset-based approaches:

```
#[derive(InputObject)]
struct PaginationInput {
    first: Option<i32>,
    after: Option<String>,
}

#[derive(SimpleObject)]
struct BookConnection {
    edges: Vec<BookEdge>,
    page_info: PageInfo,
}

#[derive(SimpleObject)]
struct BookEdge {
    node: Book,
    cursor: String,
}

#[derive(SimpleObject)]
struct PageInfo {
    has_next_page: bool,
    end_cursor: Option<String>,
}

#[Object]
impl QueryRoot {
    async fn books(&self, ctx: &Context<'_>, pagination: Option<PaginationInput>) -> Result<BookConnection, Error> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let pagination = pagination.unwrap_or_default();
        
        // Fetch one more than requested to determine if there are more items
        let limit = pagination.first.unwrap_or(10) + 1;
        let after_id = pagination.after.and_then(|cursor| decode_cursor(&cursor).ok());
        
        let books = gql_ctx.book_repository.find_with_pagination(after_id, limit).await?;
        
        // Check if there are more items
        let has_next_page = books.len() > pagination.first.unwrap_or(10);
        
        // Remove the extra item if we fetched more than requested
        let edges = books.into_iter()
            .take(pagination.first.unwrap_or(10))
            .map(|book| BookEdge {
                cursor: encode_cursor(&book.id),
                node: book,
            })
            .collect();
            
        let end_cursor = if edges.is_empty() {
            None
        } else {
            Some(edges.last().unwrap().cursor.clone())
        };
        
        Ok(BookConnection {
            edges,
            page_info: PageInfo {
                has_next_page,
                end_cursor,
            },
        })
    }
}
```

### Real-time Updates with Subscriptions

GraphQL subscriptions allow for real-time updates. Here's how to implement them:

```
#[derive(Default)]
pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn book_updates(&self, ctx: &Context<'_>) -> impl Stream<Item = Book> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Get the event broker from the context
        let event_broker = gql_ctx.event_broker.clone();
        
        // Subscribe to book updates
        event_broker.subscribe::<BookUpdatedEvent>().map(|event| event.book)
    }
}

// In your schema.rs
pub type BookSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

// In main.rs
let schema = BookSchema::build(QueryRoot, MutationRoot, SubscriptionRoot)
    .data(event_broker.clone())
    .finish();
```

## Troubleshooting

### Common Errors and Solutions

#### "Field does not exist on type"

**Problem**: The GraphQL server returns an error saying a requested field doesn't exist on the type.

**Solution**: 
1. Check if the field is properly defined in your GraphQL type
2. Ensure field names match exactly (GraphQL is case-sensitive)
3. Verify that the field is exposed in the `#[Object]` implementation

#### Resolver Function Errors

**Problem**: Your resolver function returns an error or doesn't compile.

**Solution**:
1. Check the function signature matches what `async-graphql` expects
2. Ensure the return type implements the appropriate traits
3. For `Result` types, make sure the error type can be converted to `Error`

#### Context Access Issues

**Problem**: Can't access data from the context in a resolver.

**Solution**:
1. Make sure you've added the data to the schema with `.data(...)` 
2. Use `ctx.data::<YourType>()` to extract it
3. Handle the potential error if the data isn't found

### Debugging Tips

1. **Enable detailed error messages**:

```
let schema = BookSchema::build(QueryRoot, MutationRoot, SubscriptionRoot)
    .extension(async_graphql::extensions::Logger)
    .finish();
```

2. **Use introspection** in GraphQL Playground to examine your schema

3. **Log resolver executions** to track the flow of requests

4. **Check the GraphQL query** for syntax errors in variables or arguments

## Best Practices

### Schema Design

1. **Follow GraphQL naming conventions**:
   - Types: PascalCase (e.g., `Book`, `User`)
   - Fields: camelCase (e.g., `title`, `authorName`)
   - Enums: PascalCase for type, uppercase for values (e.g., `Genre.FICTION`)

2. **Design types around business domains**, not database structures

3. **Use meaningful field names** that describe what the field represents

4. **Provide descriptions** for types and fields using doc comments:

```
/// Represents a book in the library
#[derive(SimpleObject)]
pub struct Book {
    /// Unique identifier for the book
    pub id: Uuid,
    // Other fields...
}
```

### Performance Optimization

1. **Implement DataLoader for N+1 query problems**:

```
pub struct BookLoader {
    repository: Arc<dyn BookRepository>,
}

#[async_trait::async_trait]
impl Loader<Uuid> for BookLoader {
    type Value = Book;
    type Error = Error;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let books = self.repository.find_by_ids(keys).await?;
        Ok(books.into_iter().map(|book| (book.id, book)).collect())
    }
}

// In your query resolver:
async fn author_books(&self, ctx: &Context<'_>, author_id: Uuid) -> Result<Vec<Book>, Error> {
    let book_loader = ctx.data_unchecked::<DataLoader<BookLoader>>();
    let book_ids = get_book_ids_for_author(author_id).await?;
    
    // Load all books in a single batch
    let books = book_loader.load_many(book_ids).await?;
    Ok(books)
}
```

2. **Use field complexity analysis** to prevent DoS attacks:

```
let schema = BookSchema::build(QueryRoot, MutationRoot, SubscriptionRoot)
    .limit_complexity(50)
    .finish();
```

3. **Implement timeouts** for long-running queries:

```
let schema = BookSchema::build(QueryRoot, MutationRoot, SubscriptionRoot)
    .limit_depth(10)
    .limit_complexity(50)
    .extension(async_graphql::extensions::Timeout::new(Duration::from_secs(5)))
    .finish();
```

### Error Handling

1. **Return meaningful error messages** that help clients understand what went wrong

2. **Don't expose sensitive information** in error messages

3. **Use custom error types** for different error categories:

```
#[derive(SimpleObject)]
struct ValidationError {
    field: String,
    message: String,
}

#[derive(SimpleObject)]
struct GraphQLError {
    message: String,
    code: String,
    validation_errors: Option<Vec<ValidationError>>,
}

// In your resolver:
if input.title.is_empty() {
    return Err(Error::new("Title cannot be empty")
        .extend_with(|_, e| {
            let mut validation_errors = Vec::new();
            validation_errors.push(ValidationError {
                field: "title".to_string(),
                message: "Title cannot be empty".to_string(),
            });
            
            e.set("code", "VALIDATION_ERROR");
            e.set("validation_errors", validation_errors);
        }));
}
```

## Conclusion

This example has demonstrated how to build a complete GraphQL API using Navius and async-graphql. You've learned:

- How to define GraphQL schemas, types, queries, and mutations
- How to implement repositories for data access
- How to handle errors in a GraphQL context
- Advanced topics like authentication, pagination, and subscriptions
- Troubleshooting tips and best practices

For more examples and in-depth information, refer to:
- [Navius API Documentation](../05_reference/01_api/graphql.md)
- [async-graphql Crate Documentation](https://docs.rs/async-graphql)
- [GraphQL Official Specification](https://spec.graphql.org/) 