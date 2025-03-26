# Available Instructions

Cursor rules are user provided instructions for the AI to follow to help work with the codebase.
They may or may not be relevent to the task at hand. If they are, use the fetch_rules tool to fetch the full rule.
Some rules may be automatically attached to the conversation if the user attaches a file that matches the rule's glob, and wont need to be fetched.

001-workflow: Before starting work on any task Check this rule.
020-roadmaps: When working with files in the docs/roadmaps folder.
021-error-tracking: This rule helps track attempted error fixes during the codebase cleanup process. Before attempting to fix the same error multiple times, consult this document to avoid repeating unsuccessful approaches.
022-error-handling: Guidelines for error handling in the Navius application
023-mod-rules: when dealing with modules, module exports, lib.rs, and mod.rs files
024-dependency-management: Guidelines for managing Rust dependencies and crates
025-test-coverage: When working with test coverage
026-test-examples: when creating tests use this file for reference
027-testing-guidance: if you are making, editing, or dealing with test issues reference this
040-no-legacy-code: Rules for handling legacy code and ensuring clean refactoring
999-date-formats: If writing any dates, timestamps or similar reference this rule.
999-mdc-format: Use when user asks to create or update *.mdc files
api-standards: Standards and best practices for API development 