## Remove Example Code Script

The `remove_example_code.sh` script is used to remove all example code from the project. This is useful when preparing for production deployment.

### What it does:

1. Removes all files that start with `example_`
2. Removes all directories that start with `example_`
3. Removes all import statements for example modules
4. Cleans up empty lines left after removal

### Usage:

```bash
./.devtools/scripts/remove_example_code.sh
```

### When to use:

- When preparing the codebase for production
- When you've implemented your own versions of the example components
- When you want to clean up the codebase for distribution

### Note:

This operation is irreversible, so make sure you have a backup or commit your changes before running this script. 