#!/bin/bash
set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Starting sync...${NC}"

CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo -e "\033[0;31m❌ Error: You are on branch '$CURRENT_BRANCH'. Use main.\033[0m"
    exit 1
fi

echo -e "${BLUE}Formatting Rust...${NC}"
cargo fmt
cargo clippy --fix --allow-dirty --allow-staged -- -D warnings || echo "Clippy found issues."

if [ -d "frontend" ]; then
    echo -e "${BLUE}Formatting Frontend...${NC}"
    cd frontend
    yarn lint --fix || echo "Frontend lint found issues."
    cd ..
fi

echo -e "${BLUE}Committing...${NC}"
if [[ -z $(git status -s) ]]; then
    echo -e "${GREEN}No changes.${NC}"
    exit 0
fi

git add .
COMMIT_MSG=${1:-"chore: format and auto-fix"}
git commit -m "$COMMIT_MSG"

echo -e "${BLUE}Pushing...${NC}"
git push origin main

echo -e "${GREEN}Done.${NC}"
