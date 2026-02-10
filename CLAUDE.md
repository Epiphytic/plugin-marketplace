# Gear Plugin Marketplace

Gear is a Claude Code plugin marketplace for enterprise-scale agentic workflows, powered by a unified Rust binary (`gear-core`).

## Essential Rules

1. **Rust-First**: All core logic in Rust, avoid bash scripts for heavy lifting
2. **Performance**: Every PR must pass benchmarks, max 5% regression allowed
3. **Security**: Validate all input, never store secrets in code
4. **Dual Documentation**: All docs in `.md` AND `.aisp` format
5. **Testing**: Unit, integration, and performance tests required on all PRs

## Quick Reference

| Need | Document |
|------|----------|
| System architecture | `docs/architecture.aisp` |
| Building plugins | `docs/plugin-development.aisp` |
| Configuration | `docs/configuration.aisp` |
| Writing tests | `docs/testing.aisp` |
| Security practices | `docs/security.aisp` |
| Performance tuning | `docs/performance.aisp` |
| Code style | `docs/code-style.aisp` |
| Agent development | `AGENTS.aisp` |

## Plugin Suite

| Plugin | Purpose |
|--------|---------|
| gear | Entry point, project setup (`/first`, `/config`) |
| dash | Git workflow orchestration with isolated worktrees |
| babelfish | Prompt translation to AISP format |
| memo | Distributed vector memory across projects |
| uplink | Cross-machine instance coordination |
| guide | External LLM validation for plans/code |
| tango | Instance spawning (local and remote) |
| infinite-improbability-drive | Meta-orchestration for impossible tasks |
| captain-hook | Intelligent permission gating with 6-tier decision cascade (v0.1.0) |

## Core Commands

```bash
# gear-core CLI
gear-core daemon --mode merge-queue     # Start merge daemon
gear-core cli config get <key>          # Read config
gear-core cli translate "<prompt>"      # Translate to AISP
gear-core cli memory query "<query>"    # Query memo

# Documentation conversion
npx aisp-converter --input file.md --output file.aisp
```

## Directory Structure

```
gear/
├── CLAUDE.md, CLAUDE.aisp       # Essential instructions
├── AGENTS.md, AGENTS.aisp       # Agent development
├── docs/                        # All docs (.md + .aisp)
├── core/                        # Rust binary source
├── plugins/                     # Individual plugins
├── tests/                       # Test suites
└── .github/workflows/           # CI/CD
```

## Commit Convention

```
<type>(<scope>): <description>
```
Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`

## Configuration Hierarchy

1. Default → 2. Global (`~/.config/gear/`) → 3. Project (`.gear/`) → 4. Environment (`GEAR_*`) → 5. CLI flags

Secrets go in `.local.toml` files (gitignored).

## Creating .aisp Files

All documentation must exist in dual format. Convert using:
```bash
npx aisp-converter --input docs/file.md --output docs/file.aisp
```

## Resources

- Human docs: `docs/*.md`
- Machine docs: `docs/*.aisp`
- Agent guide: `AGENTS.md` / `AGENTS.aisp`
