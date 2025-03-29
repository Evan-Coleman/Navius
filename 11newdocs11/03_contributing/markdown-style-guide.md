---
title: "Markdown Style Guide"
description: "Guidelines for consistent markdown formatting in Navius documentation"
category: "Contributing"
tags: ["documentation", "markdown", "style", "guidelines", "formatting"]
last_updated: "April 2, 2025"
version: "1.0"
---

# Markdown Style Guide

This guide outlines the standards and best practices for writing markdown documentation for the Navius project. Following these guidelines ensures consistency across all documentation.

## Table of Contents

- [General Formatting](#general-formatting)
- [Headings](#headings)
- [Lists](#lists)
- [Links](#links)
- [Images](#images)
- [Code Blocks](#code-blocks)
- [Tables](#tables)
- [Blockquotes](#blockquotes)
- [Horizontal Rules](#horizontal-rules)
- [Frontmatter](#frontmatter)
- [Accessibility Considerations](#accessibility-considerations)

## General Formatting

- Use a single blank line between paragraphs
- Use two trailing spaces for line breaks within paragraphs
- Use UTF-8 character encoding
- Use consistent indentation (2 spaces recommended)

## Headings

- Use ATX-style headings with a space after the hash signs (`#`)
- Use title case for main headings (Level 1) and sentence case for subheadings
- Leave one blank line before and after headings
- Do not skip heading levels (e.g., don't go from `##` to `####`)

```markdown
# Main Heading (Title Case)

## Secondary heading (Sentence case)

### Tertiary heading (Sentence case)
```

## Lists

### Unordered Lists

- Use hyphens (`-`) for list items
- Indent nested list items by 2 spaces
- Leave a blank line before and after lists

```markdown
- First item
- Second item
  - Nested item
  - Another nested item
- Third item
```

### Ordered Lists

- Use numbers followed by periods (`1.`, `2.`, etc.)
- Use `1.` for all items in ordered lists (let markdown handle the numbering)
- Indent nested list items by 3 spaces (to align with parent text)

```markdown
1. First item
1. Second item
   1. Nested item
   1. Another nested item
1. Third item
```

## Links

- Use reference-style links for repeated URLs
- Use descriptive link text (avoid "click here" or "this link")
- For internal documentation links, use relative paths
- For external links, include the full URL with protocol

```markdown
[Descriptive link text](URL)

[Reference-style link][reference]

[reference]: https://example.com
```

## Images

- Include alt text for all images
- Use reference-style image syntax for consistency
- Keep image file sizes reasonable (optimize before including)

```markdown
![Alt text for the image](path/to/image.jpg "Optional title")

![Alt text][image-reference]

[image-reference]: path/to/image.jpg "Optional title"
```

## Code Blocks

### Inline Code

- Use single backticks (`` ` ``) for inline code
- Use inline code for variable names, function names, and short commands

```markdown
Use the `config.get()` method to retrieve configuration values.
```

### Code Blocks

- Use triple backticks (` ``` `) with a language identifier for syntax highlighting
- Do not include language identifiers after the closing triple backticks
- Leave blank lines before and after code blocks
- Indent code properly within the code block

```markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
```

✅ CORRECT:
````markdown
```rust
let x = 5;
```
````

❌ INCORRECT:
````markdown
```rust
let x = 5;
```rust
````

### Supported Language Identifiers

| Language | Identifier |
|----------|------------|
| Rust | `rust` |
| Bash/Shell | `bash` or `sh` |
| JSON | `json` |
| YAML | `yaml` |
| SQL | `sql` |
| HTML | `html` |
| JavaScript | `js` |
| TypeScript | `ts` |
| CSS | `css` |
| Plain text | (none) |

## Tables

- Use pipe tables with header row and separator row
- Align pipe characters vertically for readability
- Use at least 3 dashes for separator row
- Use a colon (`:`) in the separator row to align columns

```markdown
| Header 1 | Header 2 | Header 3 |
|----------|:--------:|----------:|
| Left     | Center   | Right     |
| Cell     | Cell     | Cell      |
```

## Blockquotes

- Use `>` for blockquotes
- Include a space after the `>`
- Use nested blockquotes with multiple `>` characters

```markdown
> This is a blockquote
>
> > This is a nested blockquote
```

## Horizontal Rules

- Use three hyphens (`---`) for horizontal rules
- Leave blank lines before and after

```markdown
Content above

---

Content below
```

## Frontmatter

Every documentation file should include YAML frontmatter at the beginning:

```markdown
---
title: "Document Title"
description: "Brief description of the document"
category: "Category"
tags: ["tag1", "tag2", "tag3"]
last_updated: "YYYY-MM-DD"
version: "1.0"
---
```

Required frontmatter fields:
- `title`: The document title
- `description`: Brief description (1-2 sentences)
- `category`: Main category
- `tags`: Related keywords as an array
- `last_updated`: Date in format "Month DD, YYYY"
- `version`: Document version

## Accessibility Considerations

- Write descriptive alt text for images
- Use proper heading hierarchy
- Ensure sufficient color contrast in diagrams
- Provide text alternatives for complex visuals
- Use descriptive link text
- Create accessible tables with proper headers

## Document Structure

Standard document structure:

1. Frontmatter
2. Title (H1)
3. Introduction/Overview
4. Table of Contents (for longer documents)
5. Main content with hierarchical headings
6. Related Documents section (if applicable)

## Related Documents

- [Documentation Standards](../05_reference/standards/documentation-standards.md)
- [Contributing Guidelines](contributing-guidelines.md)
- [Code Style Guide](code-style-guide.md) 
