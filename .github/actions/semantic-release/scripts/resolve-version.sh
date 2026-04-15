#!/usr/bin/env bash
set -euo pipefail

# Resolve version from semantic-release output or fallback to interim version
# Usage: resolve-version.sh <logs_file> <component_path> <tag_prefix> <sha_prefix> <sha_suffix> <github_sha>

LOGS_FILE="$1"
COMPONENT_PATH="$2"
TAG_PREFIX="$3"
SHA_PREFIX="$4"
SHA_SUFFIX="$5"
GITHUB_SHA="$6"

# Check if NEXT_RELEASE_VERSION was set by semantic-release exec plugin
if [[ -n "${NEXT_RELEASE_VERSION:-}" ]]; then
  echo "Version from semantic-release: $NEXT_RELEASE_VERSION" >&2
  echo "$NEXT_RELEASE_VERSION"
  exit 0
fi

# Try to extract version from semantic-release output

# Pattern 1: Look for "The next release version is X.Y.Z"
if grep -q "The next release version is" "$LOGS_FILE"; then
  VERSION=$(grep "The next release version is" "$LOGS_FILE" | sed -E 's/.*The next release version is ([0-9]+\.[0-9]+\.[0-9]+).*/\1/')
  echo "Predicted version (pattern 1): $VERSION" >&2
  echo "$VERSION"
  exit 0
fi

# Pattern 2: Look for "NEXT_RELEASE_VERSION=X.Y.Z" from exec plugin
# Match the actual output line (not the config), which should have only the variable assignment
if grep -qE "^NEXT_RELEASE_VERSION=[0-9]" "$LOGS_FILE"; then
  VERSION=$(grep -E "^NEXT_RELEASE_VERSION=[0-9]" "$LOGS_FILE" | head -n1 | sed -E 's/NEXT_RELEASE_VERSION=([0-9]+\.[0-9]+\.[0-9]+).*/\1/')
  echo "Predicted version (pattern 2): $VERSION" >&2
  echo "$VERSION"
  exit 0
fi

if grep -q "Published release" "$LOGS_FILE"; then
  # Actual release - extract published version
  VERSION=$(grep "Published release" "$LOGS_FILE" | sed -E 's/.*release ([0-9]+\.[0-9]+\.[0-9]+).*/\1/')
  echo "Published version: $VERSION" >&2
  echo "$VERSION"
  exit 0
fi

# Fallback to interim version with SHA
if [ -n "$COMPONENT_PATH" ]; then
  # Find latest tag for this specific component
  LATEST_TAG=$(git tag -l "${TAG_PREFIX}*" | sort -V | tail -n1 || echo "")
  if [ -z "$LATEST_TAG" ]; then
    # No tags found for this component, start at 0.1.0
    VERSION="0.1.0${SHA_PREFIX}${GITHUB_SHA:0:7}${SHA_SUFFIX}"
  else
    # Extract version number from tag by removing the prefix
    LATEST_VERSION="${LATEST_TAG#"$TAG_PREFIX"}"
    VERSION="${LATEST_VERSION}${SHA_PREFIX}${GITHUB_SHA:0:7}${SHA_SUFFIX}"
  fi
else
  # No component path, find latest global tag
  LATEST_TAG=$(git describe --tags "$(git rev-list --tags --max-count=1)" 2>/dev/null || echo "0.0.0")
  VERSION="${LATEST_TAG}${SHA_PREFIX}${GITHUB_SHA:0:7}${SHA_SUFFIX}"
fi

echo "Interim version: $VERSION" >&2
echo "$VERSION"
