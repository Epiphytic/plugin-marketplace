#!/usr/bin/env bash
set -euo pipefail

# Generate a release summary from semantic-release logs
# Usage: generate-summary.sh VERSION COMPONENT_PATH TAG_PREFIX BASE_BRANCH LOGS_FILE OUTPUT_FILE

VERSION="${1:-}"
COMPONENT_PATH="${2:-}"
TAG_PREFIX="${3:-v}"
BASE_BRANCH="${4:-main}"
LOGS_FILE="${5:-release-logs.txt}"
OUTPUT_FILE="${6:-release-summary.md}"

# Strip .github/ prefix if present (for tag compatibility)
COMPONENT_PATH="${COMPONENT_PATH#.github/}"

# Build full tag
if [ -n "$COMPONENT_PATH" ]; then
  FULL_TAG="${COMPONENT_PATH}/${TAG_PREFIX}${VERSION}"
else
  FULL_TAG="${TAG_PREFIX}${VERSION}"
fi

# Create summary header
echo "## 🚀 Release Preview" > "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Check if a release will be created
if [ -n "$VERSION" ]; then
  echo "This PR will create version \`$FULL_TAG\` when merged to **$BASE_BRANCH**." >> "$OUTPUT_FILE"
  echo "" >> "$OUTPUT_FILE"

  # Display changelog from the logs file
  if [ -f "$LOGS_FILE" ] && [ -s "$LOGS_FILE" ]; then
    echo "### 📝 Changelog" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    cat "$LOGS_FILE" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
  fi

  echo "---" >> "$OUTPUT_FILE"
  echo "" >> "$OUTPUT_FILE"
  echo "_Version calculated based on conventional commits since the last release._" >> "$OUTPUT_FILE"
else
  # No release will be created
  echo "ℹ️ **No release will be created from this PR.**" >> "$OUTPUT_FILE"
  echo "" >> "$OUTPUT_FILE"
  echo "This can happen when:" >> "$OUTPUT_FILE"
  echo "- No conventional commits are present since the last release" >> "$OUTPUT_FILE"
  echo "- All commits use the \`no-release\` scope" >> "$OUTPUT_FILE"
  echo "- The PR only contains non-release changes (e.g., docs, ci)" >> "$OUTPUT_FILE"
fi

# Output the summary to stdout for GitHub Actions
cat "$OUTPUT_FILE"
