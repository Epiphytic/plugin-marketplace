#!/usr/bin/env bash
set -euo pipefail

# Create initial version tag if needed
# Usage: create-initial-tag.sh <tag_prefix> <initial_version> <component_path>

TAG_PREFIX="$1"
INITIAL_VERSION="${2:-0.1.0}"
COMPONENT_PATH="${3:-}"

# Check if there are any existing releases for this component
if ! git tag -l "${TAG_PREFIX}*" | grep -q .; then
  echo "No existing releases found for ${TAG_PREFIX}, creating temporary tag for initial version calculation"

  # Parse the initial version to get the previous version
  # For example, if initial version is 0.1.0, create a 0.0.0 tag
  IFS='.' read -r MAJOR MINOR PATCH <<< "$INITIAL_VERSION"
  PREV_MINOR=$((MINOR - 1))
  if [ $PREV_MINOR -lt 0 ]; then
    PREV_MINOR=0
  fi
  PREV_VERSION="${MAJOR}.${PREV_MINOR}.${PATCH}"

  echo "Creating temporary tag ${TAG_PREFIX}${PREV_VERSION} to set initial version base"

  # Find the commit to tag
  if [ -n "$COMPONENT_PATH" ]; then
    # Map component path to actual git paths (same logic as filter-commits-by-path.js)
    GIT_PATHS=""

    if [[ "$COMPONENT_PATH" == actions/* ]]; then
      # For actions/foo, check both .github/actions/foo and actions/foo
      ACTION_NAME="${COMPONENT_PATH#actions/}"
      GIT_PATHS=".github/actions/${ACTION_NAME} ${COMPONENT_PATH}"
    elif [[ "$COMPONENT_PATH" == workflows/* ]]; then
      # For workflows/foo, check .github/workflows/foo.yml and workflows/foo
      WORKFLOW_NAME="${COMPONENT_PATH#workflows/}"
      GIT_PATHS=".github/workflows/${WORKFLOW_NAME}.yml ${COMPONENT_PATH}"
    else
      # For other paths, use as-is
      GIT_PATHS="$COMPONENT_PATH"
    fi

    # Find the first commit that touches any of these paths
    # Use head -1 with || true to avoid SIGPIPE (exit 141) when pipe closes early
    FIRST_COMPONENT_COMMIT=$(git log --reverse --format=%H -- $GIT_PATHS | head -1 || true)

    if [ -n "$FIRST_COMPONENT_COMMIT" ]; then
      # Tag the commit BEFORE the first component commit (parent)
      PARENT_COMMIT=$(git rev-parse "${FIRST_COMPONENT_COMMIT}^" 2>/dev/null || echo "")

      if [ -n "$PARENT_COMMIT" ]; then
        echo "Tagging parent of first component commit: ${PARENT_COMMIT}"
        git tag "${TAG_PREFIX}${PREV_VERSION}" "$PARENT_COMMIT" || true
      else
        # If there's no parent (shouldn't happen), tag the first repo commit
        echo "No parent found, tagging first repo commit"
        FIRST_COMMIT=$(git rev-list --max-parents=0 HEAD)
        git tag "${TAG_PREFIX}${PREV_VERSION}" "$FIRST_COMMIT" || true
      fi
    else
      # No commits found for this component path, tag the latest commit
      echo "No commits found for component path, tagging HEAD"
      git tag "${TAG_PREFIX}${PREV_VERSION}" HEAD || true
    fi
  else
    # No component path, tag the first commit in the repo
    FIRST_COMMIT=$(git rev-list --max-parents=0 HEAD)
    git tag "${TAG_PREFIX}${PREV_VERSION}" "$FIRST_COMMIT" || true
  fi

  echo "Created temporary tag ${TAG_PREFIX}${PREV_VERSION}"
else
  echo "Existing releases found for ${TAG_PREFIX}, skipping initial tag creation"
fi
