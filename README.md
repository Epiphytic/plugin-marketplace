# Epiphytic Plugin Marketplace

Enterprise-scale Claude Code plugin marketplace for the Gear agentic workflow system.

## Available Plugins

### 🔀 agent-fork-join
**Version:** 0.5.0
**Source:** [Epiphytic/agent-fork-join](https://github.com/Epiphytic/agent-fork-join)

Multi-agent git workflow orchestrator with isolated worktrees, FIFO merge queue, and automated PR lifecycle management.

### 🪝 captain-hook
**Version:** 0.2.0
**Source:** [Epiphytic/captain-hook](https://github.com/Epiphytic/captain-hook)

Intelligent permission gating for AI coding assistants with 6-tier decision cascade.

### 🐟 babelfish
**Version:** 0.1.0
**Source:** Local (bundled in this repository)

Bidirectional prose-to-AISP translation using gear-core and rosetta-aisp-llm.

## Installation

### Add the Marketplace

```bash
/plugin marketplace add Epiphytic/plugin-marketplace
```

### Install Individual Plugins

```bash
# Install agent-fork-join
/plugin install agent-fork-join@plugin-marketplace

# Install captain-hook
/plugin install captain-hook@plugin-marketplace

# Install babelfish
/plugin install babelfish@plugin-marketplace
```

### Install All Plugins

```bash
/plugin install agent-fork-join@plugin-marketplace captain-hook@plugin-marketplace babelfish@plugin-marketplace
```

## Updating Plugins

To get the latest versions of installed plugins:

```bash
/plugin marketplace update plugin-marketplace
/plugin update
```

## Troubleshooting

### Marketplace Validation

To validate the marketplace configuration:

```bash
claude plugin validate .
```

Or from within Claude Code:

```bash
/plugin validate .
```

### Plugin Installation Issues

If plugins fail to install:

1. Ensure you have access to the GitHub repositories (for external plugins)
2. Check that you're authenticated with GitHub: `gh auth status`
3. Verify the marketplace is up to date: `/plugin marketplace update`
4. Check Claude Code logs for detailed error messages

### Repository Structure

```
plugin-marketplace/
├── .claude-plugin/
│   └── marketplace.json          # Marketplace manifest
├── plugin/
│   └── babelfish/                # Bundled plugin (lives in this repo)
│       └── .claude-plugin/
│           └── plugin.json
├── CLAUDE.md                     # Project instructions
└── README.md                     # This file
```

External plugins (agent-fork-join, captain-hook) are pulled from their respective GitHub repositories when installed.

## Development

### Local Testing

To test the marketplace locally before publishing:

```bash
/plugin marketplace add ./path/to/plugin-marketplace
/plugin install test-plugin@plugin-marketplace
```

### Adding New Plugins

1. Create the plugin in its own repository (for external plugins)
2. Add the plugin entry to `.claude-plugin/marketplace.json`
3. Use GitHub source format:
   ```json
   {
     "name": "my-plugin",
     "source": {
       "source": "github",
       "repo": "Epiphytic/my-plugin"
     },
     "description": "Plugin description",
     "version": "1.0.0",
     "category": "development"
   }
   ```
4. Validate the marketplace: `claude plugin validate .`
5. Commit and push changes

## Documentation

- [Gear System Architecture](docs/architecture.md)
- [Plugin Development Guide](docs/plugin-development.md)
- [Claude Code Plugin Documentation](https://code.claude.com/docs/en/plugins)
- [Marketplace Documentation](https://code.claude.com/docs/en/plugin-marketplaces)

## Support

For issues or questions:
- Open an issue in the [plugin-marketplace repository](https://github.com/Epiphytic/plugin-marketplace)
- Check individual plugin repositories for plugin-specific issues
- See [CLAUDE.md](CLAUDE.md) for development guidelines

## License

See individual plugin repositories for license information.
