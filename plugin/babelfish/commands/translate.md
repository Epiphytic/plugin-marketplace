---
name: "babelfish:translate"
description: "Translate between prose and AISP notation. Converts markdown/prose files to .aisp and vice versa, with optional LLM fallback for higher accuracy."
---

# /translate Command

Bidirectional prose <-> AISP translation using gear-core (rosetta-aisp + rosetta-aisp-llm).

## When This Command Is Invoked

The user wants to convert between prose (natural language / markdown) and AISP symbolic notation. They may provide:

- A file path to convert
- Inline text to convert
- A direction (to-aisp or to-prose), or let it be auto-detected
- Optional flags: tier, llm fallback, model

## Step 1: Determine Direction and Input

Parse the user's arguments to determine:

1. **Direction**: `to-aisp` (prose -> AISP) or `to-prose` (AISP -> prose)
   - If not specified, auto-detect: if the input contains AISP symbols (`⟦`, `⟧`, `≜`, `∀`, `∃`, `→`, `⊢`, `⊨`, `𝔸`) treat it as AISP and convert to prose; otherwise convert to AISP
2. **Input source**: file path or inline text from arguments
3. **Output**: file path (if converting a file, default to same name with swapped extension: `.md` <-> `.aisp`) or stdout

## Step 2: Run Conversion

### Prose to AISP

```bash
gear-core convert --input "<prose_text>" --format json --llm-fallback --aisp-prompt --verbose 2>&1
```

Or for file input:

```bash
gear-core convert --file "<input_path>" --output "<output_path>" --format text --llm-fallback --aisp-prompt --verbose 2>&1
```

**Tier selection**: If the user specified a tier (minimal, standard, full), add `--tier <tier>`. Otherwise let gear-core auto-detect.

**Model selection**: If the user specified a model (haiku, sonnet, opus), add `--model <model>`. Otherwise gear-core uses the configured default.

### AISP to Prose

```bash
gear-core to-prose --input "<aisp_text>" 2>&1
```

Or for file input:

```bash
gear-core to-prose --file "<input_path>" 2>&1
```

## Step 3: Handle Output

1. **If converting a file**: Write the output to the target path. Report what was written.
   - `.md` -> `.aisp`: Write the AISP output
   - `.aisp` -> `.md`: Write the prose output
2. **If inline text**: Display the conversion result directly to the user.
3. **If `--verbose` output was captured on stderr**: Show tier, confidence, token stats, and whether LLM fallback was used.

## Step 4: Validate (AISP output only)

When the output is AISP, run validation:

```bash
gear-core validate --input "<aisp_output>" --json 2>&1
```

Report the validation result (valid/invalid, tier, density metrics).

## Examples

```
/translate docs/architecture.md                    # Convert MD to AISP (auto-detect direction)
/translate docs/architecture.aisp                  # Convert AISP to MD (auto-detect direction)
/translate to-aisp "Define x as the sum of a and b"  # Inline prose -> AISP
/translate to-prose "x≜a⊕b"                       # Inline AISP -> prose
/translate docs/security.md --tier full --model opus  # Full tier with opus model
```

## Error Handling

- If `gear-core` is not found in PATH, check if the binary exists at the project root: `./core/target/release/gear-core` or `./core/target/debug/gear-core`. If not built, tell the user to run `cargo build --release -p gear-core`.
- If LLM fallback fails (no API key, rate limit), the deterministic conversion result is still returned. Note to the user that accuracy may be lower without LLM fallback.
- If the input file doesn't exist, report the error clearly.
