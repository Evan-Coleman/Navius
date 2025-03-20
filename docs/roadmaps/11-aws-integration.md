# AWS Integration Roadmap

## Overview
A pragmatic approach to AWS integration for our Rust Axum backend, focusing on essential AWS services needed for a secure, reliable production deployment with proper integration of Postgres, Redis, and Microsoft Entra authentication.

## Current State
Our application needs AWS integration focused on production readiness while maintaining our security-first approach and minimizing complexity.

## Target State
A focused AWS integration featuring:
- Secure AWS deployment with proper IAM configurations
- Axum-optimized container deployment to ECS/Fargate
- Postgres RDS integration with security best practices
- Redis ElastiCache integration for performance
- Observability with CloudWatch
- Automated deployment pipeline with GitLab CI/CD
- Secrets management for secure configuration

## Implementation Progress Tracking

### Phase 1: Core AWS Security & Configuration
1. **IAM & Security Setup**
   - [ ] Create IAM roles with least-privilege permissions
   - [ ] Implement secure credential management
   - [ ] Configure VPC with private subnets and security groups
   - [ ] Set up secure network ACLs and routing
   
   *Updated at: Not started*

2. **Entra Authentication Integration**
   - [ ] Configure Microsoft Entra as identity provider for application
   - [ ] Implement JWT validation with AWS-compatible settings
   - [ ] Set up proper CORS for Entra authentication flow
   - [ ] Create secure token handling with appropriate caching
   
   *Updated at: Not started*

3. **Secrets & Configuration Management**
   - [ ] Implement AWS Secrets Manager for sensitive credentials
   - [ ] Create environment-specific configuration pipeline
   - [ ] Set up secure parameter retrieval at runtime
   - [ ] Implement configuration hot reloading (when appropriate)
   
   *Updated at: Not started*

### Phase 2: AWS Service Integration
1. **RDS Postgres Integration**
   - [ ] Configure RDS PostgreSQL with encryption and backups
   - [ ] Implement secure connection pooling with IAM authentication
   - [ ] Create high-availability configuration
   - [ ] Set up monitoring and alerting for database health
   
   *Updated at: Not started*

2. **ElastiCache Redis Integration**
   - [ ] Configure Redis cluster with encryption and VPC security
   - [ ] Implement fault-tolerant Redis client with connection pooling
   - [ ] Set up Redis key management and expiration policies
   - [ ] Create monitoring and alerting for Redis performance
   
   *Updated at: Not started*

3. **S3 Storage Integration**
   - [ ] Configure S3 buckets with proper access controls
   - [ ] Implement secure file upload/download with presigned URLs
   - [ ] Set up encryption for data at rest
   - [ ] Create appropriate bucket lifecycle policies
   
   *Updated at: Not started*

### Phase 3: Deployment & Observability
1. **ECS/Fargate Deployment**
   - [ ] Create optimized Docker container for Rust Axum application
   - [ ] Implement ECS task definitions with security configurations
   - [ ] Set up auto-scaling with appropriate metrics
   - [ ] Configure Application Load Balancer with WAF protection
   
   *Updated at: Not started*

2. **CloudWatch Observability**
   - [ ] Implement structured logging compatible with CloudWatch
   - [ ] Create custom metrics for application health
   - [ ] Set up appropriate alarms and dashboards
   - [ ] Implement distributed tracing with X-Ray
   
   *Updated at: Not started*

3. **GitLab CI/CD Pipeline**
   - [ ] Create build pipeline optimized for Rust compilation
   - [ ] Implement security scanning for dependencies and code
   - [ ] Set up automated testing stage
   - [ ] Create deployment pipeline with proper environment promotion
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 20, 2024
- **Next Milestone**: IAM & Security Setup

## Success Criteria
- Application can be securely deployed to AWS with GitLab CI/CD
- RDS PostgreSQL and ElastiCache Redis are properly integrated
- Microsoft Entra authentication works seamlessly
- All connections between services use proper security measures
- Observability provides actionable insights and alerts
- Configuration and secrets are managed securely
- Deployment process is automated and reliable

## Implementation Notes

### Example Implementation: AWS-Ready Axum Application

```rust
use axum::{
    routing::get,
    Router, 
    extract::State,
    middleware,
};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_secretsmanager as secretsmanager;
use aws_sdk_rds as rds;
use aws_sdk_elasticache as elasticache;
use aws_sdk_s3 as s3;
use aws_sdk_cloudwatch as cloudwatch;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Deserialize;

// Application configuration with AWS service integration
#[derive(Clone)]
pub struct AppConfig {
    pub region: String,
    pub postgres_config: PostgresConfig,
    pub redis_config: RedisConfig,
    pub s3_config: S3Config,
    pub entra_config: EntraConfig,
}

#[derive(Clone, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub use_iam_auth: bool,
    pub connection_pool_size: u32,
}

#[derive(Clone, Deserialize)]
pub struct RedisConfig {
    pub primary_endpoint: String,
    pub reader_endpoint: Option<String>,
    pub port: u16,
    pub use_tls: bool,
}

#[derive(Clone, Deserialize)]
pub struct S3Config {
    pub bucket_name: String,
    pub upload_expiry_seconds: u64,
}

#[derive(Clone, Deserialize)]
pub struct EntraConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub issuer: String,
    pub audience: String,
}

// AWS service clients wrapped for use with Axum
#[derive(Clone)]
pub struct AwsClients {
    pub secrets_manager: Arc<secretsmanager::Client>,
    pub s3: Arc<s3::Client>,
    pub cloudwatch: Arc<cloudwatch::Client>,
}

// Initialize AWS configuration
async fn initialize_aws(region: Option<String>) -> AwsClients {
    // Set up region from env or use default
    let region_provider = RegionProviderChain::first_try(region.map(aws_sdk_secretsmanager::config::Region::new))
        .or_default_provider()
        .or_else("us-west-2");
    
    // Create AWS SDK config with behavior version
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    
    // Create service clients
    let secrets_manager = Arc::new(secretsmanager::Client::new(&aws_config));
    let s3 = Arc::new(s3::Client::new(&aws_config));
    let cloudwatch = Arc::new(cloudwatch::Client::new(&aws_config));
    
    AwsClients {
        secrets_manager,
        s3,
        cloudwatch,
    }
}

// Initialize application with AWS-integrated configuration
async fn initialize_app_config(aws_clients: &AwsClients) -> AppConfig {
    // Retrieve configuration from Secrets Manager
    let secret_name = std::env::var("APP_CONFIG_SECRET").unwrap_or_else(|_| "app/config".to_string());
    
    let secret_response = aws_clients.secrets_manager
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await
        .expect("Failed to retrieve secret");
    
    let secret_string = secret_response.secret_string()
        .expect("Secret value not found");
    
    // Parse JSON configuration
    let config: serde_json::Value = serde_json::from_str(secret_string)
        .expect("Invalid secret format");
    
    let region = config["region"].as_str().unwrap_or("us-west-2").to_string();
    
    let postgres_config: PostgresConfig = serde_json::from_value(config["postgres"].clone())
        .expect("Invalid Postgres configuration");
    
    let redis_config: RedisConfig = serde_json::from_value(config["redis"].clone())
        .expect("Invalid Redis configuration");
    
    let s3_config: S3Config = serde_json::from_value(config["s3"].clone())
        .expect("Invalid S3 configuration");
    
    let entra_config: EntraConfig = serde_json::from_value(config["entra"].clone())
        .expect("Invalid Entra configuration");
    
    AppConfig {
        region,
        postgres_config,
        redis_config,
        s3_config,
        entra_config,
    }
}

// Setup PostgreSQL connection with IAM authentication
async fn setup_postgres(config: &PostgresConfig, aws_clients: &AwsClients) -> sqlx::PgPool {
    let connection_string = if config.use_iam_auth {
        // Generate an auth token for RDS IAM authentication
        let rds_client = rds::Client::new(&aws_config::from_env().load().await);
        
        let auth_token = rds_client
            .generate_db_auth_token()
            .hostname(&config.host)
            .port(config.port)
            .username("app_user")
            .region(&aws_config::from_env().region().unwrap().to_string())
            .build()
            .await
            .expect("Failed to generate RDS auth token");
        
        format!(
            "postgres://app_user:{}@{}:{}/{}?sslmode=require",
            auth_token, config.host, config.port, config.database
        )
    } else {
        // Retrieve password from Secrets Manager
        let secret_name = format!("db/password/{}", config.database);
        
        let secret_response = aws_clients.secrets_manager
            .get_secret_value()
            .secret_id(secret_name)
            .send()
            .await
            .expect("Failed to retrieve database password");
        
        let password = secret_response.secret_string()
            .expect("Database password not found");
        
        format!(
            "postgres://app_user:{}@{}:{}/{}?sslmode=require",
            password, config.host, config.port, config.database
        )
    };
    
    // Create connection pool
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.connection_pool_size)
        .connect(&connection_string)
        .await
        .expect("Failed to connect to PostgreSQL")
}

// Setup Redis connection with TLS
async fn setup_redis(config: &RedisConfig) -> redis::Client {
    let scheme = if config.use_tls { "rediss" } else { "redis" };
    
    let connection_string = format!(
        "{}://{}:{}",
        scheme, config.primary_endpoint, config.port
    );
    
    redis::Client::open(connection_string)
        .expect("Failed to create Redis client")
}

// Entra authentication middleware
async fn entra_auth_middleware(
    State(entra_config): State<EntraConfig>,
    mut request: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    let auth_header = request.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_string())
            } else {
                None
            }
        });
    
    if let Some(token) = auth_header {
        // Validate JWT token (simplified example)
        match validate_jwt(&token, &entra_config) {
            Ok(claims) => {
                // Add user identity to request extensions
                request.extensions_mut().insert(UserIdentity {
                    user_id: claims.sub,
                    roles: claims.roles,
                    tenant_id: claims.tid,
                });
                
                next.run(request).await
            },
            Err(_) => {
                // Return 401 for invalid token
                axum::response::Response::builder()
                    .status(axum::http::StatusCode::UNAUTHORIZED)
                    .body(axum::body::Body::from("Invalid authentication token"))
                    .unwrap()
            }
        }
    } else {
        // Return 401 for missing token
        axum::response::Response::builder()
            .status(axum::http::StatusCode::UNAUTHORIZED)
            .body(axum::body::Body::from("Missing authentication token"))
            .unwrap()
    }
}

// Main application setup
pub async fn create_app() -> Router {
    // Initialize AWS services
    let aws_clients = initialize_aws(None).await;
    
    // Load application configuration
    let app_config = initialize_app_config(&aws_clients).await;
    
    // Initialize database connection
    let db_pool = setup_postgres(&app_config.postgres_config, &aws_clients).await;
    
    // Initialize Redis client
    let redis_client = setup_redis(&app_config.redis_config).await;
    
    // Create structured logger for CloudWatch
    let logger = create_cloudwatch_logger(&aws_clients.cloudwatch).await;
    
    // Create the Axum application with middleware
    Router::new()
        .route("/health", get(health_handler))
        .nest("/api", 
            build_api_router()
                .layer(middleware::from_fn_with_state(
                    app_config.entra_config.clone(),
                    entra_auth_middleware,
                ))
                .layer(middleware::from_fn(logging_middleware))
        )
        .with_state(app_config)
        .with_state(aws_clients)
        .with_state(db_pool)
        .with_state(redis_client)
        .with_state(logger)
}

// Health check handler
async fn health_handler() -> &'static str {
    "Service is healthy"
}
```

This roadmap prioritizes a practical AWS integration that:

1. **Puts security first**: IAM roles with least privilege, secure VPC configuration, and proper authentication integration with Microsoft Entra

2. **Focuses on essential AWS services**: The implementation centers around RDS (PostgreSQL), ElastiCache (Redis), and container deployment to ECS/Fargate

3. **Optimizes for Axum**: The implementation pattern leverages Axum's middleware and state management for AWS service integration

4. **Streamlines deployments**: GitLab CI/CD integration provides automated build, test, and deployment pipelines

5. **Ensures proper observability**: CloudWatch integration enables comprehensive logging, metrics, and alerting

This approach provides the fastest path to a secure, production-ready AWS deployment while avoiding unnecessary complexity and focusing on the specific technologies in your stack.

## References
- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust)
- [AWS Fargate Documentation](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/AWS_Fargate.html)
- [AWS RDS for PostgreSQL](https://aws.amazon.com/rds/postgresql/)
- [AWS ElastiCache for Redis](https://aws.amazon.com/elasticache/redis/)
- [AWS Secrets Manager](https://aws.amazon.com/secrets-manager/)
- [CloudWatch Observability](https://aws.amazon.com/cloudwatch/)
- [Microsoft Entra ID Documentation](https://learn.microsoft.com/en-us/entra/identity-platform/)
- [GitLab CI/CD Documentation](https://docs.gitlab.com/ee/ci/) 