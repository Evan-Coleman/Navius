---
title: "Security Best Practices"
description: "Essential security practices and guidelines for building secure Navius applications"
category: "Guides"
tags: ["security", "best practices", "authentication", "authorization", "encryption", "vulnerabilities"]
last_updated: "April 6, 2025"
version: "1.0"
---

# Security Best Practices

## Overview

This guide provides comprehensive security best practices for developing and maintaining Navius applications. Following these practices will help protect your application against common security threats and vulnerabilities.

## General Security Principles

### Defense in Depth

Implement multiple layers of security controls, so that if one layer fails, others are still in place:

- Implement network security (firewalls, HTTPS)
- Use application-level security (authentication, authorization)
- Secure your data layer (encryption, access controls)
- Apply infrastructure security (secure deployments, updates)

### Least Privilege

Grant only the minimum permissions needed:

- Use role-based access control for users
- Run services with minimal permissions
- Restrict database user permissions
- Limit API access scopes

```rust
// Example: Creating a user with minimal permissions
let user = sqlx::query_as::<_, User>(
    "INSERT INTO users (email, name, role) VALUES ($1, $2, 'basic_user') RETURNING *"
)
.bind(email)
.bind(name)
.fetch_one(&pool)
.await?;
```

### Secure by Default

Configure systems to be secure out of the box:

- Enable security features by default
- Set secure defaults for all configuration
- Require explicit opt-out for security features
- Document security implications of configuration changes

```rust
// Example: Secure defaults for session configuration
let session_config = SessionConfig {
    cookie_secure: true, // Require HTTPS
    cookie_http_only: true, // Prevent JavaScript access
    cookie_same_site: SameSite::Strict, // Prevent CSRF
    expire_after: Some(Duration::hours(2)), // Short session lifetime
    ..Default::default()
};
```

### Keep It Simple

Complexity is the enemy of security:

- Choose simple, well-understood solutions
- Remove unused code and dependencies
- Document security-critical components
- Use standard, vetted security libraries

## Authentication Best Practices

### Implement Multi-Factor Authentication

```rust
// Example: Implementing MFA verification
async fn verify_mfa(
    user_id: Uuid,
    totp_code: &str,
    totp_service: &TotpService,
) -> Result<bool, ServiceError> {
    let user = get_user_by_id(user_id).await?;
    
    if user.mfa_enabled {
        return totp_service.verify(user.mfa_secret, totp_code).await;
    }
    
    Ok(true) // MFA not required
}
```

### Secure Password Handling

- Use strong, adaptive hashing algorithms (Argon2id, bcrypt)
- Implement account lockout after failed attempts
- Enforce strong password policies
- Support secure password reset flows

```rust
// Example: Secure password hashing with Argon2id
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;

fn hash_password(password: &str) -> Result<String, argon2::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
```

### Secure Session Management

- Use short session timeouts
- Implement secure cookie attributes
- Regenerate session IDs after privilege changes
- Provide session revocation capabilities

## Authorization Best Practices

### Implement Fine-Grained Authorization

```rust
// Example: Role-based authorization middleware
async fn authorize(
    req: Request<State>,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = req.extensions().get::<User>().ok_or(StatusCode::UNAUTHORIZED)?;
    let required_role = req.extensions().get::<RequiredRole>().ok_or(StatusCode::FORBIDDEN)?;
    
    if !user.has_role(required_role) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}
```

### Authorization for All Resources

- Protect all sensitive resources, not just UI pages
- Implement authorization at the API level
- Secure static resources when needed
- Check permissions on every request

### Validate Authorization Context

- Verify user identity on each request
- Check token expiration and validity
- Validate permissions against current data
- Consider time-of-check vs. time-of-use issues

## Data Protection

### Encrypt Sensitive Data

- Use TLS/HTTPS for all connections
- Encrypt sensitive data at rest
- Use strong, standard encryption algorithms
- Manage encryption keys securely

```rust
// Example: Encrypting sensitive data
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, anyhow::Error> {
    let cipher = Aes256Gcm::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(b"unique nonce"); // Use a unique nonce per encryption
    
    Ok(cipher.encrypt(nonce, plaintext)?)
}

fn decrypt_data(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, anyhow::Error> {
    let cipher = Aes256Gcm::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(b"unique nonce"); // Must match the encryption nonce
    
    Ok(cipher.decrypt(nonce, ciphertext)?)
}
```

### Protect Against SQL Injection

- Use parameterized queries for all database operations
- Validate and sanitize all user inputs
- Apply least privilege to database users
- Consider using an ORM with built-in protections

```rust
// Example: Using parameterized queries with SQLx
let users = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE username = $1 AND active = $2"
)
.bind(username) // Parameters are safely bound
.bind(true)
.fetch_all(&pool)
.await?;

// NEVER do this:
// let query = format!("SELECT * FROM users WHERE username = '{}'", username);
```

### Secure File Handling

- Validate file uploads (type, size, content)
- Store files outside the web root
- Use anti-virus scanning when appropriate
- Generate random filenames to prevent path traversal

## API Security

### Secure API Design

- Implement proper authentication for all APIs
- Use HTTPS for all API communication
- Apply rate limiting to prevent abuse
- Version your APIs to manage security changes

### Prevent CSRF Attacks

- Use anti-CSRF tokens for state-changing operations
- Implement SameSite cookie restrictions
- Validate the Origin and Referer headers
- Consider using custom request headers for APIs

```rust
// Example: Adding CSRF protection middleware
async fn csrf_protection(
    req: Request<State>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip for GET, HEAD, OPTIONS requests
    if matches!(req.method(), &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return Ok(next.run(req).await);
    }
    
    // Validate CSRF token
    let csrf_cookie = req.cookie("csrf_token").ok_or(StatusCode::FORBIDDEN)?;
    let csrf_header = req.header("X-CSRF-Token").ok_or(StatusCode::FORBIDDEN)?;
    
    if csrf_cookie.value() != csrf_header.as_str() {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}
```

### Mitigate Cross-Site Scripting (XSS)

- Encode output in the correct context (HTML, JavaScript, CSS)
- Use Content Security Policy (CSP) headers
- Implement XSS filters for user-generated content
- Use frameworks that automatically escape output

## Infrastructure Security

### Secure Deployment Practices

- Use immutable infrastructure
- Scan container images for vulnerabilities
- Apply the principle of least privilege to services
- Implement proper secrets management

### Dependency Management

- Regularly update dependencies
- Scan for known vulnerabilities (e.g., with `cargo audit`)
- Pin dependency versions for predictability
- Monitor security advisories for your dependencies

```shell
# Example: Checking for vulnerabilities in dependencies
cargo audit
```

### Logging and Monitoring

- Log security-relevant events
- Implement centralized log collection
- Set up alerts for suspicious activities
- Regularly review security logs

```rust
// Example: Logging security events
use tracing::{error, info, warn};

async fn login_attempt(username: &str, success: bool, ip: &str) {
    if success {
        info!(
            username = username,
            ip = ip,
            event = "login_success",
            "Successful login"
        );
    } else {
        warn!(
            username = username,
            ip = ip,
            event = "login_failure",
            "Failed login attempt"
        );
    }
}
```

## Security Testing

### Implement Security Testing

- Include security tests in your CI/CD pipeline
- Perform regular security assessments
- Conduct penetration testing before major releases
- Consider bug bounty programs for mature applications

### Vulnerability Scanning

- Scan dependencies for known vulnerabilities
- Check containers for security issues
- Implement dynamic application security testing
- Use static analysis to find common security issues

## Incident Response

### Prepare for Security Incidents

- Develop an incident response plan
- Define roles and responsibilities
- Document contact information for key personnel
- Establish communication channels for incidents

### Respond to Vulnerabilities

- Have a process for receiving security reports
- Establish a timeline for addressing vulnerabilities
- Communicate transparently about security issues
- Provide security patches promptly

## Regulatory Compliance

### Data Protection Regulations

- Identify applicable regulations (GDPR, CCPA, etc.)
- Implement data minimization practices
- Provide mechanisms for data subject rights
- Document your compliance measures

### Industry-Specific Requirements

- Address industry-specific security requirements
- Implement required security controls
- Maintain compliance documentation
- Perform regular compliance reviews

## Security Checklist

Use this checklist to assess your application's security:

- [ ] HTTPS is enforced for all communications
- [ ] Authentication is required for sensitive operations
- [ ] Authorization checks are implemented for all resources
- [ ] Passwords are stored using strong hashing algorithms
- [ ] Sessions are managed securely
- [ ] Input validation is implemented for all user inputs
- [ ] Output encoding is used to prevent XSS
- [ ] Sensitive data is encrypted at rest
- [ ] SQL injection protections are in place
- [ ] CSRF protections are implemented
- [ ] Security headers are configured properly
- [ ] Error handling does not expose sensitive information
- [ ] Logging captures security-relevant events
- [ ] Dependency vulnerabilities are regularly checked
- [ ] Security testing is part of the development process

## Related Resources

- [Authentication Implementation Guide](./authentication-implementation.md)
- [Authorization Guide](./authorization-guide.md)
- [Data Protection Guide](./data-protection.md)
- [API Security Guide](./api-security.md)
- [Security Headers Guide](./security-headers.md) 