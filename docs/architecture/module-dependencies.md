# Module Dependencies Diagram

**Updated At:** March 22, 2025

This document provides visualizations of the module dependencies in the Navius project to help developers understand the architecture.

## Core Components and Their Dependencies

```mermaid
graph TD
    subgraph Core
        Router[Router]
        API[API Handlers]
        Services[Services]
        Repository[Repository]
        Database[Database]
        Auth[Authentication]
        Cache[Cache]
        Config[Configuration]
        Error[Error Handling]
        Metrics[Metrics]
        Reliability[Reliability]
        Utils[Utilities]
    end
    
    subgraph External
        External[External APIs]
        Redis[Redis]
        Postgres[PostgreSQL]
        MSAuth[MS Entra]
    end
    
    subgraph App
        AppRouter[App Router]
        AppAPI[App API]
        AppServices[App Services]
    end
    
    %% Core Dependencies
    Router --> API
    Router --> Auth
    API --> Services
    API --> Error
    Services --> Repository
    Services --> Cache
    Services --> Error
    Repository --> Database
    Repository --> Error
    Database --> Error
    Auth --> Error
    Auth --> Config
    Cache --> Error
    Cache --> Config
    Reliability --> Error
    Reliability --> Config
    Metrics --> Error
    
    %% External Dependencies
    Database --> Postgres
    Cache --> Redis
    Auth --> MSAuth
    External --> API
    
    %% App Dependencies
    AppRouter --> Router
    AppAPI --> API
    AppServices --> Services
    AppRouter --> AppAPI
    AppAPI --> AppServices
    
    %% Legend
    classDef core fill:#f9f,stroke:#333,stroke-width:2px;
    classDef external fill:#bbf,stroke:#333,stroke-width:2px;
    classDef app fill:#bfb,stroke:#333,stroke-width:2px;
    
    class Router,API,Services,Repository,Database,Auth,Cache,Config,Error,Metrics,Reliability,Utils core;
    class External,Redis,Postgres,MSAuth external;
    class AppRouter,AppAPI,AppServices app;
```

## Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant Router as Router
    participant Auth as Authentication
    participant API as API Handler
    participant Service as Service
    participant Repository as Repository
    participant Database as Database
    participant Cache as Cache
    
    Client->>Router: HTTP Request
    Router->>Auth: Authenticate
    Auth-->>Router: Authentication Result
    
    alt Authentication Failed
        Router-->>Client: 401 Unauthorized
    else Authentication Succeeded
        Router->>API: Route to Handler
        
        API->>Service: Call Service Method
        
        Service->>Cache: Check Cache
        alt Cache Hit
            Cache-->>Service: Cached Data
        else Cache Miss
            Service->>Repository: Query Data
            Repository->>Database: Execute Query
            Database-->>Repository: Query Result
            Repository-->>Service: Data
            Service->>Cache: Update Cache
        end
        
        Service-->>API: Result
        API-->>Router: Response
        Router-->>Client: HTTP Response
    end
```

## Core Module Structure

```mermaid
graph TD
    subgraph src
        subgraph core
            api[api]
            auth[auth]
            cache[cache]
            config[config]
            database[database]
            error[error]
            metrics[metrics]
            reliability[reliability]
            repository[repository]
            router[router]
            services[services]
            utils[utils]
        end
        
        subgraph app
            app_router[router.rs]
            app_api[api/]
            app_services[services/]
        end
        
        lib[lib.rs]
        main[main.rs]
        generated[generated_apis.rs]
    end
    
    %% Structure relationships
    lib --> core
    lib --> app
    main --> lib
    main --> app_router
    app_router --> app_api
    app_api --> app_services
    generated --> core
    
    %% Core relationships
    router --> api
    api --> services
    services --> repository
    repository --> database
    
    %% Legend
    classDef coreModule fill:#f9f,stroke:#333,stroke-width:1px;
    classDef appModule fill:#bfb,stroke:#333,stroke-width:1px;
    classDef entryPoint fill:#fbb,stroke:#333,stroke-width:1px;
    
    class api,auth,cache,config,database,error,metrics,reliability,repository,router,services,utils coreModule;
    class app_router,app_api,app_services appModule;
    class lib,main,generated entryPoint;
```

## Clean Architecture View

```mermaid
graph TD
    subgraph "Frameworks & Drivers"
        Web[Web Framework]
        DB[Database Drivers]
        External[External Libraries]
    end
    
    subgraph "Interface Adapters"
        Controllers[Controllers/API]
        Gateways[Repository Implementations]
        Presenters[Response Formatters]
    end
    
    subgraph "Application Business Rules"
        Services[Services]
        DTOs[Data Transfer Objects]
    end
    
    subgraph "Enterprise Business Rules"
        Entities[Domain Entities]
        Rules[Business Rules]
    end
    
    %% Dependencies
    Web --> Controllers
    Controllers --> Services
    Services --> Entities
    Services --> Rules
    Services --> Gateways
    Gateways --> DB
    Gateways --> Entities
    Presenters --> DTOs
    Controllers --> Presenters
    
    %% Navius Mapping Notes
    Controllers -. "src/core/api/" .-> Services
    Gateways -. "src/core/repository/" .-> Entities
    Services -. "src/core/services/" .-> Entities
    Web -. "Axum Framework" .-> Controllers
    DB -. "SQLx" .-> Gateways
    
    %% Legend
    classDef outerLayer fill:#bbf,stroke:#333,stroke-width:1px;
    classDef interfaceLayer fill:#fbf,stroke:#333,stroke-width:1px;
    classDef appLayer fill:#bfb,stroke:#333,stroke-width:1px;
    classDef coreLayer fill:#fbb,stroke:#333,stroke-width:1px;
    
    class Web,DB,External outerLayer;
    class Controllers,Gateways,Presenters interfaceLayer;
    class Services,DTOs appLayer;
    class Entities,Rules coreLayer;
```

## How to View These Diagrams

These diagrams are written in Mermaid format, which can be viewed in several ways:

1. **GitHub Rendering**: If viewing on GitHub, the diagrams will render automatically

2. **VS Code Extension**: Install the "Markdown Preview Mermaid Support" extension in VS Code

3. **Mermaid Live Editor**: Copy the diagram code and paste it into the [Mermaid Live Editor](https://mermaid.live)

4. **Browser Extension**: Use a browser extension like "Mermaid Diagrams" for Chrome 