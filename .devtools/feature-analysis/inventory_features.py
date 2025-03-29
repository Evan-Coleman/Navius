#!/usr/bin/env python3
"""
Feature Flag Inventory Tool

This script analyzes the codebase to:
1. Identify modules and files using feature flags
2. Map code locations to specific features
3. Generate a feature usage report
"""

import os
import re
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

# Feature mapping patterns
FEATURE_PATTERNS = {
    # Pattern for cfg attributes
    'cfg_attributes': re.compile(r'#\[cfg\((.*?)\)\]', re.DOTALL),
    
    # Pattern for feature usage with if/cfg
    'cfg_if': re.compile(r'(?:feature\s*=\s*"([^"]+)")'),
    
    # Pattern for using dep: syntax in Cargo.toml
    'dep_syntax': re.compile(r'dep:([a-zA-Z0-9_-]+)'),
    
    # Pattern for feature blocks in Cargo.toml
    'feature_block': re.compile(r'(?:^|\n)\s*\[features\]\s*\n(.*?)(?=\n\s*\[|\Z)', re.DOTALL),
    
    # Pattern for conditional compilation in code
    'cfg_condition': re.compile(r'#\[cfg\(.*?feature\s*=\s*"([^"]+)".*?\)\]', re.DOTALL),
    
    # Pattern for cfg_if usage
    'cfg_if_let': re.compile(r'cfg_if::cfg_if!\s*\{\s*if\s*#\[cfg\(.*?feature\s*=\s*"([^"]+)".*?\)\]', re.DOTALL),
}

def find_rust_files(base_dir):
    """Find all Rust source files in the codebase"""
    rust_files = []
    
    print(f"Searching for Rust files in {base_dir}...")
    
    for root, _, files in os.walk(base_dir):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    print(f"Found {len(rust_files)} Rust files.")
    return rust_files

def extract_cargo_features(cargo_toml_path):
    """Extract feature definitions from Cargo.toml"""
    try:
        print(f"Extracting features from {cargo_toml_path}...")
        
        if not os.path.exists(cargo_toml_path):
            print(f"ERROR: Cargo.toml not found at {cargo_toml_path}")
            return {}
            
        with open(cargo_toml_path, 'r') as f:
            content = f.read()
            
        features = {}
        feature_match = FEATURE_PATTERNS['feature_block'].search(content)
        
        if feature_match:
            feature_block = feature_match.group(1)
            
            # Extract simple features (feature = [])
            for line in feature_block.split('\n'):
                line = line.strip()
                if '=' in line:
                    key, value = [x.strip() for x in line.split('=', 1)]
                    # Normalize the value
                    if value.startswith('[') and value.endswith(']'):
                        # It's an array
                        raw_deps = value[1:-1].strip()
                        if raw_deps:
                            deps = [d.strip().strip('"') for d in raw_deps.split(',')]
                            features[key] = deps
                        else:
                            features[key] = []
                    else:
                        # It's a single value
                        features[key] = [value.strip('"')]
        
        print(f"Extracted {len(features)} features.")
        return features
    except Exception as e:
        print(f"ERROR parsing Cargo.toml: {e}")
        return {}

def analyze_file_features(file_path):
    """Analyze a file for feature flag usage"""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        features = set()
        
        # Direct feature attributes (most common case)
        for match in FEATURE_PATTERNS['cfg_condition'].finditer(content):
            feature_name = match.group(1)
            features.add(feature_name)
        
        # Check for cfg attributes with multiple features (any, all, etc.)
        for match in FEATURE_PATTERNS['cfg_attributes'].finditer(content):
            attrs = match.group(1)
            # Extract feature names
            for feature_match in FEATURE_PATTERNS['cfg_if'].finditer(attrs):
                features.add(feature_match.group(1))
        
        # Check for cfg_if macros
        for match in FEATURE_PATTERNS['cfg_if_let'].finditer(content):
            feature_name = match.group(1)
            features.add(feature_name)
        
        return list(features)
    except Exception as e:
        print(f"ERROR analyzing {file_path}: {e}")
        return []

def generate_feature_inventory(base_dir):
    """Generate inventory of feature usage across the codebase"""
    cargo_toml_path = os.path.join(base_dir, 'Cargo.toml')
    
    # Extract defined features
    features = extract_cargo_features(cargo_toml_path)
    
    # Find all Rust files
    rust_files = find_rust_files(base_dir)
    
    # Analyze each file
    feature_usage = defaultdict(list)
    
    print("Analyzing feature usage in files...")
    file_count = 0
    files_with_features = 0
    
    for file in rust_files:
        file_features = analyze_file_features(file)
        rel_path = os.path.relpath(file, base_dir)
        
        if file_features:
            files_with_features += 1
            
        for feature in file_features:
            feature_usage[feature].append(rel_path)
            
        file_count += 1
        if file_count % 100 == 0:
            print(f"Processed {file_count} files...")
    
    # Generate report data
    report = {
        'defined_features': features,
        'feature_usage': {k: sorted(v) for k, v in feature_usage.items()},
        'stats': {
            'total_files': file_count,
            'files_with_features': files_with_features
        }
    }
    
    print(f"Found {len(feature_usage)} features used across {files_with_features} files.")
    return report

def main():
    try:
        # Use the current working directory as the base
        base_dir = os.getcwd()
        
        print(f"Starting feature usage analysis in {base_dir}...")
        
        # Generate feature inventory
        inventory = generate_feature_inventory(base_dir)
        
        # Create output directory
        report_dir = os.path.join(base_dir, '.devtools/feature-analysis/report')
        os.makedirs(report_dir, exist_ok=True)
        
        # Save report as JSON
        json_path = os.path.join(report_dir, 'feature_inventory.json')
        with open(json_path, 'w') as f:
            json.dump(inventory, f, indent=2)
        
        print(f"Saved JSON report to {json_path}")
        
        # Generate markdown report
        markdown_path = os.path.join(report_dir, 'feature_inventory.md')
        
        with open(markdown_path, 'w') as f:
            f.write("# Feature Flag Inventory\n\n")
            
            # Write summary stats
            f.write("## Summary\n\n")
            f.write(f"- Total Rust files analyzed: {inventory['stats']['total_files']}\n")
            f.write(f"- Files with feature flags: {inventory['stats']['files_with_features']}\n")
            f.write(f"- Features defined in Cargo.toml: {len(inventory['defined_features'])}\n")
            f.write(f"- Features used in codebase: {len(inventory['feature_usage'])}\n\n")
            
            # Write defined features
            f.write("## Defined Features\n\n")
            for feature, deps in inventory['defined_features'].items():
                f.write(f"### {feature}\n\n")
                if deps:
                    f.write("Dependencies:\n")
                    for dep in deps:
                        f.write(f"- {dep}\n")
                else:
                    f.write("No dependencies.\n")
                
                # Check for usage
                if feature in inventory['feature_usage']:
                    f.write(f"\nUsed in {len(inventory['feature_usage'][feature])} files.\n")
                else:
                    f.write("\nNot used directly in code.\n")
                f.write("\n")
            
            # Write feature usage
            f.write("## Feature Usage\n\n")
            if inventory['feature_usage']:
                for feature, files in inventory['feature_usage'].items():
                    f.write(f"### {feature}\n\n")
                    
                    if feature in inventory['defined_features']:
                        f.write("Status: ✓ Defined in Cargo.toml\n\n")
                    else:
                        f.write("Status: ❌ Not defined in Cargo.toml\n\n")
                    
                    f.write(f"Used in {len(files)} files:\n")
                    for file in files:
                        f.write(f"- `{file}`\n")
                    f.write("\n")
            else:
                f.write("No feature usage detected in the codebase.\n\n")
            
            # Write recommendations
            f.write("## Recommendations\n\n")
            
            # Find features defined but not used
            unused_features = [f for f in inventory['defined_features'] if f not in inventory['feature_usage']]
            if unused_features:
                f.write("### Unused Features\n\n")
                f.write("The following features are defined but not directly used in code (they might be used indirectly through dependencies):\n\n")
                for feature in unused_features:
                    f.write(f"- `{feature}`\n")
                f.write("\n")
            
            # Find features used but not defined
            undefined_features = [f for f in inventory['feature_usage'] if f not in inventory['defined_features']]
            if undefined_features:
                f.write("### Undefined Features\n\n")
                f.write("The following features are used in code but not defined in Cargo.toml:\n\n")
                for feature in undefined_features:
                    f.write(f"- `{feature}`\n")
                f.write("\n")
        
        print(f"Saved Markdown report to {markdown_path}")
    except Exception as e:
        print(f"ERROR: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 