# Codebase Cleanup Prompt

## Introduction
This prompt is designed to help with systematically fixing errors and updating documentation after implementing the Pet API Database Integration. 

## Background Context
- We've completed the Pet API Database Integration roadmap (#16)
- We have approximately 60 errors in tests and 32 build errors
- Documentation and implementation patterns need updating for consistency
- We're following the Codebase Cleanup Roadmap (#17)

## Current Focus
We're working on [PHASE] of the Codebase Cleanup Roadmap.

## Specific Request
[DETAILED REQUEST ABOUT THE SPECIFIC TASK]

## Required Approach
- Start by analyzing errors at their root cause
- Use a bottom-up approach to fix dependencies first
- Ensure backward compatibility where possible
- Update tests alongside code fixes
- Follow existing architectural patterns
- Document changes clearly 

## Guidelines for AI

### General Guidelines
- Focus on fixing errors first, optimizations later
- Keep changes minimal but comprehensive
- Consider all potential side effects of changes
- Update documentation alongside code changes

### Style Guidelines
- Follow existing Rust code style
- Keep consistent module and import structure
- Update all affected tests when changing interfaces
- Document public APIs

### Error Resolution Process
1. Start with `cargo check` or `cargo build -v` to see detailed errors
2. Group errors by root cause
3. Fix lowest-level dependencies first
4. Rerun tests after each significant change
5. Update documentation to reflect changes

### Testing Guidelines
- Ensure all changed components have tests
- Update test fixtures for any model changes
- Use test isolation for database tests
- Mock external dependencies consistently

### Documentation Updates
- Keep README and other docs in sync with code
- Update API documentation for any endpoint changes
- Follow date format standards (June 24, 2024) for timestamps
- Include examples for new patterns

## Expected Outcome
After implementing the changes, we should:
- Have zero build errors
- Have all tests passing
- Have up-to-date documentation
- Maintain consistent implementation patterns 