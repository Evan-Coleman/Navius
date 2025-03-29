---
title: "Data Protection Guide"
description: "Comprehensive guide for implementing data protection in Navius applications, covering encryption, secure storage, data privacy, and compliance"
category: "Guides"
tags: ["security", "encryption", "data protection", "privacy", "PII", "compliance", "GDPR"]
last_updated: "April 7, 2025"
version: "1.0"
---

# Data Protection Guide

## Overview

This guide provides detailed instructions for implementing data protection in Navius applications. Data protection is critical for safeguarding sensitive information, maintaining user privacy, and ensuring compliance with regulations.

## Data Protection Concepts

### Types of Sensitive Data

- **Personally Identifiable Information (PII)** - Information that can identify an individual (names, email addresses, phone numbers)
- **Protected Health Information (PHI)** - Health-related information protected by regulations like HIPAA
- **Financial Data** - Payment information, account numbers, financial records
- **Authentication Data** - Passwords, security questions, biometric data
- **Business-Sensitive Data** - Intellectual property, trade secrets, business plans

### Data Protection Principles

1. **Data Minimization** - Collect and store only what is necessary
2. **Purpose Limitation** - Use data only for its intended purpose
3. **Storage Limitation** - Retain data only as long as necessary
4. **Integrity and Confidentiality** - Protect data from unauthorized access and accidental loss

## Encryption Implementation

### Configuration

Configure encryption in your `config/default.yaml`:

```yaml
encryption:
  provider: "aes_gcm"
  key_management: "kms"
  kms:
    provider: "aws"
    key_id: "your-key-id"
    region: "us-west-2"
  data_key_rotation:
    enabled: true
    rotation_period_days: 90
```

### Encryption Service

Implement the encryption service:

```rust
use navius::security::encryption::{EncryptionService, EncryptionConfig};

// Create encryption service
async fn create_encryption_service(config: &Config) -> Result<impl EncryptionService, Error> {
    let encryption_config = EncryptionConfig::from_config(config)?;
    let encryption_service = EncryptionService::new(encryption_config).await?;
    Ok(encryption_service)
}

// Encrypt data
async fn encrypt_data<T: Serialize>(
    data: &T,
    context: &EncryptionContext,
    encryption_service: &impl EncryptionService,
) -> Result<EncryptedData, Error> {
    let encrypted = encryption_service.encrypt(data, context).await?;
    Ok(encrypted)
}

// Decrypt data
async fn decrypt_data<T: DeserializeOwned>(
    encrypted: &EncryptedData,
    context: &EncryptionContext,
    encryption_service: &impl EncryptionService,
) -> Result<T, Error> {
    let decrypted = encryption_service.decrypt(encrypted, context).await?;
    Ok(decrypted)
}
```

### Encryption Context

Use encryption context to bind encryption to a specific context:

```rust
// Create an encryption context
let context = EncryptionContext::new()
    .with_user_id(user_id)
    .with_resource_type("payment_info")
    .with_resource_id(payment_id)
    .with_purpose("payment_processing");

// Encrypt with context
let encrypted = encryption_service.encrypt(&payment_info, &context).await?;

// Decrypt with the same context
let decrypted: PaymentInfo = encryption_service.decrypt(&encrypted, &context).await?;
```

### Envelope Encryption

Implement envelope encryption for better key management:

```rust
// Envelope encryption with data keys
async fn envelope_encrypt<T: Serialize>(
    data: &T,
    kms_service: &impl KmsService,
    encryption_service: &impl EncryptionService,
) -> Result<EnvelopeEncryptedData, Error> {
    // Generate a data key
    let data_key = kms_service.generate_data_key().await?;
    
    // Encrypt data with the data key
    let encrypted_data = encryption_service
        .encrypt_with_key(data, &data_key.plaintext)
        .await?;
    
    // Return envelope with encrypted data and encrypted key
    let envelope = EnvelopeEncryptedData {
        encrypted_data,
        encrypted_key: data_key.ciphertext,
        key_id: data_key.key_id,
    };
    
    Ok(envelope)
}

// Envelope decryption
async fn envelope_decrypt<T: DeserializeOwned>(
    envelope: &EnvelopeEncryptedData,
    kms_service: &impl KmsService,
    encryption_service: &impl EncryptionService,
) -> Result<T, Error> {
    // Decrypt the data key
    let data_key = kms_service
        .decrypt_data_key(&envelope.encrypted_key, &envelope.key_id)
        .await?;
    
    // Decrypt the data with the data key
    let decrypted = encryption_service
        .decrypt_with_key(&envelope.encrypted_data, &data_key)
        .await?;
    
    Ok(decrypted)
}
```

## Secure Database Storage

### Encrypted Fields in Database

Define a model with encrypted fields:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: Uuid,
    username: String,
    #[encrypted]
    email: String,
    #[encrypted]
    phone_number: Option<String>,
    #[encrypted(sensitive = true)]
    payment_info: Option<PaymentInfo>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### Database Repository with Encryption

Implement a repository that handles encryption:

```rust
struct UserRepository {
    db_pool: PgPool,
    encryption_service: Box<dyn EncryptionService>,
}

impl UserRepository {
    // Create a new user with encrypted fields
    async fn create(&self, user: User) -> Result<User, Error> {
        // Create encryption context
        let context = EncryptionContext::new()
            .with_resource_type("user")
            .with_resource_id(user.id);
        
        // Encrypt sensitive fields
        let encrypted_email = self.encryption_service
            .encrypt(&user.email, &context)
            .await?;
        
        let encrypted_phone = match user.phone_number {
            Some(phone) => Some(self.encryption_service.encrypt(&phone, &context).await?),
            None => None,
        };
        
        let encrypted_payment_info = match user.payment_info {
            Some(payment) => {
                let payment_context = context.clone().with_purpose("payment_processing");
                Some(self.encryption_service.encrypt(&payment, &payment_context).await?)
            },
            None => None,
        };
        
        // Store encrypted data in database
        let query = sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, phone_number, payment_info, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            user.id,
            user.username,
            encrypted_email.to_string(),
            encrypted_phone.map(|e| e.to_string()),
            encrypted_payment_info.map(|e| e.to_string()),
            user.created_at,
            user.updated_at,
        );
        
        let result = query.fetch_one(&self.db_pool).await?;
        
        Ok(user)
    }
    
    // Retrieve and decrypt user data
    async fn get_by_id(&self, id: Uuid) -> Result<User, Error> {
        // Query database for encrypted user
        let encrypted_user = sqlx::query!(
            r#"
            SELECT id, username, email, phone_number, payment_info, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id,
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        // Create encryption context
        let context = EncryptionContext::new()
            .with_resource_type("user")
            .with_resource_id(id);
        
        // Decrypt sensitive fields
        let email = self.encryption_service
            .decrypt::<String>(&EncryptedData::from_string(&encrypted_user.email)?, &context)
            .await?;
        
        let phone_number = match encrypted_user.phone_number {
            Some(phone) => {
                let encrypted = EncryptedData::from_string(&phone)?;
                Some(self.encryption_service.decrypt::<String>(&encrypted, &context).await?)
            },
            None => None,
        };
        
        let payment_info = match encrypted_user.payment_info {
            Some(payment) => {
                let encrypted = EncryptedData::from_string(&payment)?;
                let payment_context = context.clone().with_purpose("payment_processing");
                Some(self.encryption_service.decrypt::<PaymentInfo>(&encrypted, &payment_context).await?)
            },
            None => None,
        };
        
        // Construct and return decrypted user
        let user = User {
            id: encrypted_user.id,
            username: encrypted_user.username,
            email,
            phone_number,
            payment_info,
            created_at: encrypted_user.created_at,
            updated_at: encrypted_user.updated_at,
        };
        
        Ok(user)
    }
}
```

## Data Masking and Anonymization

### Data Masking

Implement data masking for displaying sensitive information:

```rust
use navius::security::masking::{MaskingService, MaskingStrategy};

// Create masking service
let masking_service = MaskingService::new();

// Mask PII with different strategies
fn mask_user_data(user: &User, masking_service: &MaskingService) -> MaskedUser {
    MaskedUser {
        id: user.id,
        username: user.username.clone(),
        email: masking_service.mask_email(&user.email),
        phone_number: user.phone_number.as_ref().map(|p| masking_service.mask_phone(p)),
        payment_info: user.payment_info.as_ref().map(|p| masking_service.mask_payment_info(p)),
    }
}

// Example masking strategies
let masked_email = masking_service.mask_email("john.doe@example.com"); // "j***.*****@e******.com"
let masked_phone = masking_service.mask_phone("+1-555-123-4567"); // "+*-***-***-4567"
let masked_card = masking_service.mask_card_number("4111111111111111"); // "************1111"
```

### Data Anonymization

Implement data anonymization for analytics:

```rust
use navius::security::anonymization::{AnonymizationService, AnonymizationStrategy};

// Create anonymization service
let anonymization_service = AnonymizationService::new();

// Anonymize data for analytics
fn anonymize_user_data(user: &User, anonymization_service: &AnonymizationService) -> AnonymizedUser {
    AnonymizedUser {
        id: anonymization_service.hash_id(user.id),
        age_range: anonymization_service.bin_age(user.age),
        region: anonymization_service.generalize_location(&user.location),
        activity_level: anonymization_service.categorize_activity(user.login_count),
    }
}

// Implement k-anonymity for data sets
async fn get_k_anonymized_dataset(
    users: Vec<User>,
    k: usize,
    anonymization_service: &AnonymizationService,
) -> Result<Vec<AnonymizedUser>, Error> {
    anonymization_service.k_anonymize(users, k).await
}
```

## Secure File Storage

### Encrypted File Storage

Store files securely with encryption:

```rust
use navius::storage::files::{FileStorage, EncryptedFileStorage};

// Create encrypted file storage
let file_storage = EncryptedFileStorage::new(
    S3FileStorage::new(s3_client),
    encryption_service,
);

// Store a file with encryption
async fn store_file(
    file_data: &[u8],
    file_name: &str,
    content_type: &str,
    user_id: Uuid,
    file_storage: &impl FileStorage,
) -> Result<FileMetadata, Error> {
    // Create metadata
    let metadata = FileMetadata {
        owner_id: user_id,
        content_type: content_type.to_string(),
        original_name: file_name.to_string(),
        created_at: Utc::now(),
    };
    
    // Store file with metadata
    let stored_file = file_storage.store(file_data, metadata).await?;
    
    Ok(stored_file.metadata)
}

// Retrieve a file with decryption
async fn get_file(
    file_id: Uuid,
    user_id: Uuid,
    file_storage: &impl FileStorage,
) -> Result<(Vec<u8>, FileMetadata), Error> {
    // Check access permission
    if !can_access_file(file_id, user_id).await? {
        return Err(Error::AccessDenied);
    }
    
    // Retrieve and decrypt file
    let file = file_storage.get(file_id).await?;
    
    Ok((file.data, file.metadata))
}
```

## Data Privacy Features

### Data Subject Rights

Implement features for GDPR compliance:

```rust
use navius::privacy::{DataSubjectService, DataSubjectRequest};

// Create data subject service
let data_subject_service = DataSubjectService::new(
    user_repository,
    activity_repository,
    file_storage,
);

// Handle right to access request
async fn handle_access_request(
    user_id: Uuid,
    data_subject_service: &DataSubjectService,
) -> Result<DataExport, Error> {
    let request = DataSubjectRequest::new(user_id, RequestType::Access);
    let export = data_subject_service.process_access_request(request).await?;
    Ok(export)
}

// Handle right to erasure (right to be forgotten)
async fn handle_erasure_request(
    user_id: Uuid,
    data_subject_service: &DataSubjectService,
) -> Result<ErasureConfirmation, Error> {
    let request = DataSubjectRequest::new(user_id, RequestType::Erasure);
    let confirmation = data_subject_service.process_erasure_request(request).await?;
    Ok(confirmation)
}

// Handle data portability request
async fn handle_portability_request(
    user_id: Uuid,
    format: ExportFormat,
    data_subject_service: &DataSubjectService,
) -> Result<PortableData, Error> {
    let request = DataSubjectRequest::new(user_id, RequestType::Portability)
        .with_export_format(format);
    let portable_data = data_subject_service.process_portability_request(request).await?;
    Ok(portable_data)
}
```

### User Consent Management

Implement consent tracking:

```rust
use navius::privacy::consent::{ConsentService, ConsentRecord};

// Create consent service
let consent_service = ConsentService::new(consent_repository);

// Record user consent
async fn record_user_consent(
    user_id: Uuid,
    purpose: &str,
    granted: bool,
    consent_service: &ConsentService,
) -> Result<ConsentRecord, Error> {
    let consent = ConsentRecord::new(
        user_id,
        purpose.to_string(),
        granted,
        Utc::now(),
    );
    
    let saved_consent = consent_service.record_consent(consent).await?;
    Ok(saved_consent)
}

// Check if user has consented to a specific purpose
async fn has_user_consented(
    user_id: Uuid,
    purpose: &str,
    consent_service: &ConsentService,
) -> Result<bool, Error> {
    let consented = consent_service.has_consent(user_id, purpose).await?;
    Ok(consented)
}

// Revoke consent
async fn revoke_consent(
    user_id: Uuid,
    purpose: &str,
    consent_service: &ConsentService,
) -> Result<(), Error> {
    consent_service.revoke_consent(user_id, purpose).await?;
    Ok(())
}
```

## Data Access Audit Logging

### Audit Trail Implementation

Create a comprehensive audit trail:

```rust
use navius::security::audit::{AuditService, AuditEvent, AuditEventType};

// Create audit service
let audit_service = AuditService::new(audit_repository);

// Log data access event
async fn log_data_access(
    user_id: Uuid,
    resource_type: &str,
    resource_id: Uuid,
    action: &str,
    audit_service: &AuditService,
) -> Result<(), Error> {
    let event = AuditEvent::new(
        user_id,
        AuditEventType::DataAccess,
        resource_type.to_string(),
        resource_id,
        action.to_string(),
        Utc::now(),
    );
    
    audit_service.log_event(event).await?;
    Ok(())
}

// Get audit trail for a resource
async fn get_resource_audit_trail(
    resource_type: &str,
    resource_id: Uuid,
    audit_service: &AuditService,
) -> Result<Vec<AuditEvent>, Error> {
    let events = audit_service
        .get_events_by_resource(resource_type, resource_id)
        .await?;
    
    Ok(events)
}

// Get audit trail for a user
async fn get_user_audit_trail(
    user_id: Uuid,
    audit_service: &AuditService,
) -> Result<Vec<AuditEvent>, Error> {
    let events = audit_service
        .get_events_by_user(user_id)
        .await?;
    
    Ok(events)
}
```

## Secure Data Transmission

### TLS Configuration

Configure secure transmission:

```rust
use navius::security::tls::{TlsConfig, TlsVersion};

// Configure TLS settings
let tls_config = TlsConfig {
    minimum_version: TlsVersion::Tls13,
    certificate_path: "/path/to/cert.pem".to_string(),
    private_key_path: "/path/to/key.pem".to_string(),
    verify_client: false,
};

// Apply TLS to HTTP server
let server = Server::new()
    .with_tls(tls_config)
    .bind("0.0.0.0:443")
    .serve(app);
```

### Secure Headers

Implement security headers for additional protection:

```rust
use navius::security::headers::SecurityHeadersLayer;

// Add security headers to all responses
let app = Router::new()
    .route("/", get(handler))
    .layer(SecurityHeadersLayer::new());

// Security headers include:
// - Strict-Transport-Security (HSTS)
// - Content-Security-Policy (CSP)
// - X-Content-Type-Options
// - X-Frame-Options
// - Referrer-Policy
```

## Data Breach Response

### Breach Detection

Implement breach detection:

```rust
use navius::security::breach::{BreachDetectionService, BreachAlert};

// Create breach detection service
let breach_detection = BreachDetectionService::new(
    audit_service,
    notification_service,
);

// Configure breach detection rules
breach_detection
    .add_rule(RateLimitRule::new("login_failure", 10, Duration::minutes(5)))
    .add_rule(UnusualAccessPatternRule::new())
    .add_rule(DataExfiltractionRule::new(1000, Duration::minutes(10)));

// Handle breach alert
async fn handle_breach_alert(
    alert: BreachAlert,
    response_service: &BreachResponseService,
) -> Result<(), Error> {
    // Log the alert
    response_service.log_alert(&alert).await?;
    
    // Notify security team
    response_service.notify_security_team(&alert).await?;
    
    // Take automated remediation actions
    match alert.severity {
        Severity::High => {
            response_service.lock_affected_accounts(&alert).await?;
            response_service.revoke_active_sessions(&alert).await?;
        },
        Severity::Medium => {
            response_service.require_reauthentication(&alert).await?;
        },
        Severity::Low => {
            // Just monitor
        }
    }
    
    Ok(())
}
```

## Testing Data Protection

### Unit Testing Encryption

```rust
#[tokio::test]
async fn test_encryption_service() {
    // Setup test encryption service
    let config = EncryptionConfig {
        provider: "aes_gcm".to_string(),
        key: generate_test_key(),
        ..Default::default()
    };
    
    let encryption_service = EncryptionService::new(config).await.unwrap();
    
    // Test data
    let sensitive_data = "sensitive information";
    let context = EncryptionContext::new().with_purpose("test");
    
    // Encrypt data
    let encrypted = encryption_service.encrypt(&sensitive_data, &context).await.unwrap();
    
    // Verify encrypted data is different from original
    assert_ne!(encrypted.ciphertext, sensitive_data.as_bytes());
    
    // Decrypt data
    let decrypted: String = encryption_service.decrypt(&encrypted, &context).await.unwrap();
    
    // Verify decryption works
    assert_eq!(decrypted, sensitive_data);
    
    // Verify wrong context fails
    let wrong_context = EncryptionContext::new().with_purpose("wrong");
    let result: Result<String, _> = encryption_service.decrypt(&encrypted, &wrong_context).await;
    assert!(result.is_err());
}
```

### Integration Testing Data Protection

```rust
#[tokio::test]
async fn test_data_protection_integration() {
    // Setup test app with data protection
    let app = test_app().await;
    
    // Create test user with sensitive data
    let user_data = UserCreate {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        phone_number: Some("+1-555-123-4567".to_string()),
        payment_info: Some(PaymentInfo {
            card_number: "4111111111111111".to_string(),
            expiry_date: "12/25".to_string(),
            cardholder_name: "Test User".to_string(),
        }),
    };
    
    // Create user
    let response = app.post("/users")
        .json(&user_data)
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let user: User = response.json().await;
    
    // Verify user was created with correct data
    assert_eq!(user.username, user_data.username);
    assert_eq!(user.email, user_data.email);
    assert_eq!(user.phone_number, user_data.phone_number);
    
    // Check database directly to verify encryption
    let db_user = sqlx::query!("SELECT * FROM users WHERE id = $1", user.id)
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    
    // Verify sensitive fields are encrypted in database
    assert_ne!(db_user.email, user_data.email);
    assert!(db_user.email.starts_with("ENC:"));
    
    if let Some(phone) = &db_user.phone_number {
        assert!(phone.starts_with("ENC:"));
    }
    
    if let Some(payment) = &db_user.payment_info {
        assert!(payment.starts_with("ENC:"));
    }
}
```

## Compliance Considerations

### GDPR Compliance

Key GDPR requirements for Navius applications:

1. **Lawful Basis for Processing**: Implement consent tracking
2. **Data Subject Rights**: Implement access, erasure, and portability features
3. **Data Protection by Design**: Use encryption and minimization strategies
4. **Breach Notification**: Implement detection and response capabilities

### HIPAA Compliance (Healthcare)

Key HIPAA requirements for healthcare applications:

1. **PHI Encryption**: Implement strong encryption for health data
2. **Access Controls**: Implement role-based access control
3. **Audit Logging**: Maintain comprehensive audit trails
4. **Business Associate Agreements**: Enable BAA compliance

### PCI DSS Compliance (Payment Data)

Key PCI DSS requirements for payment processing:

1. **Secure Transmission**: Implement TLS for all payment data
2. **Storage Restrictions**: Avoid storing sensitive authentication data
3. **Encryption**: Protect stored cardholder data with strong encryption
4. **Access Restrictions**: Limit access to payment data

## Best Practices

### Secure Development Practices

1. **Security Reviews**: Conduct regular security reviews of data handling code
2. **Dependency Scanning**: Regularly check dependencies for vulnerabilities
3. **Security Testing**: Include security tests in CI/CD pipeline
4. **Code Analysis**: Use static code analysis tools to identify security issues

### Operational Security

1. **Key Rotation**: Regularly rotate encryption keys
2. **Access Monitoring**: Monitor and audit data access
3. **Security Updates**: Keep all systems updated with security patches
4. **Incident Response**: Maintain an incident response plan

## Troubleshooting

### Common Issues

1. **Performance Impact**: Optimize encryption operations for performance
2. **Key Management Issues**: Ensure proper key backup and recovery
3. **Integration Challenges**: Verify compatibility with existing systems
4. **Compliance Gaps**: Regularly audit against compliance requirements

### Debugging Data Protection

```rust
// Enable detailed logging for data protection components
tracing_subscriber::fmt()
    .with_env_filter("navius::security=debug,navius::privacy=debug")
    .init();
```

## Related Resources

- [Security Best Practices](./security-best-practices.md)
- [Authentication Implementation Guide](./authentication-implementation.md)
- [Authorization Guide](./authorization-guide.md)
- [API Security Guide](./api-security.md)
- [OWASP Data Protection Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html) 