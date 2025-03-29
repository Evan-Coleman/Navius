---
title: "Pet Database Api"
description: ""
category: "Documentation"
tags: []
last_updated: "March 28, 2025"
version: "1.0"
---

---
title: "Pet Database API Reference"
description: "Reference documentation for the Pet Database API, including CRUD operations and architecture details"
category: reference
tags:
  - api
  - database
  - pets
  - repository
related:
  - database-api.md
  - ../patterns/repository-pattern.md
  - ../../02_examples/database-service-example.md
last_updated: March 31, 2025
version: 1.0
---

# Pet Database API

## Overview

The Pet Database API provides a complete set of CRUD (Create, Read, Update, Delete) operations for managing pet records in the database. This API is built following clean architecture principles, with proper separation between database abstractions in the core layer and pet-specific implementations in the application layer.

## Endpoints

### Get All Pets

Retrieves a list of all pets in the database.

**URL**: `/petdb`

**Method**: `GET`

**Authentication**: Public

**Response**: 

```json
[
  {
    "id": "uuid-string",
    "name": "Pet Name",
    "species": "Pet Species",
    "age": 5,
    "created_at": "2024-06-01T12:00:00.000Z",
    "updated_at": "2024-06-01T12:00:00.000Z"
  },
  ...
]
```

**Status Codes**:
- `200 OK`: Successfully retrieved the list of pets
- `500 Internal Server Error`: Server encountered an error

### Get Pet by ID

Retrieves a specific pet by its unique identifier.

**URL**: `/petdb/:id`

**Method**: `GET`

**URL Parameters**:
- `id`: UUID of the pet to retrieve

**Authentication**: Public

**Response**: 

```json
{
  "id": "uuid-string",
  "name": "Pet Name",
  "species": "Pet Species",
  "age": 5,
  "created_at": "2024-06-01T12:00:00.000Z",
  "updated_at": "2024-06-01T12:00:00.000Z"
}
```

**Status Codes**:
- `200 OK`: Successfully retrieved the pet
- `400 Bad Request`: Invalid UUID format
- `404 Not Found`: Pet with the given ID was not found
- `500 Internal Server Error`: Server encountered an error

### Create Pet

Creates a new pet in the database.

**URL**: `/petdb`

**Method**: `POST`

**Authentication**: Required

**Request Body**:

```json
{
  "name": "Pet Name",
  "species": "Pet Species",
  "age": 5
}
```

**Validation Rules**:
- `name`: Required, cannot be empty, maximum 50 characters
- `species`: Required, cannot be empty
- `age`: Required, must be non-negative, must be realistic (0-100)

**Response**: 

```json
{
  "id": "uuid-string",
  "name": "Pet Name",
  "species": "Pet Species",
  "age": 5,
  "created_at": "2024-06-01T12:00:00.000Z",
  "updated_at": "2024-06-01T12:00:00.000Z"
}
```

**Status Codes**:
- `201 Created`: Successfully created the pet
- `400 Bad Request`: Validation error in the request data
- `401 Unauthorized`: Authentication required
- `500 Internal Server Error`: Server encountered an error

### Update Pet

Updates an existing pet in the database.

**URL**: `/petdb/:id`

**Method**: `PUT`

**URL Parameters**:
- `id`: UUID of the pet to update

**Authentication**: Required

**Request Body**:

```json
{
  "name": "Updated Name",      // Optional
  "species": "Updated Species", // Optional
  "age": 6                     // Optional
}
```

**Validation Rules**:
- `name` (if provided): Cannot be empty, maximum 50 characters
- `species` (if provided): Cannot be empty
- `age` (if provided): Must be non-negative, must be realistic (0-100)

**Response**: 

```json
{
  "id": "uuid-string",
  "name": "Updated Name",
  "species": "Updated Species",
  "age": 6,
  "created_at": "2024-06-01T12:00:00.000Z",
  "updated_at": "2024-06-01T13:00:00.000Z"
}
```

**Status Codes**:
- `200 OK`: Successfully updated the pet
- `400 Bad Request`: Invalid UUID format or validation error
- `401 Unauthorized`: Authentication required
- `404 Not Found`: Pet with the given ID was not found
- `500 Internal Server Error`: Server encountered an error

### Delete Pet

Deletes a pet from the database.

**URL**: `/petdb/:id`

**Method**: `DELETE`

**URL Parameters**:
- `id`: UUID of the pet to delete

**Authentication**: Required

**Response**: No content

**Status Codes**:
- `204 No Content`: Successfully deleted the pet
- `400 Bad Request`: Invalid UUID format
- `401 Unauthorized`: Authentication required
- `404 Not Found`: Pet with the given ID was not found
- `500 Internal Server Error`: Server encountered an error

## Architecture

The Pet API follows a clean architecture approach with the following layers:

### Core Layer

- **EntityRepository**: Generic repository interface in `core/database/repository.rs` that defines standard CRUD operations
- **Database Utilities**: Common database functions in `core/database/utils.rs` for transaction management and error handling

### Application Layer

- **PetRepository**: Implementation of the `EntityRepository` for Pet entities in `app/database/repositories/pet_repository.rs`
- **PetService**: Business logic and validation in `app/services/pet_service.rs`
- **API Endpoints**: HTTP handlers in `app/api/pet_db.rs` that expose the functionality via REST API

This separation allows for clear responsibilities:
- **Core Layer**: Generic interfaces and abstractions
- **App Layer**: Pet-specific implementations

## Error Handling

The API follows a consistent error handling approach:

- **400 Bad Request**: Input validation errors (invalid data, format issues)
- **401 Unauthorized**: Authentication issues
- **404 Not Found**: Resource not found
- **500 Internal Server Error**: Database or server-side errors

All error responses include:
- HTTP status code
- Error message
- Error type

Example error response:

```json
{
  "code": 400,
  "message": "Pet name cannot be empty",
  "error_type": "validation_error"
}
```

## Testing

The API includes comprehensive tests:

- **Unit Tests**: For core database abstractions and PetService business logic
- **API Endpoint Tests**: Testing the HTTP layer and response handling
- **Validation Tests**: Ensuring proper input validation and error handling

## Database Schema

Pet records are stored in the `pets` table with the following schema:

| Column     | Type          | Description                         |
|------------|---------------|-------------------------------------|
| id         | UUID          | Primary key                         |
| name       | VARCHAR(50)   | Pet's name                          |
| species    | VARCHAR(50)   | Type of animal                      |
| age        | INTEGER       | Pet's age in years                  |
| created_at | TIMESTAMP     | When the record was created         |
| updated_at | TIMESTAMP     | When the record was last updated    | 
