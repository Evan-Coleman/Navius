---
title: "Documentation Guidelines"
description: ""
category: "Documentation"
tags: []
last_updated: "March 28, 2025"
version: "1.0"
---

# Documentation Guidelines

This guide provides standards and best practices for contributing to Navius framework documentation. Following these guidelines ensures our documentation remains consistent, clear, and valuable to users.

## Documentation Structure

Navius documentation is organized into the following categories:

1. **Getting Started**: Introductory guides for new users
2. **Guides**: Comprehensive how-to guides on specific topics
3. **Examples**: Practical code examples demonstrating features
4. **API Reference**: Detailed API specifications and usage
5. **Architecture**: Framework design and architectural concepts
6. **Contributing**: Guidelines for contributors

## Markdown Standards

All Navius documentation is written in Markdown and built with mdBook. The following standards apply:

### File Naming

- Use kebab-case for all file names (e.g., `getting-started.md`)
- Make file names descriptive and concise
- Avoid special characters except for hyphens
- Group related files in appropriate directories

### Document Structure

- Start with a level 1 heading (`# Title`) that matches the navigation title
- Use a logical heading structure (don't skip levels)
- Include a brief introduction after the title
- Organize content with clear section headings
- End with related resources or next steps where appropriate

### Formatting

- Use **bold** for emphasis on important concepts
- Use *italics* for new terms or secondary emphasis
- Use `code formatting` for:
  - Code snippets
  - File names
  - Command-line instructions
  - API endpoints
- Use blockquotes for important notes or cautions
- Use tables for structured data comparison

### Code Blocks

- Always specify the language for syntax highlighting:
  ```rust
  fn main() {
      println!("Hello, world!");
  }
  ```
- Add comments to explain complex code
- Use complete, runnable examples when possible
- Keep examples concise but complete
- Ensure code examples follow Navius coding standards

### Links

- Use relative links for internal documentation
- Use descriptive link text (avoid "click here")
- Link first mentions of related concepts
- Check all links before submitting contributions

## Content Guidelines

### Language Style

- Write in clear, direct language
- Use present tense: "The function returns..." not "The function will return..."
- Use active voice: "The service processes requests" not "Requests are processed by the service"
- Address the reader directly: "You can configure..." not "The user can configure..."
- Be concise but complete - don't sacrifice clarity for brevity

### Technical Accuracy

- Verify all technical information
- Test all code examples
- Include version information when relevant
- Document limitations and edge cases
- Identify prerequisites clearly

### Inclusivity

- Use inclusive language
- Avoid unnecessary technical jargon
- Explain acronyms on first use
- Provide context for complex concepts
- Consider users of different experience levels

## Documentation Types

### Tutorials

Tutorials walk users through specific tasks step-by-step:

- Include a clear goal statement
- List prerequisites at the beginning
- Break down into small, manageable steps
- Explain the "why" alongside the "how"
- Include screenshots or diagrams when helpful
- Verify all steps work as documented

### Conceptual Guides

Conceptual guides explain architecture, design patterns, and key concepts:

- Begin with a clear definition
- Explain the purpose and benefits
- Use diagrams to illustrate relationships
- Connect to practical applications
- Compare with alternatives when relevant
- Link to relevant API documentation

### API Documentation

API documentation should be comprehensive and precise:

- Document all public APIs
- Include method signatures and return types
- Describe parameters and return values
- Provide usage examples
- Document exceptions and error conditions
- Include both simple and complex examples

### Example Projects

Example projects demonstrate complete implementations:

- Include a brief overview of what the example demonstrates
- List key features and patterns shown
- Provide complete, runnable code
- Include setup and running instructions
- Explain key portions of code with comments
- Link to relevant guides and API documentation

## Special Elements

### Admonitions

Use admonitions to highlight important information:

> **Note**: This is useful information that users should know.

> **Warning**: This alerts users to potential problems or important cautions.

> **Tip**: This provides helpful advice to improve user experience.

### Diagrams

Include diagrams for complex concepts or relationships:

- Use mermaid.js for diagrams when possible
- Keep diagrams simple and focused
- Include alt text for accessibility
- Use consistent styling across diagrams
- Include a text explanation to complement the diagram

### Screenshots

When including screenshots:

- Capture only the relevant portion of the screen
- Use annotations to highlight important elements
- Ensure text is readable at the image size
- Include alt text for accessibility
- Consider dark/light mode variants for UI elements

## Documentation Process

### Creating New Documentation

1. **Plan**: Identify the purpose and audience
2. **Outline**: Create a logical structure
3. **Write**: Draft the content following the guidelines
4. **Review**: Self-review for clarity, accuracy, and completeness
5. **Submit**: Create a pull request with your changes

### Updating Existing Documentation

1. **Identify**: Find documentation that needs updating
2. **Verify**: Confirm the current state and needed changes
3. **Update**: Make changes following the guidelines
4. **Test**: Ensure all examples and procedures still work
5. **Submit**: Create a pull request with your changes

### Pull Request Requirements

Documentation pull requests should:

- Focus on specific, related documentation changes
- Include a clear description of what was changed and why
- Link to related issues or discussions
- Pass all automatic checks (links, formatting)
- Be self-contained and not dependent on other pending changes

## Building and Testing Documentation

### Local Testing

Before submitting changes, build and test the documentation locally:

```bash
cd docs
mdbook build
```

Open `book/index.html` in your browser to preview the changes.

Check for:
- Correct rendering of all elements
- Working internal and external links
- Proper code syntax highlighting
- Logical content flow
- Mobile responsiveness

### Link Checking

Run the link checker to find any broken links:

```bash
cd docs
mdbook-linkcheck
```

Fix any issues before submitting your changes.

## Additional Resources

- [Markdown Syntax Guide](https://www.markdownguide.org/basic-syntax/)
- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [Technical Writing Best Practices](https://developers.google.com/tech-writing/one)
- [Inclusion in Technical Documentation](https://developers.google.com/style/inclusive-documentation) 
