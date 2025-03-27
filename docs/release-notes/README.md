# Release Notes

This section contains the release notes for each version of the Navius framework, detailing changes, new features, fixes, and migration guidance.

## Version History

- [Version 0.1.0](0.1.0.md) - Initial release with core functionality

## Release Strategy

Navius follows Semantic Versioning (SemVer) with the following format:

`MAJOR.MINOR.PATCH`

- **MAJOR** version increases for incompatible API changes
- **MINOR** version increases for backward-compatible functionality additions
- **PATCH** version increases for backward-compatible bug fixes

## Release Process

Each release undergoes the following process:

1. Feature freeze and creation of a release branch
2. Extensive testing and bug fixing
3. Documentation updates and release notes preparation
4. Final approval and signing
5. Publishing to crates.io and GitHub releases

## Supported Versions

| Version | Status | End of Support |
|---------|--------|----------------|
| 0.1.x   | Active | TBD            |

## Migration Guides

When migrating between versions, refer to the specific migration guides:

- [0.1.x to 0.2.x Migration Guide](migrations/0.1-to-0.2.md) (Coming soon)

## Release Schedule

Navius aims for a predictable release schedule:

- **PATCH** releases: As needed for critical bug fixes
- **MINOR** releases: Approximately every 2-3 months
- **MAJOR** releases: Approximately once per year

## Pre-release Versions

Pre-release versions are available for testing new features before official releases:

- **Alpha**: Early development, incomplete features, not for production
- **Beta**: Feature complete, testing phase, not recommended for production
- **RC** (Release Candidate): Final testing before release, conditionally suitable for production

## Reporting Issues

If you encounter issues with any release:

1. Check existing issues in the GitHub repository
2. Submit a new issue with detailed reproduction steps
3. Specify the exact version where the issue occurs
4. Include any relevant logs or error messages 