---
title: "GraphQL API Example"
description: "Building GraphQL APIs with Navius"
category: examples
tags:
  - examples
  - graphql
  - api
related:
  - examples/rest-api-example.md
  - examples/error-handling-example.md
last_updated: March 26, 2025
version: 1.0
---

# GraphQL API Example

This example demonstrates how to implement a GraphQL API using Navius, including schema definition, resolvers, queries, mutations, and error handling.

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

```toml
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

```rust
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

```rust
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

```rust
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

```rust
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

```rust
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

```rust
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

```rust
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

```rust
pub mod types;
pub mod query;
pub mod mutation;
pub mod schema;

pub use schema::{BookSchema, create_schema};
```

### src/main.rs

```rust
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

```graphql
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

```graphql
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

```graphql
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

## Testing the GraphQL API

### Running the Example

```bash
cargo run
```

The server will start with the following endpoints:
- GraphQL API: http://127.0.0.1:8080/graphql
- GraphiQL Interface: http://127.0.0.1:8080/graphiql
- GraphQL Playground: http://127.0.0.1:8080/playground

### Sample Queries

#### Get All Books

```graphql
query {
  books {
    id
    title
    author
    genre
    rating
  }
}
```

#### Get a Book by ID

```graphql
query {
  book(id: "BOOK_ID_HERE") {
    id
    title
    author
    description
    genre
    rating
    createdAt
    updatedAt
  }
}
```

#### Books by Author

```graphql
query {
  booksByAuthor(author: "Martin") {
    id
    title
    author
    genre
  }
}
```

### Sample Mutations

#### Create a Book

```graphql
mutation {
  createBook(input: {
    title: "GraphQL in Action"
    author: "Samer Buna"
    description: "A practical guide to GraphQL APIs"
    genre: "Programming"
    rating: 4.5
  }) {
    id
    title
    author
    genre
    rating
    createdAt
  }
}
```

#### Update a Book

```graphql
mutation {
  updateBook(
    id: "BOOK_ID_HERE"
    input: {
      rating: 4.7
      description: "Updated description"
    }
  ) {
    id
    title
    description
    rating
    updatedAt
  }
}
```

#### Delete a Book

```graphql
mutation {
  deleteBook(id: "BOOK_ID_HERE")
}
```

## Key Concepts Demonstrated

1. **GraphQL Schema Definition**: Using `async-graphql` to define the schema in a type-safe way.

2. **GraphQL Types and Resolvers**: Mapping between Rust types and GraphQL types.

3. **GraphQL Queries and Mutations**: Implementing query and mutation resolvers.

4. **GraphQL Error Handling**: Converting application errors to GraphQL errors with appropriate extensions.

5. **Repository Pattern**: Separating data access from the GraphQL resolvers.

6. **GraphiQL and Playground Integration**: Providing interactive interfaces for exploring the API.

## Best Practices

1. **Type Safety**: Using Rust's type system to ensure GraphQL schema correctness.

2. **Error Handling**: Consistent error handling with meaningful messages and codes.

3. **Data Layer Separation**: Repository pattern to abstract data access.

4. **Input Validation**: Separate input types for create and update operations.

5. **Context Injection**: Passing repositories through the GraphQL context.

6. **Schema Organization**: Splitting queries and mutations into separate modules.

## Next Steps

- Add authentication and authorization
- Implement GraphQL subscriptions for real-time updates
- Add dataloader for efficient resolving of related data
- Implement filtering, sorting, and pagination
- Connect to a real database

## Related Documentation

- [REST API Example](./rest-api-example.md)
- [Error Handling Example](./error-handling-example.md) 