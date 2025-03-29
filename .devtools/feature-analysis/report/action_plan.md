# Feature Flag Optimization Action Plan

## Phase 3: Core Module Optimization

### 1. Auth Module (High Priority)
- **Current Status**: 11 files with no feature gates
- **Dependencies to Make Optional**: 
  - jsonwebtoken
  - oauth2
  - reqwest-middleware (partially)
- **Implementation Steps**:
  1. Update module exports in src/core/auth/mod.rs with #[cfg(feature = "auth")]
  2. Update auth service registration in router files with feature gates
  3. Make dependencies optional in Cargo.toml and map to auth feature
  4. Add feature documentation to all auth module files
  5. Ensure tests are properly feature-gated
- **Success Criteria**: Can build without auth feature, reducing binary size

### 2. Database Module (High Priority)
- **Current Status**: 0 files identified (likely in a different location)
- **Dependencies to Make Optional**:
  - Any database client libraries
  - Postgres dependencies
  - SQLite dependencies
- **Implementation Steps**:
  1. Locate all database-related files
  2. Update module exports with #[cfg(feature = "database")]
  3. Make dependencies optional in Cargo.toml
  4. Add conditional compilation for different database providers
  5. Update database service registration with feature gates
- **Success Criteria**: Can build without database features, reducing binary size

### 3. Redis/Caching Module (Medium Priority)
- **Current Status**: 0 files identified (likely in a different location)
- **Dependencies to Make Optional**:
  - Redis client libraries
  - Moka or other in-memory cache libraries
- **Implementation Steps**:
  1. Locate all caching-related files
  2. Update module exports with #[cfg(feature = "caching")]
  3. Make dependencies optional in Cargo.toml
  4. Add conditional provider registration
- **Success Criteria**: Can build without caching features, reducing binary size

## Phase 4: App Module Optimization

### 1. API Controllers (Medium Priority)
- **Current Status**: Need to identify controller files
- **Implementation Steps**:
  1. Identify controllers that depend on optional features
  2. Add feature gates to controller registrations
  3. Add conditional compilation for endpoint handlers
- **Success Criteria**: Endpoints for disabled features aren't compiled or registered

### 2. Service Implementations (Medium Priority)
- **Current Status**: Need to identify service implementations
- **Implementation Steps**:
  1. Identify service implementations that depend on optional features
  2. Add feature gates to service registrations
  3. Add conditional compilation for service implementations
- **Success Criteria**: Services for disabled features aren't compiled

## Phase 5: Measurement & Documentation

### 1. Size Comparison Tool
- **Current Status**: Basic size measurement in analyze_features.sh
- **Implementation Steps**:
  1. Enhance analyze_features.sh to measure more detailed metrics
  2. Create a comparison report template
  3. Add CI job to track binary size over time
- **Success Criteria**: Automated reporting on binary size changes

### 2. Feature Usage Documentation
- **Current Status**: Basic documentation in feature_inventory.md
- **Implementation Steps**:
  1. Create a comprehensive feature usage guide
  2. Document each feature's purpose, dependencies, and impact
  3. Add examples for common feature combinations
- **Success Criteria**: Clear documentation on feature usage

### 3. Developer Guidelines
- **Current Status**: No formal guidelines
- **Implementation Steps**:
  1. Create guidelines for adding new features
  2. Document best practices for feature gates
  3. Add examples of proper feature gating
- **Success Criteria**: Consistent approach to feature gating

## Implementation Timeline

1. **Week 1**: Complete Auth Module Optimization
2. **Week 2**: Complete Database Module Optimization
3. **Week 3**: Complete Redis/Caching Module Optimization
4. **Week 4**: Complete App Module Optimization
5. **Week 5**: Complete Measurement & Documentation

## Success Metrics

- **Binary Size Reduction**: At least 30% reduction for minimal builds
- **Compilation Time**: At least 20% reduction for targeted builds
- **Code Quality**: All conditional features properly documented
- **Feature Independence**: Ability to build with only core features 