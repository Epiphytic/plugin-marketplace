#!/usr/bin/env bash
set -euo pipefail

# Create .releaserc.json from template
# Usage: create-releaserc.sh <template_path> <branch> <tag_prefix> <is_dry_run> <component_path> <action_path>

TEMPLATE_PATH="$1"
BRANCH="$2"
TAG_PREFIX="$3"
IS_DRY_RUN="${4:-false}"
COMPONENT_PATH="${5:-}"
ACTION_PATH="${6:-}"

if [ -f .releaserc.json ]; then
  echo ".releaserc.json already exists, skip default .releaserc.json creation!"
  exit 0
fi

# Copy template and replace placeholders
cp "$TEMPLATE_PATH" .releaserc.json

# Replace branch placeholder
# Use | as delimiter to handle slashes in branch names (e.g., refs/pull/19/merge)
sed -i "s|BRANCH_PLACEHOLDER|$BRANCH|g" .releaserc.json

if [ "$IS_DRY_RUN" = "true" ]; then
  echo "Configured for dry-run mode: will calculate next version for $BRANCH"
else
  echo "Configured for release mode: only releasing from $BRANCH"
fi

# Replace tag prefix placeholder
# Use different delimiter to avoid conflicts with slashes in paths
sed -i "s#TAG_PREFIX_PLACEHOLDER#${TAG_PREFIX}#g" .releaserc.json

# Replace component path placeholder (empty string if not provided)
sed -i "s#COMPONENT_PATH_PLACEHOLDER#${COMPONENT_PATH}#g" .releaserc.json

# Replace action path placeholder
sed -i "s#ACTION_PATH_PLACEHOLDER#${ACTION_PATH}#g" .releaserc.json

if [ -n "$COMPONENT_PATH" ]; then
  echo "Created .releaserc.json with tagFormat=${TAG_PREFIX}\${version} and component path filtering: ${COMPONENT_PATH}"
else
  echo "Created .releaserc.json with tagFormat=${TAG_PREFIX}\${version} (no component path filtering)"
fi
