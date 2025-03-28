---
title: Document Templates
description: Templates for different types of documentation in the new structure
category: internal
tags:
  - documentation
  - templates
  - standards
related:
  - ../05_reference/standards/documentation-standards.md
last_updated: March 27, 2025
version: 1.0
status: active
---

# Document Templates

This file contains templates for different types of documentation in the Navius project. Use these templates when creating new documentation to ensure consistency in structure and formatting.

## Table of Contents

- [Getting Started Document](#getting-started-document)
- [Example Document](#example-document)
- [Contributing Document](#contributing-document)
- [Guide Document](#guide-document)
- [Reference Document](#reference-document)
- [API Reference Document](#api-reference-document)
- [Roadmap Document](#roadmap-document)

---

## Getting Started Document

```markdown
---
title: [Title]
description: [Brief description]
category: getting-started
tags:
  - beginner
  - [additional tags]
related:
  - [related document paths]
last_updated: March 27, 2025
version: 1.0
status: [draft|review|active]
---

# [Title]

## Overview

[Brief introduction explaining the purpose and scope of this document]

## Prerequisites

- [Prerequisite 1]
- [Prerequisite 2]
- [Prerequisite 3]

## Quick Start

[Step-by-step instructions for the most basic implementation]

```shell
# Example command
cargo run --example hello-world
```

## Key Concepts

### [Concept 1]

[Explanation]

### [Concept 2]

[Explanation]

## Next Steps

- [Link to next document in the learning path]
- [Link to related examples]
- [Link to more advanced concepts]

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| [Issue description] | [Solution steps] |
| [Issue description] | [Solution steps] |
```

---

## Example Document

```markdown
---
title: [Example Title]
description: [Brief description]
category: example
tags:
  - example
  - [additional tags]
related:
  - [related document paths]
last_updated: March 27, 2025
version: 1.0
status: [draft|review|active]
---

# [Example Title]

## Overview

[Brief introduction explaining what this example demonstrates]

## Prerequisites

- [Prerequisite 1]
- [Prerequisite 2]
- [Prerequisite 3]

## Implementation

### Step 1: [Description]

```rust
// Code for step 1
fn example_function() {
    println!("This is an example");
}
```

### Step 2: [Description]

```rust
// Code for step 2
fn another_function() {
    // Implementation
}
```

## Complete Example

```rust
// Complete working code
fn main() {
    example_function();
    another_function();
    
    println!("Example complete!");
}
```

## Key Takeaways

- [Important point 1]
- [Important point 2]
- [Important point 3]

## Variations

### [Variation 1]

[Description and code for variation 1]

### [Variation 2]

[Description and code for variation 2]
```

---

## Contributing Document

```markdown
---
title: [Contributing Title]
description: [Brief description]
category: contributing
tags:
  - contributing
  - [additional tags]
related:
  - [related document paths]
last_updated: March 27, 2025
version: 1.0
status: [draft|review|active]
---

# [Contributing Title]

## Overview

[Brief introduction to this contributing guide]

## Prerequisites

- [Prerequisite 1]
- [Prerequisite 2]
- [Prerequisite 3]

## Process

### Step 1: [Description]

[Detailed explanation]

### Step 2: [Description]

[Detailed explanation]

## Best Practices

- [Best practice 1]
- [Best practice 2]
- [Best practice 3]

## Examples

### Example 1: [Description]

[Code or process example]

### Example 2: [Description]

[Code or process example]

## Checklist

- [ ] [Checklist item 1]
- [ ] [Checklist item 2]
- [ ] [Checklist item 3]
```

---

## Guide Document

```markdown
---
title: [Guide Title]
description: [Brief description]
category: guide
tags:
  - [relevant tags]
related:
  - [related document paths]
last_updated: March 27, 2025
version: 1.0
status: [draft|review|active]
---

# [Guide Title]

## Overview

[Brief introduction explaining what this guide covers and who it's for]

## Prerequisites

- [Prerequisite 1]
- [Prerequisite 2]
- [Prerequisite 3]

## Detailed Guide

### Section 1: [Description]

[Detailed explanation with examples]

```rust
// Code example if applicable
fn example() {
    // Implementation
}
```

### Section 2: [Description]

[Detailed explanation with examples]

## Advanced Techniques

### Technique 1: [Description]

[Explanation and examples]

### Technique 2: [Description]

[Explanation and examples]

## Common Patterns

### Pattern 1: [Description]

[Explanation of pattern]

### Pattern 2: [Description]

[Explanation of pattern]

## Best Practices

- [Best practice 1]
- [Best practice 2]
- [Best practice 3]

## Related Resources

- [Related internal resources]
- [Related external resources]
```

---

## Reference Document

```markdown
---
title: [Reference Title]
description: [Brief description]
category: reference
tags:
  - reference
  - [additional tags]
related:
  - [related document paths]
last_updated: March 27, 2025
version: 1.0
status: [draft|review|active]
---

# [Reference Title]

## Overview

[Brief introduction explaining the purpose of this reference document]

## Specifications

| Property | Value | Description |
|----------|-------|-------------|
| [Property Name] | [Value] | [Description] |
| [Property Name] | [Value] | [Description] |
| [Property Name] | [Value] | [Description] |

## Detailed Reference

### [Item 1]

[Detailed description]

#### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| [Property] | [Type] | [Default] | [Description] |
| [Property] | [Type] | [Default] | [Description] |

#### Methods

##### `[Method Name]([parameters])`

[Description]

**Parameters:**

- `[parameter]` ([type]): [description]
- `[parameter]` ([type]): [description]

**Returns:**

- [Return type]: [description]

**Examples:**

```rust
// Example usage
let result = object.method_name(parameters);
```

### [Item 2]

[Repeat structure for additional items]
```

---

## API Reference Document

```markdown
---
title: [API Title]
description: [Brief description]
category: api-reference
tags:
  - api
  - [additional tags]
related:
  - [related document paths]
last_updated: March 27, 2025
version: 1.0
status: [draft|review|active]
---

# [API Title]

## Overview

[Brief introduction to this API]

## Base URL

```
[Base URL for API calls]
```

## Authentication

[Description of authentication method]

## Endpoints

### `[HTTP Method] [Endpoint Path]`

[Description of what this endpoint does]

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| [parameter] | [type] | [Yes/No] | [Description] |

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| [parameter] | [type] | [Yes/No] | [default] | [Description] |

**Request Body:**

```json
{
  "property1": "value1",
  "property2": "value2"
}
```

**Response:**

```json
{
  "status": "success",
  "data": {
    "property1": "value1",
    "property2": "value2"
  }
}
```

**Status Codes:**

| Status Code | Description |
|-------------|-------------|
| 200 | [Description] |
| 400 | [Description] |
| 401 | [Description] |
| 404 | [Description] |
| 500 | [Description] |

**Example:**

```bash
curl -X [METHOD] "[BASE_URL]/[ENDPOINT]" \
  -H "Authorization: Bearer [TOKEN]" \
  -H "Content-Type: application/json" \
  -d '{
    "property1": "value1",
    "property2": "value2"
  }'
```

### `[Additional Endpoints]`

[Repeat structure for additional endpoints]
```

---

## Roadmap Document

```markdown
---
title: [Roadmap Title]
description: [Brief description]
category: roadmap
tags:
  - roadmap
  - planning
  - [additional tags]
related:
  - [related document paths]
last_updated: March 27, 2025
version: 1.0
status: [planned|in-progress|completed]
---

# [Roadmap Title]

## Overview

[Brief introduction explaining the purpose and scope of this roadmap]

## Current State

[Description of the current state or challenges being addressed]

## Target State

[Description of the desired end state after implementation]

## Implementation Phases

### Phase 1: [Phase Name]

**Status:** [Not Started|In Progress|Completed]

[Description of this phase and its objectives]

1. **[Task 1]**
   - [Subtask a]
   - [Subtask b]
   - [Subtask c]
   
2. **[Task 2]**
   - [Subtask a]
   - [Subtask b]
   - [Subtask c]

### Phase 2: [Phase Name]

**Status:** [Not Started|In Progress|Completed]

[Description of this phase and its objectives]

1. **[Task 1]**
   - [Details]
   
2. **[Task 2]**
   - [Details]

## Success Criteria

[List of measurable criteria that define successful completion]

## Timeline

| Phase | Estimated Start | Estimated Completion | Actual Completion | Status |
|-------|----------------|----------------------|-------------------|--------|
| Phase 1 | [Date] | [Date] | [Date] | [Status] |
| Phase 2 | [Date] | [Date] | [Date] | [Status] |
| Phase 3 | [Date] | [Date] | [Date] | [Status] |

## Dependencies

- [Dependency 1]
- [Dependency 2]
- [Dependency 3]

## Stakeholders

- [Stakeholder 1]: [Role/Responsibility]
- [Stakeholder 2]: [Role/Responsibility]
- [Stakeholder 3]: [Role/Responsibility]
``` 