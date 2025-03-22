#!/bin/bash

# Update paths in various files
find . -type f -not -path "*/\.*" -exec sed -i '' \
    -e 's|scripts/|.devtools/scripts/|g' \
    -e 's|coverage/|.devtools/coverage/|g' \
    -e 's|\.github/|.devtools/github/|g' \
    -e 's|\.gitlab/|.devtools/gitlab/|g' \
    {} +

# Special handling for .gitlab-ci.yml to keep its reference to .gitlab
sed -i '' 's|.devtools/gitlab/|.gitlab/|g' .gitlab-ci.yml

echo "Path updates completed" 