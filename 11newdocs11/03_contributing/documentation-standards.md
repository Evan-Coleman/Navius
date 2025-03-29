---
title: "Documentation Standards"
description: "Guidelines and best practices for creating and maintaining documentation in the Navius project"
category: "Contributing"
tags: ["documentation", "standards", "guidelines", "contributing", "markdown"]
last_updated: "April 5, 2025"
---

# Documentation Standards

## Overview

High-quality documentation is essential for the success of the Navius project. This document outlines our documentation standards, including organization, formatting, and best practices to ensure that our documentation is consistent, comprehensive, and useful.

## Documentation Types

The Navius project includes several types of documentation, each with a specific purpose:

### 1. Getting Started
- Quick introduction to the project
- Installation and setup instructions
- Basic usage examples
- Located in `01_getting_started/`

### 2. Examples
- Code examples showcasing specific features
- Complete, working examples that users can run
- Located in `02_examples/`

### 3. Contributing Guides
- Guidelines for contributors
- Development setup instructions
- Process documentation
- Located in `03_contributing/`

### 4. User Guides
- Comprehensive guides for users
- Detailed usage instructions
- Best practices and patterns
- Located in `04_guides/`

### 5. Reference Documentation
- API references
- Configuration options
- Technical specifications
- Located in `05_reference/`

## Document Structure

### Required Elements

Every document should include:

1. **Frontmatter** with:
   - `title`: Concise, descriptive title
   - `description`: 1-2 sentence summary
   - `category`: Document category
   - `tags`: Relevant keywords for search
   - `last_updated`: Date of last significant update

2. **Title (H1)**: Document title matching the frontmatter title

3. **Overview**: Brief introduction explaining the document's purpose

4. **Body Content**: The main content organized with clear headings

5. **Related Resources**: Links to related documentation (when applicable)

### Example Template

```markdown
---
title: "Feature Name"
description: "Brief description of the feature"
category: "Category"
tags: ["tag1", "tag2"]
last_updated: "YYYY-MM-DD"
---

# Feature Name

## Overview

Brief introduction to the feature and its purpose.

## Usage

How to use the feature.

## Examples

Code examples demonstrating the feature.

## Related Resources

- [Related Document 1](./related-doc-1.md)
- [Related Document 2](./related-doc-2.md)
```

## Formatting Guidelines

### Markdown Usage

- Use Markdown for all documentation
- Follow the [Markdown Style Guide](./markdown-style-guide.md)
- Use semantic heading levels (H1 for title, H2 for sections, H3 for subsections)
- Use [GitLab Flavored Markdown](https://docs.gitlab.com/ee/user/markdown.html) extensions when helpful

### Code Examples

- Use syntax highlighting for code blocks:
  ```rust
  fn example() -> Result<(), Error> {
      println!("This is an example");
      Ok(())
  }
  ```
- Include comments in complex code examples
- Show both function calls and expected outputs
- Use realistic examples that solve common problems

### Links

- Use relative links for internal documentation
- Use descriptive link text instead of "click here" or URLs
- Check links periodically to ensure they aren't broken
- For external resources, consider including the date accessed

### Images and Diagrams

- Include alt text for accessibility
- Use SVG format when possible for diagrams
- Keep images in an `assets` directory next to markdown files
- Include source files for diagrams when possible
- Optimize images for web viewing

## Content Guidelines

### Writing Style

- Use clear, direct language
- Write in present tense
- Use active voice
- Be concise but complete
- Target a technical audience but avoid unnecessary jargon
- Use consistent terminology throughout all documentation

### Content Organization

- Start with the most important information
- Group related information together
- Use lists and tables for better readability
- Include examples for complex concepts
- Break long documents into logical sections with clear headings

### Updates and Maintenance

- Review documentation for each release
- Update documentation when features change
- Include "last_updated" date in frontmatter
- Archive rather than delete obsolete documentation
- Follow the [No Legacy Code Rule](../05_reference/standards/no-legacy-code.md) for documentation

## Documentation Workflow

### Creating New Documentation

1. Identify the appropriate documentation type and location
2. Use the standard template for the document type
3. Write the content following these standards
4. Submit for review alongside related code changes

### Reviewing Documentation

- Ensure adherence to these standards
- Verify technical accuracy
- Check for completeness
- Review for clarity and readability
- Validate links and examples

### Publishing Documentation

- Documentation is published automatically from the main branch
- Changes follow the same workflow as code changes

## Tools and Resources

### Recommended Tools

- Visual Studio Code with Markdown extensions
- [Mermaid](https://mermaid-js.github.io/) for diagrams
- [markdownlint](https://github.com/DavidAnson/markdownlint) for Markdown validation

### CI/CD Integration

- Documentation changes trigger the documentation validation workflow
- Broken links and formatting issues are automatically detected
- Documentation is deployed alongside code changes

## Success Criteria

Documentation meets our standards when it:

1. Is complete, covering all aspects of the subject
2. Is accurate and up-to-date
3. Follows the formatting and structure guidelines
4. Is clear and understandable to the target audience
5. Includes appropriate examples and illustrations
6. Links to related resources

## Related Resources

- [Markdown Style Guide](./markdown-style-guide.md)
- [Contributing Guidelines](./contributing-guidelines.md)
- [Code Review Process](./code-review-process.md) 