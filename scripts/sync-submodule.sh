#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

# Get project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SUBMODULE_PATH="libs/soundcloud-rs"

echo "=== Syncing Submodule $SUBMODULE_PATH ==="

# 1. Check if the submodule directory exists
if [ ! -d "$PROJECT_ROOT/$SUBMODULE_PATH" ]; then
    echo "Error: Submodule directory not found at $SUBMODULE_PATH"
    exit 1
fi

cd "$PROJECT_ROOT/$SUBMODULE_PATH"

# 2. Check if there are changes in the submodule
if [ -n "$(git status --porcelain)" ]; then
    echo "Found uncommitted changes in $SUBMODULE_PATH. Committing..."
    
    # Check if we are in a detached HEAD state in the submodule
    CURRENT_BRANCH=$(git symbolic-ref --short -q HEAD || true)
    if [ -z "$CURRENT_BRANCH" ]; then
        echo "Warning: Submodule is in a detached HEAD state. Checking out 'main' branch..."
        git checkout main || git checkout master || echo "Could not checkout branch, committing on detached HEAD..."
    fi
    
    git add -A
    git commit -m "feat: sync local changes to soundcloud-rs"
    echo "Successfully committed changes inside the submodule."
else
    echo "No uncommitted changes in $SUBMODULE_PATH."
fi

# Push changes in the submodule to remote (converting HTTPS URL to SSH if needed)
REMOTE_URL=$(git remote get-url origin 2>/dev/null || true)
if [[ "$REMOTE_URL" =~ ^https://github.com/([^/]+)/([^/]+)$ || "$REMOTE_URL" =~ ^https://github.com/([^/]+)/([^/]+)\.git$ ]]; then
    SSH_URL="git@github.com:${BASH_REMATCH[1]}/${BASH_REMATCH[2]}"
    if [[ ! "$SSH_URL" =~ \.git$ ]]; then
        SSH_URL="${SSH_URL}.git"
    fi
else
    SSH_URL="$REMOTE_URL"
fi

echo "Pushing changes in $SUBMODULE_PATH to remote using SSH ($SSH_URL)..."
git push "$SSH_URL" HEAD || echo "Warning: Failed to push to remote. You might need to push manually."

# Get the latest commit hash from the submodule
SUBMODULE_COMMIT=$(git rev-parse --short HEAD)

cd "$PROJECT_ROOT"

# 3. Check if the main repo sees the submodule pointer change
# (Using git status --porcelain to check for any changes in the submodule folder pointer)
if git status --porcelain "$SUBMODULE_PATH" | grep -q "^ M"; then
    echo "Updating submodule pointer in main repository..."
    git add "$SUBMODULE_PATH"
    git commit -m "chore: update $SUBMODULE_PATH pointer to $SUBMODULE_COMMIT"
    echo "Successfully committed submodule pointer update in main repository."
else
    echo "Submodule pointer is already up to date in the main repository."
fi

echo "=== Submodule sync completed successfully! ==="
