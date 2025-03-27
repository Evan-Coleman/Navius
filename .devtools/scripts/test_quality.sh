#!/bin/bash

# Script to check test quality and detect common test smells

# Define color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "Analyzing test quality..."
echo ""

# Function to count occurrences in test files
count_occurrences() {
  local pattern=$1
  local files=$2
  grep -E "$pattern" $files | wc -l | xargs
}

# Find all test files
TEST_FILES=$(find . -name "*_tests.rs" -or -name "tests.rs" -or -path "./tests/*.rs")
TEST_COUNT=$(echo "$TEST_FILES" | wc -l | xargs)

echo "Found $TEST_COUNT test files"

# Check for common test smells
echo ""
echo "=== Test Smell Analysis ==="

# 1. Check for assertion counts
TOTAL_ASSERTS=$(count_occurrences "assert|expect" "$TEST_FILES")
TESTS_WITH_NO_ASSERT=$(grep -l "#\[test\]" $TEST_FILES | xargs grep -L "assert\|expect" | wc -l | xargs)

# 2. Check for magic values/strings
MAGIC_NUMBERS=$(count_occurrences "[^\"a-zA-Z_]([-0-9]+)[ ]*[,);]" "$TEST_FILES")

# 3. Check for commented out tests
COMMENTED_TESTS=$(count_occurrences "//[ ]*#\[test\]" "$TEST_FILES")

# 4. Check for sleeps in tests
SLEEPS_IN_TESTS=$(count_occurrences "std::thread::sleep|tokio::time::sleep" "$TEST_FILES")

# 5. Check for empty catch blocks
EMPTY_CATCH_BLOCKS=$(count_occurrences "Result<.*>.*unwrap\(\)" "$TEST_FILES")

# 6. Check for tests that may run too long
TESTS_WITHOUT_TIMEOUT=$(grep -l "#\[tokio::test\]" $TEST_FILES | xargs grep -L "timeout" | wc -l | xargs)

# Display results
echo "Assertion Usage:"
echo "- Total assertions: $TOTAL_ASSERTS"
if [ "$TESTS_WITH_NO_ASSERT" -gt 0 ]; then
  echo -e "${YELLOW}⚠ Tests without assertions: $TESTS_WITH_NO_ASSERT${NC}"
else
  echo -e "${GREEN}✓ All tests have assertions${NC}"
fi

echo ""
echo "Test Code Quality:"
if [ "$MAGIC_NUMBERS" -gt 50 ]; then
  echo -e "${YELLOW}⚠ Many magic numbers/values: $MAGIC_NUMBERS (consider using constants)${NC}"
else
  echo -e "${GREEN}✓ Magic numbers/values: $MAGIC_NUMBERS${NC}"
fi

if [ "$COMMENTED_TESTS" -gt 0 ]; then
  echo -e "${RED}✗ Commented out tests: $COMMENTED_TESTS (should be removed or fixed)${NC}"
else
  echo -e "${GREEN}✓ No commented out tests${NC}"
fi

if [ "$SLEEPS_IN_TESTS" -gt 0 ]; then
  echo -e "${YELLOW}⚠ Tests with sleep calls: $SLEEPS_IN_TESTS (consider using mocks instead)${NC}"
else
  echo -e "${GREEN}✓ No tests with sleep calls${NC}"
fi

if [ "$EMPTY_CATCH_BLOCKS" -gt 10 ]; then
  echo -e "${YELLOW}⚠ Many unwrap() calls: $EMPTY_CATCH_BLOCKS (consider proper error handling)${NC}"
else
  echo -e "${GREEN}✓ Unwrap() calls: $EMPTY_CATCH_BLOCKS${NC}"
fi

if [ "$TESTS_WITHOUT_TIMEOUT" -gt 0 ]; then
  echo -e "${YELLOW}⚠ Async tests without timeout: $TESTS_WITHOUT_TIMEOUT${NC}"
else
  echo -e "${GREEN}✓ All async tests have timeouts${NC}"
fi

echo ""
echo "=== Recommendations ==="

if [ "$TESTS_WITH_NO_ASSERT" -gt 0 ]; then
  echo "- Add assertions to tests that don't have them"
fi

if [ "$MAGIC_NUMBERS" -gt 50 ]; then
  echo "- Replace magic numbers with named constants"
fi

if [ "$COMMENTED_TESTS" -gt 0 ]; then
  echo "- Uncomment and fix tests or remove commented test code"
fi

if [ "$SLEEPS_IN_TESTS" -gt 0 ]; then
  echo "- Replace sleep calls with proper mocks or test utilities"
fi

if [ "$EMPTY_CATCH_BLOCKS" -gt 10 ]; then
  echo "- Add proper error handling instead of unwrap()"
fi

if [ "$TESTS_WITHOUT_TIMEOUT" -gt 0 ]; then
  echo "- Add timeouts to async tests to prevent hanging"
fi

# Final output
if [ "$TESTS_WITH_NO_ASSERT" -eq 0 ] && [ "$COMMENTED_TESTS" -eq 0 ] && [ "$SLEEPS_IN_TESTS" -eq 0 ]; then
  echo -e "\n${GREEN}Test quality is good!${NC}"
  exit 0
else
  echo -e "\n${YELLOW}Some test quality issues found. See recommendations above.${NC}"
  exit 1
fi 