#!/bin/bash

# Simple wrapper script for the build process
# Just passes all arguments to the actual build script

exec .devtools/scripts/build.sh "$@" 