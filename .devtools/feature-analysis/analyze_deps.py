#!/usr/bin/env python3
"""
Dependency Analysis Tool for Feature Flag Optimization

This script analyzes Cargo.toml to identify:
1. Which dependencies are used by which features
2. Dependencies that could be made optional
3. Feature interdependencies
"""

import json
import subprocess
import re
import os
from collections import defaultdict

def run_cargo_metadata():
    """Get detailed cargo metadata in JSON format"""
    try:
        result = subprocess.run(
            ["cargo", "metadata", "--format-version=1"],
            capture_output=True, text=True, check=True
        )
        return json.loads(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Error running cargo metadata: {e}")
        return None
    except json.JSONDecodeError as e:
        print(f"Error parsing cargo metadata: {e}")
        return None

def extract_dependency_information(metadata):
    """Extract dependency and feature information from cargo metadata"""
    # Find the root package
    root_package = None
    for package in metadata["packages"]:
        if package["id"] in metadata["workspace_members"]:
            root_package = package
            break
    
    if not root_package:
        print("Root package not found!")
        return None
    
    # Get features
    features = root_package["features"]
    
    # Get dependencies
    dependencies = []
    for dep in root_package["dependencies"]:
        dependency = {
            "name": dep["name"],
            "optional": dep.get("optional", False),
            "features": dep.get("features", []),
            "kind": dep.get("kind", "normal"),
        }
        dependencies.append(dependency)
    
    return {
        "package_name": root_package["name"],
        "features": features,
        "dependencies": dependencies
    }

def calculate_dependency_usage(data):
    """Calculate which features use which dependencies"""
    # Direct dependencies of each feature
    feature_deps = {}
    for feature_name, feature_items in data["features"].items():
        deps = []
        for item in feature_items:
            if item.startswith("dep:"):
                deps.append(item[4:])  # Remove 'dep:' prefix
        feature_deps[feature_name] = deps
    
    # Resolve transitive dependencies
    resolved_deps = defaultdict(set)
    
    def resolve_feature(feature_name, seen=None):
        if seen is None:
            seen = set()
        
        if feature_name in seen:
            return set()  # Circular dependency
        
        seen.add(feature_name)
        
        if feature_name not in data["features"]:
            return set()
            
        direct_deps = set(feature_deps.get(feature_name, []))
        indirect_deps = set()
        
        for item in data["features"][feature_name]:
            if not item.startswith("dep:") and item in data["features"]:
                indirect_deps.update(resolve_feature(item, seen.copy()))
        
        return direct_deps.union(indirect_deps)
    
    # Calculate complete dependency sets
    for feature in data["features"]:
        resolved_deps[feature] = resolve_feature(feature)
    
    return resolved_deps

def generate_dependency_matrix(data, resolved_deps):
    """Generate a markdown table showing which dependencies are used by which features"""
    all_features = sorted(data["features"].keys())
    
    # Get all dependencies and categorize them
    all_deps = []
    optional_deps = set()
    dev_deps = set()
    build_deps = set()
    
    for dep in data["dependencies"]:
        all_deps.append(dep["name"])
        if dep["optional"]:
            optional_deps.add(dep["name"])
        if dep["kind"] == "dev":
            dev_deps.add(dep["name"])
        if dep["kind"] == "build":
            build_deps.add(dep["name"])
    
    all_deps = sorted(all_deps)
    
    # Create the matrix table
    table = ["# Dependency to Feature Matrix\n"]
    table.append("| Dependency | " + " | ".join(all_features) + " |")
    table.append("|" + "-" * 20 + "|" + "".join(["-" * 12 + "|" for _ in all_features]))
    
    for dep_name in all_deps:
        # Skip dev dependencies for clarity
        if dep_name in dev_deps:
            continue
            
        row = f"| {dep_name} "
        
        if dep_name in optional_deps:
            # Optional dependency
            for feature in all_features:
                if dep_name in resolved_deps[feature]:
                    row += "| ✅ "
                else:
                    row += "| ❌ "
        else:
            # Non-optional dependency
            for feature in all_features:
                if dep_name in resolved_deps[feature]:
                    row += "| ✅ "
                else:
                    row += "| ⚪ "
        
        row += "|"
        table.append(row)
    
    table.append("\nLegend:")
    table.append("- ✅: Required by feature")
    table.append("- ❌: Optional dependency not included in feature")
    table.append("- ⚪: Always included (non-optional dependency)")
    
    return "\n".join(table)

def identify_optimization_candidates(data, resolved_deps):
    """Identify dependencies that could be made optional"""
    # Dependencies used by some features but not all
    all_features = set(data["features"].keys())
    candidates = []
    
    for dep in data["dependencies"]:
        # Skip dependencies that are already optional or dev dependencies
        if dep["optional"] or dep["kind"] != "normal":
            continue
            
        # Find which features use this dependency
        used_by_features = []
        for feature, deps in resolved_deps.items():
            if dep["name"] in deps:
                used_by_features.append(feature)
        
        # If used by some features but not all, it's a candidate
        if 0 < len(used_by_features) < len(all_features):
            candidates.append({
                "name": dep["name"],
                "used_by": sorted(used_by_features),
            })
    
    return candidates

def main():
    # Get cargo metadata
    metadata = run_cargo_metadata()
    if not metadata:
        return
    
    # Extract dependency information
    data = extract_dependency_information(metadata)
    if not data:
        return
    
    # Calculate dependency usage
    resolved_deps = calculate_dependency_usage(data)
    
    # Generate dependency matrix
    matrix = generate_dependency_matrix(data, resolved_deps)
    
    # Identify optimization candidates
    candidates = identify_optimization_candidates(data, resolved_deps)
    
    # Output results
    print(matrix)
    
    print("\n\n# Optimization Candidates\n")
    print("The following dependencies could be made optional:")
    
    for candidate in candidates:
        print(f"- **{candidate['name']}**: Used by {', '.join(candidate['used_by'])}")
    
    # Write results to a file
    with open("dependency_analysis.md", "w") as f:
        f.write(matrix)
        f.write("\n\n# Optimization Candidates\n\n")
        f.write("The following dependencies could be made optional:\n\n")
        
        for candidate in candidates:
            f.write(f"- **{candidate['name']}**: Used by {', '.join(candidate['used_by'])}\n")
    
    print("\nAnalysis complete. Results saved to dependency_analysis.md")

if __name__ == "__main__":
    main() 