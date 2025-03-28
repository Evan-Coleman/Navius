---
title: "Roadmap Update Template"
description: "# [Feature Name] Roadmap"
category: roadmap
tags:
  - authentication
  - development
  - documentation
  - integration
  - testing
last_updated: March 23, 2025
version: 1.0
---
# Roadmap Update Template

When updating roadmap files, follow these guidelines carefully:

## 1. Document Structure

Convert existing roadmaps to this structure:
```markdown
# [Feature Name] Roadmap

## Overview
[Brief description of the feature and its goals]

## Current Status
[List of current implementation state, progress, and achievements]

## Target State
[Description of the end goal and desired features]

## Implementation Progress Tracking
[Detailed progress tracking sections]

## Implementation Status
[Overall status and next steps]

## Success Criteria
[Measurable success metrics]
```

## 2. Progress Tracking Format

Convert implementation sections to this format:

```markdown
### Phase 1: [Phase Name]
1. **[Feature Name]**
   - [ ] Task 1
   - [ ] Task 2
   - [ ] Task 3
   
   *Updated at: Not started*
```

## 3. Date Handling Guidelines

IMPORTANT: Follow these rules for "Updated at" timestamps:

1. For new sections:
   ```markdown
   *Updated at: Not started*
   ```

2. When marking tasks as complete:
   - Use the CURRENT system date (not future dates)
   - If the last update was today, DO NOT update the timestamp
   - Get current date: `date "+%B %d, %Y"`
   
   Example:
   ```markdown
   - [x] Completed task
   - [ ] Pending task
   
   *Updated at: March 24, 2025 - Added basic implementation*
   ```

3. Status updates without task completion:
   - Only update the date if adding significant information
   - Keep existing date if only changing task status

## 4. Implementation Status Section

Place this section before Success Criteria:

```markdown
## Implementation Status
- **Overall Progress**: [0-100]% complete
- **Last Updated**: [Current system date - use `date "+%B %d, %Y"`]
- **Next Milestone**: [Next feature or task to implement]
- **Current Focus**: [Current area of development]
```

## 5. Progress Calculation Guidelines

Calculate overall progress using these weightings:
- Phase 1 tasks: 40% of total
- Phase 2 tasks: 35% of total
- Phase 3 tasks: 25% of total

Within each phase:
- Divide percentage by number of features
- Mark feature progress based on completed tasks

Example:
```markdown
Phase 1 (40%):
- Feature 1 (20%): 2/4 tasks complete = 10%
- Feature 2 (20%): 1/4 tasks complete = 5%
Total Progress = 15%
```

## 6. Status Updates

When updating task status:

1. Individual tasks:
```markdown
- [x] Completed task    # Use [x] for completed
- [-] Abandoned task    # Use [-] for abandoned
- [~] In progress task  # Use [~] for in progress
- [ ] Not started task  # Use [ ] for not started
```

2. Feature status:
```markdown
*Updated at: March 24, 2025 - Status: [one of the following]*
- "Not started" - No tasks begun
- "In progress" - Some tasks started
- "Completed" - All tasks finished
- "Blocked" - Cannot proceed (add reason)
- "Abandoned" - Will not implement (add reason)
```

## 7. Module-Specific Updates

When updating module-specific information:

```markdown
## Module Information
- **Module**: [module::path]
- **Status**: [Not Started/In Progress/Complete]
- **Coverage**: [Current]% (Previous: [Last]%)
- **Last Updated**: [Current system date]

## Implementation Details
- **Added Tests**: [List new tests]
- **Fixed Issues**: [List fixes]
- **Known Gaps**: [List remaining work]

## How to Test
```bash
# Run module tests
cargo test -- module::path

# Check coverage
./scripts/coverage.sh -m module::path
```
```

## 8. Example Complete Update

### Before:
```markdown
### Phase 1: Core Features
1. **User Authentication**
   - Implement login
   - Add registration
   - Create password reset
```

### After:
```markdown
### Phase 1: Core Features
1. **User Authentication**
   - [x] Implement login
   - [~] Add registration
   - [ ] Create password reset
   
   *Updated at: March 24, 2025 - Completed login implementation, registration in progress*

## Implementation Status
- **Overall Progress**: 15% complete
- **Last Updated**: March 24, 2025
- **Next Milestone**: Complete user registration
- **Current Focus**: Authentication system
```

## 9. Common Mistakes to Avoid

1. **Date Handling**
   - ❌ Don't use future dates
   - ❌ Don't increment dates automatically
   - ❌ Don't update dates without changes
   - ✅ Use current system date
   - ✅ Keep existing date if no significant changes

2. **Progress Tracking**
   - ❌ Don't mark tasks complete without implementation
   - ❌ Don't skip status updates
   - ✅ Update overall progress when tasks complete
   - ✅ Include specific status messages

3. **Documentation**
   - ❌ Don't remove previous status information
   - ❌ Don't leave status messages vague
   - ✅ Include specific implementation details
   - ✅ Document any blockers or issues

## 10. References

- [Rust Documentation Guidelines](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html)
- [Semantic Versioning](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)

# Testing Roadmap Update Template

## Module Information
- **Module Name**: [e.g., core::utils::api_resource]
- **Implementation Status**: [Not Started / In Progress / Completed]
- **Last Updated**: March 22, 2025
- **Updated By**: [Your Name]

## Coverage Metrics
- **Current Coverage**: [Percentage from navius-coverage.json]
- **Previous Coverage**: [Percentage from last update]
- **Change**: [Increase/Decrease percentage]
- **Target Coverage**: [Typically 80% minimum]

## Implementation Progress
- [x] Completed items
- [ ] Items in progress
- [ ] Items not started

## Test Types Implemented
- [ ] Unit Tests
- [ ] Integration Tests
- [ ] Property-Based Tests
- [ ] Doc Tests

## Key Functionality Tested
- [List key functions/features tested]
- [Include both success and error paths]

## Remaining Test Gaps
- [List areas not yet covered by tests]
- [Prioritize critical paths]

## Next Steps
1. [First priority action]
2. [Second priority action]
3. [Additional actions]

## Notes
[Any additional information about testing approach or challenges]

## How to Run Tests
```bash
# Run all tests for this module
cargo test -- module::path

# Run coverage analysis
./scripts/coverage.sh -m module::path
```

## Post Roadmap Completion Documentation
 - We want to keep our docs updated with all changes. Please when finishing a section of a roadmap to add/update any docs related to it.

## Related Documents
- [Project Structure Roadmap](../completed/11_project_structure_future_improvements.md) - Future improvements
- [Documentation Overhaul](../12_document_overhaul.md) - Documentation plans

