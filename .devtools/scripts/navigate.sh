#!/bin/bash
# navigate.sh - Helper script for navigating the Navius codebase
# Usage: navigate.sh [component|path|find] [query]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print usage information
usage() {
  echo -e "${GREEN}Navius Code Navigation Helper${NC}"
  echo -e "Usage: $0 [command] [query]"
  echo -e "\nCommands:"
  echo -e "  ${CYAN}component${NC} [name]     Find core components (api, auth, cache, etc.)"
  echo -e "  ${CYAN}path${NC} [path_fragment] Find files with a path fragment"
  echo -e "  ${CYAN}find${NC} [pattern]       Find files containing a pattern"
  echo -e "  ${CYAN}flow${NC} [endpoint]      Show the request flow for an endpoint"
  echo -e "  ${CYAN}help${NC}                 Show this help message"
  echo -e "\nExamples:"
  echo -e "  $0 component auth       # Show all files in the auth component"
  echo -e "  $0 path router          # Find files with 'router' in the path"
  echo -e "  $0 find 'fn create_user' # Find files containing the pattern"
  echo -e "  $0 flow 'GET /users'    # Show the request flow for the GET /users endpoint"
}

# Find core components
find_component() {
  if [ -z "$1" ]; then
    echo -e "${RED}Error: No component name provided${NC}"
    echo -e "Available components: api, auth, cache, config, database, error, metrics, reliability, repository, router, services, utils"
    return 1
  fi

  component=$1
  echo -e "${GREEN}Files in the $component component:${NC}"
  
  if [ -d "src/core/$component" ]; then
    find "src/core/$component" -type f -name "*.rs" | sort
  else
    echo -e "${YELLOW}Component not found in src/core/$component${NC}"
    
    # Try to find it elsewhere
    echo -e "\n${GREEN}Searching for '$component' in other locations:${NC}"
    find src -path "*/$component/*.rs" -o -path "*/$component.rs" | sort
  fi
}

# Find files with a path fragment
find_path() {
  if [ -z "$1" ]; then
    echo -e "${RED}Error: No path fragment provided${NC}"
    return 1
  fi

  fragment=$1
  echo -e "${GREEN}Files with '$fragment' in the path:${NC}"
  find src -path "*$fragment*" -name "*.rs" | sort
}

# Find files containing a pattern
find_pattern() {
  if [ -z "$1" ]; then
    echo -e "${RED}Error: No pattern provided${NC}"
    return 1
  fi

  pattern=$1
  echo -e "${GREEN}Files containing '$pattern':${NC}"
  grep -r --include="*.rs" --color=always "$pattern" src || echo -e "${YELLOW}No matches found${NC}"
}

# Show the request flow for an endpoint
show_flow() {
  if [ -z "$1" ]; then
    echo -e "${RED}Error: No endpoint provided${NC}"
    return 1
  fi

  endpoint=$1
  echo -e "${GREEN}Request flow for endpoint '$endpoint':${NC}"
  
  # Find route definition
  echo -e "\n${CYAN}1. Route Definition:${NC}"
  grep -r --include="*.rs" --color=always -A 3 -B 3 "$endpoint" src/core/router src/app/router || echo -e "${YELLOW}Route not found${NC}"
  
  # Try to extract the handler name from the route definition
  handler=$(grep -r --include="*.rs" "$endpoint" src/core/router src/app/router | grep -o -E "handler\s*=\s*[a-zA-Z0-9_:]+|[a-zA-Z0-9_:]+::handler" | sed -E 's/handler\s*=\s*//g' | sed -E 's/::handler//g')
  
  if [ ! -z "$handler" ]; then
    echo -e "\n${CYAN}2. Handler Implementation:${NC}"
    grep -r --include="*.rs" --color=always -A 10 -B 3 "fn $handler" src || 
    grep -r --include="*.rs" --color=always -A 10 -B 3 "$handler" src || 
    echo -e "${YELLOW}Handler not found${NC}"

    # Try to extract service calls from the handler
    service_calls=$(grep -r --include="*.rs" -A 20 "fn $handler" src | grep -o -E "[a-zA-Z0-9_]+_service\.[a-zA-Z0-9_]+" || echo "")
    
    if [ ! -z "$service_calls" ]; then
      echo -e "\n${CYAN}3. Service Implementation:${NC}"
      for service_call in $service_calls; do
        service_method=$(echo $service_call | cut -d'.' -f2)
        echo -e "\n${MAGENTA}Service method: $service_method${NC}"
        grep -r --include="*.rs" --color=always -A 10 -B 3 "fn $service_method" src/core/services src/app/services || echo -e "${YELLOW}Service method not found${NC}"
      done
    fi
  fi
}

# Main logic
if [ "$1" == "help" ] || [ -z "$1" ]; then
  usage
  exit 0
fi

command=$1
query=$2

case $command in
  component)
    find_component "$query"
    ;;
  path)
    find_path "$query"
    ;;
  find)
    find_pattern "$query"
    ;;
  flow)
    show_flow "$query"
    ;;
  *)
    echo -e "${RED}Error: Unknown command '$command'${NC}"
    usage
    exit 1
    ;;
esac 