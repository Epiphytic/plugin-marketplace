/**
 * Custom semantic-release plugin to filter commits by component path
 * This plugin wraps around semantic-release-unsquash to add path filtering
 */

const { execSync } = require('child_process');

/**
 * Map component path to actual git paths
 * Component paths like "actions/foo" need to check both "actions/foo" and ".github/actions/foo"
 * Component paths like "workflows/bar" need to check ".github/workflows/bar.yml"
 *
 * @param {string} componentPath - The component path (e.g., "actions/semantic-release")
 * @returns {string[]} Array of git paths to check
 */
function mapComponentToGitPaths(componentPath) {
  if (!componentPath) {
    return [];
  }

  const paths = [];

  // For actions/* paths, check both locations
  if (componentPath.startsWith('actions/')) {
    const actionPath = componentPath.substring('actions/'.length);
    // Check .github/actions/foo (composite actions in .github/)
    paths.push(`.github/actions/${actionPath}`);
    // Also check actions/foo itself (actions at root level)
    paths.push(componentPath);
  }
  // For workflows/* paths
  else if (componentPath.startsWith('workflows/')) {
    const workflowName = componentPath.substring('workflows/'.length);
    // Check .github/workflows/foo.yml
    paths.push(`.github/workflows/${workflowName}.yml`);
    // Also check workflows/foo (in case it's at root)
    paths.push(componentPath);
  }
  // For any other path, use it as-is
  // This handles custom paths like "terraform/modules/vpc", "lib/utils", etc.
  else {
    paths.push(componentPath);
  }

  return paths;
}

/**
 * Get commits that affect a specific path
 * @param {string} componentPath - The path to filter commits by
 * @param {string} fromRef - Starting git ref
 * @param {string} toRef - Ending git ref (usually HEAD)
 * @returns {Set<string>} Set of commit SHAs that touched the path
 */
function getCommitsForPath(componentPath, fromRef = '', toRef = 'HEAD') {
  if (!componentPath) {
    // No filtering needed
    return null;
  }

  try {
    // Map component path to actual git paths
    const gitPaths = mapComponentToGitPaths(componentPath);
    const allCommits = new Set();

    // Get commits for each path
    for (const gitPath of gitPaths) {
      const gitLogCmd = fromRef
        ? `git log ${fromRef}..${toRef} --format=%H -- ${gitPath}`
        : `git log ${toRef} --format=%H -- ${gitPath}`;

      const output = execSync(gitLogCmd, { encoding: 'utf-8' });
      const commits = output
        .trim()
        .split('\n')
        .filter(sha => sha.length > 0);

      commits.forEach(sha => allCommits.add(sha));
    }

    return allCommits;
  } catch (error) {
    console.error('Error getting commits for path:', error.message);
    return new Set();
  }
}

/**
 * Filter commits array by component path
 * @param {Array} commits - Array of commit objects from semantic-release
 * @param {string} componentPath - The path to filter by
 * @returns {Array} Filtered commits array
 */
function filterCommitsByPath(commits, componentPath) {
  if (!componentPath || !commits || commits.length === 0) {
    return commits;
  }

  console.log(`Filtering ${commits.length} commits by path: ${componentPath}`);

  // Get the set of commit SHAs that touched this path
  const relevantCommits = getCommitsForPath(componentPath);

  if (!relevantCommits || relevantCommits.size === 0) {
    console.log(`No commits found for path: ${componentPath}`);
    return [];
  }

  // Filter commits to only those that touched the path
  const filtered = commits.filter(commit => {
    const sha = commit.hash || commit.commit?.long;
    const isRelevant = relevantCommits.has(sha);
    if (!isRelevant) {
      console.log(`Filtering out commit ${sha?.substring(0, 7)}: ${commit.subject || commit.message}`);
    }
    return isRelevant;
  });

  console.log(`Filtered to ${filtered.length} commits that affected ${componentPath}`);
  return filtered;
}

/**
 * verifyConditions plugin hook
 * Filters commits early in the lifecycle, before any analysis happens
 */
async function verifyConditions(pluginConfig, context) {
  const { commits, logger } = context;
  const componentPath = process.env.COMPONENT_PATH || pluginConfig.componentPath;

  if (!componentPath) {
    logger.log('[filter-commits-by-path] No component path specified, skipping commit filtering');
    return;
  }

  logger.log(`[filter-commits-by-path] Filtering commits for component path: ${componentPath}`);
  logger.log(`[filter-commits-by-path] Original commit count: ${commits?.length || 0}`);

  if (commits && commits.length > 0) {
    // Filter commits in the context
    const filteredCommits = filterCommitsByPath(commits, componentPath);

    // Update context with filtered commits
    context.commits = filteredCommits;

    logger.log(`[filter-commits-by-path] Filtered to ${filteredCommits.length} commits`);
  }
}

/**
 * generateNotes plugin hook
 * Filters commits before generating release notes
 */
async function generateNotes(pluginConfig, context) {
  const { commits, logger } = context;
  const componentPath = process.env.COMPONENT_PATH || pluginConfig.componentPath;

  if (!componentPath) {
    logger.log('No component path specified, skipping commit filtering for notes');
    return;
  }

  logger.log(`Filtering commits for release notes: ${componentPath}`);

  // Filter commits in the context
  const filteredCommits = filterCommitsByPath(commits, componentPath);

  // Update context with filtered commits
  context.commits = filteredCommits;

  logger.log(`Filtered from ${commits.length} to ${filteredCommits.length} commits for release notes`);

  // Return undefined to let the next plugin generate the notes
  return undefined;
}

module.exports = {
  verifyConditions,
  generateNotes,
  filterCommitsByPath,
  getCommitsForPath
};
