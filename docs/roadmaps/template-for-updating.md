# Roadmap Update Template

When updating the remaining roadmap files, use the following patterns to add progress tracking:

## 1. Change "Implementation Steps" to "Implementation Progress Tracking"

```markdown
## Implementation Progress Tracking
```

## 2. Convert bullet points to checkboxes with "Updated at" sections

For each implementation point, convert:

```markdown
1. **Feature Name**
   - Task 1
   - Task 2
   - Task 3
   - Task 4
```

To:

```markdown
1. **Feature Name**
   - [ ] Task 1
   - [ ] Task 2
   - [ ] Task 3
   - [ ] Task 4
   
   *Updated at: Not started*
```

## 3. Add Implementation Status section

Add this section before the Success Criteria:

```markdown
## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 22, 2025
- **Next Milestone**: [Name of next feature to implement]
```

## 4. When updating progress

When a task is completed:

```markdown
   - [x] Task 1  
   
   *Updated at: March 22, 2025 - Implemented basic version with unit tests*
```

And update the overall progress:

```markdown
## Implementation Status
- **Overall Progress**: 5% complete
- **Last Updated**: March 22, 2025
- **Next Milestone**: Continue with Feature Name implementation
```

## Example

### Before:
```markdown
### Phase 1: Example Phase
1. **Example Feature**
   - Implement X
   - Create Y
   - Add Z
   - Build A
```

### After:
```markdown
### Phase 1: Example Phase
1. **Example Feature**
   - [ ] Implement X
   - [ ] Create Y
   - [ ] Add Z
   - [ ] Build A
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 22, 2025
- **Next Milestone**: Example Feature
```

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