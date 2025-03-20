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
- **Last Updated**: March 20, 2024
- **Next Milestone**: [Name of next feature to implement]
```

## 4. When updating progress

When a task is completed:

```markdown
   - [x] Task 1  
   
   *Updated at: March 25, 2024 - Implemented basic version with unit tests*
```

And update the overall progress:

```markdown
## Implementation Status
- **Overall Progress**: 5% complete
- **Last Updated**: March 25, 2024
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
- **Last Updated**: March 20, 2024
- **Next Milestone**: Example Feature
``` 