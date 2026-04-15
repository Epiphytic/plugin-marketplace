#!/usr/bin/env bash
set -euo pipefail

# Run semantic-release and capture output
# Usage: run-semantic-release.sh <semantic_release_dir> <should_create_release> <output_file> [pr_branch]
#
# Environment variables:
#   DEBUG - Set to 'true' to enable semantic-release debug mode (default: false)

SEMANTIC_RELEASE_DIR="$1"
SHOULD_CREATE_RELEASE="$2"
OUTPUT_FILE="$3"
PR_BRANCH="${4:-}"

# Build debug flag based on DEBUG environment variable
DEBUG_FLAG=""
if [[ "${DEBUG:-false}" == "true" ]]; then
  DEBUG_FLAG="--debug"
fi

# For PRs, update git symbolic-ref to point to the PR head branch
# This makes git commands (including semantic-release) detect the correct branch
if [[ -n "$PR_BRANCH" ]]; then
  echo "Current git ref: $(git symbolic-ref HEAD 2>/dev/null || echo 'detached')"
  echo "Updating git HEAD to refs/heads/$PR_BRANCH"
  git symbolic-ref HEAD "refs/heads/$PR_BRANCH"
  echo "Updated git ref: $(git symbolic-ref HEAD)"

  # Also export GITHUB_REF for semantic-release and child processes
  export GITHUB_REF="refs/heads/$PR_BRANCH"
  echo "Set GITHUB_REF=$GITHUB_REF"
fi

if [[ "$SHOULD_CREATE_RELEASE" == "true" ]]; then
  # Actual release
  npx --prefix "$SEMANTIC_RELEASE_DIR" semantic-release $DEBUG_FLAG 2>&1 | tee "$OUTPUT_FILE"
  exit "${PIPESTATUS[0]}"
else
  # Dry-run mode - use --no-ci (semantic-release will detect branch from git)
  npx --prefix "$SEMANTIC_RELEASE_DIR" semantic-release --dry-run --no-ci $DEBUG_FLAG 2>&1 | tee "$OUTPUT_FILE"
  # In dry-run mode, don't fail on non-zero exit codes
  exit 0
fi
