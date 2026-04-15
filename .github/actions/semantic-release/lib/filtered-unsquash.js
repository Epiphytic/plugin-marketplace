/**
 * Extended semantic-release-unsquash plugin with path filtering
 * This plugin extends semantic-release-unsquash to filter commits by path before processing
 */

const {
  analyzeCommits: originalAnalyzeCommits,
} = require('@semantic-release/commit-analyzer');
const {
  generateNotes: originalGenerateNotes,
} = require('@semantic-release/release-notes-generator');
const { getUnsquashedCommits } = require('semantic-release-unsquash/src/get-unsquashed-commits');
const { filterCommitsByPath } = require('./filter-commits-by-path');

/**
 * Analyze commits with path filtering and unsquashing
 */
const analyzeCommits = async (pluginConfig, context) => {
  const { commitAnalyzerConfig, componentPath } = pluginConfig || {};
  const { logger } = context;

  // First, get unsquashed commits
  let commits = getUnsquashedCommits(context);
  logger.log(`[filtered-unsquash] Got ${commits.length} unsquashed commits`);

  // Then, filter by component path if specified
  if (componentPath) {
    logger.log(`[filtered-unsquash] Filtering commits for path: ${componentPath}`);
    commits = filterCommitsByPath(commits, componentPath);
    logger.log(`[filtered-unsquash] Filtered to ${commits.length} commits`);

    if (commits.length === 0) {
      logger.log('[filtered-unsquash] No relevant commits found, no release needed');
      return null; // No release
    }
  }

  // Finally, analyze the filtered commits
  const releaseType = await originalAnalyzeCommits(commitAnalyzerConfig ?? {}, {
    ...context,
    commits,
  });

  logger.log(`[filtered-unsquash] lastRelease: ${JSON.stringify(context.lastRelease)}`);
  logger.log(`[filtered-unsquash] releaseType: ${releaseType}`);

  return releaseType;
};

/**
 * Generate notes with path filtering and unsquashing
 */
const generateNotes = async (pluginConfig, context) => {
  const { notesGeneratorConfig, componentPath } = pluginConfig || {};
  const { logger } = context;

  if (notesGeneratorConfig === false) {
    return;
  }

  // First, get unsquashed commits
  let commits = getUnsquashedCommits(context);
  logger.log(`[filtered-unsquash] Got ${commits.length} unsquashed commits for notes`);

  // Then, filter by component path if specified
  if (componentPath) {
    logger.log(`[filtered-unsquash] Filtering commits for notes, path: ${componentPath}`);
    commits = filterCommitsByPath(commits, componentPath);
    logger.log(`[filtered-unsquash] Filtered to ${commits.length} commits for notes`);
  }

  // Finally, generate notes from the filtered commits
  return originalGenerateNotes(notesGeneratorConfig ?? {}, {
    ...context,
    commits,
  });
};

module.exports = { analyzeCommits, generateNotes };
