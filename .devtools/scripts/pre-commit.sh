#!/bin/bash

# Pre-commit hook to detect sensitive data
# Exit with non-zero status if sensitive data is found

echo "Running pre-commit hook to check for sensitive data..."

# Files to check (staged files only)
FILES=$(git diff --cached --name-only)

# Patterns to detect
PATTERNS=(
  "API[_-]KEY[=\"':]\S+"               # API keys
  "SECRET[_-]KEY[=\"':]\S+"            # Secret keys
  "password[=\"':]\S+"                 # Passwords
  "DATABASE_URL[=\"':].*@"             # Database URLs with credentials
  "AKIA[0-9A-Z]{16}"                   # AWS Access Key IDs
  "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}" # UUIDs (potential tokens)
  "-----BEGIN.*PRIVATE KEY-----"       # Private keys (simplified pattern)
  "redis://[^@]*@"                     # Redis URLs with auth
  "mongodb://[^@]*@"                   # MongoDB URLs with auth
  "postgres://[^@]*@"                  # PostgreSQL URLs with auth - causing false positives
  "mysql://[^@]*@"                     # MySQL URLs with auth
)

# Files to ignore
IGNORED_FILES=(
  "*.md"                               # Markdown documentation
  "*_test.rs"                          # Test files
  "*.test.ts"                          # Test files
  "*.lock"                             # Lock files
  "package-lock.json"                  # NPM lock file
  "Cargo.lock"                         # Cargo lock file
  "pre-commit.sh"                      # This script itself
  "setup-hooks.sh"                     # Hook setup script
  ".env.example"                       # Environment example file
)

# Replace the postgres pattern with a more precise one that won't have false positives
for i in "${!PATTERNS[@]}"; do
  if [[ ${PATTERNS[$i]} == "postgres://[^@]*@" ]]; then
    # This pattern requires "postgres://" followed by something that's not @ then an @
    # It will match real connection strings but not type declarations like sqlx::Postgres
    PATTERNS[$i]="postgres://[^@]+@"
  fi
done

# Flags
DETECTED=0
IGNORED_PATTERN="\.md$|_test\.rs$|\.test\.ts$|\.lock$|package-lock\.json$|Cargo\.lock$|pre-commit\.sh$|setup-hooks\.sh$|\.env\.example$"

# Foreground colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Check each staged file
for FILE in $FILES; do
  # Skip ignored files
  if [[ $FILE =~ $IGNORED_PATTERN ]]; then
    echo "Skipping sensitive data check for ignored file: $FILE"
    continue
  fi

  # Check each pattern against file content - only look at added or modified lines (not removed lines)
  for PATTERN in "${PATTERNS[@]}"; do
    # Get the diff but filter out lines that start with a minus (being removed)
    DIFF_CONTENT=$(git diff --cached --no-color "$FILE" | grep -v "^-" || true)
    MATCHES=$(echo "$DIFF_CONTENT" | grep -E "$PATTERN" || true)
    
    if [ ! -z "$MATCHES" ]; then
      if [ $DETECTED -eq 0 ]; then
        echo -e "${RED}SENSITIVE DATA DETECTED!${NC}"
        echo -e "The following files contain sensitive data:"
        DETECTED=1
      fi
      echo -e "${YELLOW}$FILE${NC} - Pattern: $PATTERN"
      echo "$MATCHES" | grep -E --color "$PATTERN"
      echo ""
    fi
  done
done

# If sensitive data detected, prevent commit
if [ $DETECTED -eq 1 ]; then
  echo -e "${RED}Commit blocked due to sensitive data.${NC}"
  echo -e "Please remove the sensitive data and try again."
  echo -e "If this is a false positive, you can bypass with ${YELLOW}git commit --no-verify${NC}"
  exit 1
else
  echo -e "${GREEN}No sensitive data detected.${NC}"
fi

# All checks passed
exit 0 