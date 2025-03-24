# Codebase Cleanup Prompt

I need help with the next phase of our codebase cleanup after the Pet API Database Integration. We're working on Roadmap #17.

## Current Progress
- Analyzed build errors (32) and test errors (~60)
- Created MockTokenClient implementation
- Set up SQLx cache generation script

## Next Priority Area
[PRIORITY_AREA]
- Remove users service (being replaced by petdb service)
- Tag all pet-related code as examples (for future removal script)
- Fix module structure and imports
- Fix type mismatches and lifetime issues
- Fix CacheRegistry issues
- Fix SQLx offline mode errors
- Fix test failures

## Requirements
- Follow error handling guidelines in error-handling.mdc
- Keep changes minimal but comprehensive
- Fix lowest-level errors first
- Update tests alongside code changes
- Remove all user service related code and tests
- Ensure petdb service is the only database service implementation
- Tag all example code (pet API) with @example comment tags
- Move all example code to /examples directories where possible
- Add @example_dependency tags to any dependencies only used by examples

Please help implement fixes for this area and update our tracking in the roadmap. 

Roadmap : /Users/goblin/dev/git/navius/docs/roadmaps/17-codebase-cleanup.md
Instructions : /Users/goblin/dev/git/navius/docs/roadmaps/roadmap-instructions/17-codebase-cleanup-instructions.md