#!/usr/bin/env bash
set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

if ! git diff-index --quiet HEAD --; then
    echo -e "${YELLOW}Warning: You have uncommitted changes. Please commit or stash them first.${NC}"
    exit 1
fi

CURRENT_VERSION=$(grep "^version =" Cargo.toml | head -n 1 | cut -d '"' -f 2)
echo -e "${BLUE}Current version:${NC} $CURRENT_VERSION"

IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"
SUGGESTED_VERSION="$major.$minor.$((patch + 1))"

read -p "Enter new version [$SUGGESTED_VERSION]: " NEW_VERSION
NEW_VERSION=${NEW_VERSION:-$SUGGESTED_VERSION}

echo -e "${BLUE}Updating to version:${NC} $NEW_VERSION"

sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
sed -i "0,/\"version\": \"$CURRENT_VERSION\"/s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" frontend/package.json

git add Cargo.toml frontend/package.json

git commit -m "chore: release v$NEW_VERSION"
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

echo -e "${GREEN}Version updated to v$NEW_VERSION and tagged locally.${NC}"

read -p "Do you want to push the commit and tag? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git push && git push --tags
    echo -e "${GREEN}Successfully pushed to remote.${NC}"
else
    echo -e "${YELLOW}Push skipped. Don't forget to 'git push && git push --tags' later.${NC}"
fi
