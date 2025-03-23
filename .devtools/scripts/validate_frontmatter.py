#!/usr/bin/env python3

import os
import sys
import yaml
import datetime
from pathlib import Path

REQUIRED_FIELDS = ['title', 'description', 'category', 'tags', 'last_updated', 'version']

def validate_frontmatter(file_path):
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Check if file has frontmatter
    if not content.startswith('---'):
        print(f"Error: {file_path} is missing frontmatter")
        return False
    
    # Extract frontmatter
    try:
        frontmatter = content.split('---')[1]
        metadata = yaml.safe_load(frontmatter)
    except:
        print(f"Error: {file_path} has invalid frontmatter format")
        return False
    
    # Check required fields
    missing_fields = [field for field in REQUIRED_FIELDS if field not in metadata]
    if missing_fields:
        print(f"Error: {file_path} is missing required fields: {', '.join(missing_fields)}")
        return False
    
    # Validate date format
    try:
        datetime.datetime.strptime(metadata['last_updated'], '%B %d, %Y')
    except ValueError:
        print(f"Error: {file_path} has invalid date format. Use 'Month DD, YYYY' format")
        return False
    
    # Validate version format
    if not isinstance(metadata['version'], (int, float)):
        print(f"Error: {file_path} has invalid version format. Use numeric format")
        return False
    
    # Validate tags
    if not isinstance(metadata['tags'], list):
        print(f"Error: {file_path} tags must be a list")
        return False
    
    return True

def main():
    docs_dir = Path('docs')
    success = True
    
    for md_file in docs_dir.rglob('*.md'):
        if not validate_frontmatter(md_file):
            success = False
    
    sys.exit(0 if success else 1)

if __name__ == '__main__':
    main() 