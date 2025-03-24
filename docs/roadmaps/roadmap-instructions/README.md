---
title: "Roadmap Implementation Instructions"
description: "Documentation about Roadmap Implementation Instructions"
category: roadmap
tags:
  - documentation
  - testing
last_updated: March 23, 2025
version: 1.0
---
# Roadmap Implementation Instructions

This directory contains detailed guides and instructions for implementing specific roadmaps in the Navius framework. These instructions provide step-by-step guidance, prompts, and examples for completing roadmap tasks effectively.

## Purpose

Roadmap instructions differ from general guides in several ways:
- They are specifically tied to roadmap implementation
- They contain specific AI prompts and commands
- They include detailed testing procedures for roadmap items
- They follow a consistent implementation pattern

## Available Instructions

| Roadmap | Instruction Document | Status |
|---------|----------------------|--------|
| Project Restructuring | [Project Restructuring Guide](project-restructuring-guide.md) | Complete |
| Documentation Overhaul | [Documentation Overhaul Guide](documentation-overhaul-guide.md) | Not Started |
| API Model Management | [API Model Management Guide](api-model-management-guide.md) | Not Started |

## How to Use Roadmap Instructions

1. **Reference the Roadmap First**: Always start by reviewing the corresponding roadmap document to understand the overall goals, success criteria, and implementation phases.

2. **Follow the Implementation Sequence**: Instructions are designed to be followed in order. Each step builds on previous steps.

3. **Execute Verification Steps**: After completing each step, perform the verification steps to ensure the implementation meets requirements.

4. **Update Progress**: After completing tasks, update the corresponding roadmap document to mark items as complete.

5. **Document Lessons Learned**: If you encounter challenges or discover better approaches, update the instructions for future reference.

## Creating New Instruction Documents

When creating new roadmap instruction documents, follow these guidelines:

1. **Use the Standardized Format**:
   ```markdown
   # [Roadmap Name] Implementation Guide
   
   ## Overview
   [Brief description of the implementation]
   
   ## Prerequisites
   [List of requirements before starting]
   
   ## Implementation Steps
   
   ### Step 1: [Step Name]
   
   #### Instructions
   [Detailed instructions for this step]
   
   #### Implementation Prompts
   ```
   [AI prompt to use for implementation]
   ```
   
   #### Verification
   [How to verify this step was completed correctly]
   ```

2. **Include Clear AI Prompts**: When the implementation involves using AI tools, provide specific, well-formatted prompts.

3. **Provide Verification Steps**: For each implementation step, include clear verification criteria to confirm success.

4. **Link to Resources**: Include references to related documentation, examples, or external resources.

5. **Keep Updated**: As implementation patterns evolve, update the instruction documents to reflect best practices.

## Relationship to Roadmaps

Each instruction document should:
- Reference the specific roadmap it implements
- Follow the same phase structure as the roadmap
- Use the same terminology and concepts
- Target the same success criteria

## Migration Notes

The following documents have been moved to this location from their previous locations:
- `docs/guides/project-restructuring-guide.md` → `docs/roadmaps/roadmap-instructions/project-restructuring-guide.md` 
