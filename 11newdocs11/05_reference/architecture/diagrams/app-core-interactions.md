---
title: "App-Core Interactions"
description: "Documentation about App-Core Interactions"
category: architecture
tags:
  - api
  - architecture
  - database
last_updated: March 23, 2025
version: 1.0
---
# App-Core Interactions

This diagram illustrates how the user-facing `src/app` components interact with the framework's `src/core` components.

```mermaid
graph TD
    subgraph User_Code[User Code]
        app_api[app/api]
        app_services[app/services]
        app_router[app/router.rs]
    end
    
    subgraph Framework_Core[Framework Core]
        core_api[core/api]
        core_router[core/router]
        core_auth[core/auth]
        core_cache[core/cache]
        core_config[core/config]
        core_database[core/database]
        core_error[core/error]
        core_models[core/models]
        core_repository[core/repository]
        core_services[core/services]
        core_utils[core/utils]
    end
    
    %% API Layer Dependencies
    app_api --> core_router
    app_api --> core_utils
    app_api --> core_error
    app_api --> core_models
    app_api --> core_auth
    
    %% Service Layer Dependencies
    app_services --> core_repository
    app_services --> core_services
    app_services --> core_error
    app_services --> core_models
    app_services --> core_cache
    
    %% Router Dependencies
    app_router --> core_router
    app_router --> core_auth
    
    %% Core Component Relationships
    core_api --> core_services
    core_services --> core_repository
    core_repository --> core_database
    core_api --> core_error
    core_services --> core_error
    core_repository --> core_error
    
    classDef user fill:#d5e8d4,stroke:#82b366,stroke-width:1px;
    classDef core fill:#f8cecc,stroke:#b85450,stroke-width:1px;
    
    class app_api,app_services,app_router user;
    class core_api,core_router,core_auth,core_cache,core_config,core_database,core_error,core_models,core_repository,core_services,core_utils core;
```

## Extension Patterns

The diagram illustrates the key extension patterns used in the Navius framework:

### 1. API Extension Pattern

```mermaid
graph LR
    subgraph User_API[User API Implementation]
        user_handler[Custom Handler]
    end
    
    subgraph Core_Components[Core Components]
        router[core/router]
        error[core/error]
        models[core/models]
        utils[core/utils]
    end
    
    user_handler --> router
    user_handler --> error
    user_handler --> models
    user_handler --> utils
    
    classDef user fill:#d5e8d4,stroke:#82b366,stroke-width:1px;
    classDef core fill:#f8cecc,stroke:#b85450,stroke-width:1px;
    
    class user_handler user;
    class router,error,models,utils core;
```

### 2. Service Extension Pattern

```mermaid
graph LR
    subgraph User_Service[User Service Implementation]
        user_service[Custom Service]
    end
    
    subgraph Core_Components[Core Components]
        repository[core/repository]
        error[core/error]
        cache[core/cache]
    end
    
    user_service --> repository
    user_service --> error
    user_service --> cache
    
    classDef user fill:#d5e8d4,stroke:#82b366,stroke-width:1px;
    classDef core fill:#f8cecc,stroke:#b85450,stroke-width:1px;
    
    class user_service user;
    class repository,error,cache core;
```

### 3. Router Extension Pattern

```mermaid
graph LR
    subgraph User_Router[User Router Configuration]
        user_router[app/router.rs]
    end
    
    subgraph Core_Components[Core Components]
        core_router[core/router]
        app_state[AppState]
        middleware[Middleware Stack]
    end
    
    user_router --> core_router
    user_router --> app_state
    user_router --> middleware
    
    classDef user fill:#d5e8d4,stroke:#82b366,stroke-width:1px;
    classDef core fill:#f8cecc,stroke:#b85450,stroke-width:1px;
    
    class user_router user;
    class core_router,app_state,middleware core;
```

## Dependency Flow Guidelines

For maintainability and clean architecture, dependencies should flow according to these guidelines:

1. **User code depends on core code**, not the other way around
2. **Core code should never depend on user code**
3. **App components should depend only on their corresponding core components**
4. **Cross-cutting concerns (error, config) may be used by any component**

Following these guidelines ensures:

- **Separation of concerns** between framework and application code
- **Testability** of user code in isolation
- **Upgradability** of the framework without breaking user code
- **Extensibility** through well-defined interfaces 

## Related Documents
- [Project Structure](../project-structure.md) - Overall structure
- [Module Dependencies](../module-dependencies.md) - Dependencies between modules

