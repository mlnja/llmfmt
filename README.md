<h1 align="center">llmfmt</h1>

<p align="center">
  Deterministic CLI for converting structured data into compact, prompt-ready text for LLM workflows.
</p>

<p align="center">
  <a href="https://github.com/mlnja/llmfmt/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/mlnja/llmfmt?style=flat-square" alt="License" />
  </a>
  <a href="https://github.com/mlnja/llmfmt/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/mlnja/llmfmt/ci.yml?branch=main&style=flat-square&label=ci" alt="CI" />
  </a>
  <a href="https://github.com/mlnja/llmfmt/releases">
    <img src="https://img.shields.io/github/v/release/mlnja/llmfmt?style=flat-square" alt="Latest release" />
  </a>
  <img src="https://img.shields.io/badge/runtime-Rust-000000?style=flat-square&logo=rust" alt="Rust runtime" />
  <img src="https://img.shields.io/badge/output-TOON%20%7C%20TSV%20%7C%20YAML%20%7C%20JSON-2EA44F?style=flat-square" alt="Output formats" />
  <img src="https://img.shields.io/badge/routing-versioned%20profiles-0A66C2?style=flat-square" alt="Versioned profiles" />
</p>

<p align="center">
  <a href="#quick-commands">Quick Commands</a>
  ·
  <a href="#cli">CLI</a>
  ·
  <a href="#automation">Automation</a>
</p>

It is intentionally narrow:

- input: JSON, YAML, TOML, CSV, TOON
- output: TOON, TSV, YAML, compact JSON
- routing: versioned heuristic profiles
- stats: byte-based size estimates, not exact tokenizer counts

The point is reproducible formatting for shell pipelines, not universal token optimality.

## Quickstart

GitHub READMEs do not support real tabs, so the examples below are grouped as separate command/output blocks.

### Auto-route a local JSON file

```bash
llmfmt users.json
```

```text
id	name	role
1	alice	admin
2	bob	user
3	charlie	user
```

### Force TOON output

```bash
llmfmt users.json --output-format toon
```

```text
users[3]{id,name,role}:
  1,alice,admin
  2,bob,user
  3,charlie,user
```

### Wrap API output for direct prompt use

```bash
curl -s https://jsonplaceholder.typicode.com/users | llmfmt --wrap
```

````text
```yaml
- address:
    city: Gwenborough
    geo:
      lat: '-37.3159'
      lng: '81.1496'
...
```
````

### Run quietly in scripts

```bash
curl -s https://jsonplaceholder.typicode.com/users | llmfmt --stats off
```

```text
- address:
    city: Gwenborough
    geo:
      lat: '-37.3159'
      lng: '81.1496'
...
```

## Why this exists

Most structured data shown to LLMs is still dumped as JSON, even when that is verbose or awkward to scan. `llmfmt` sits in the middle of a pipeline and rewrites the same data into a format that is often smaller and easier for a model to read.

The product wedge is:

- deterministic output
- stable routing
- stdin/stdout-first CLI ergonomics
- a small native binary

## Conversion model

`llmfmt` performs the same steps on every run:

1. Read from stdin or a file.
2. Detect the input format, unless `--input-format` is set.
3. Parse into `serde_json::Value`.
4. Canonicalize object key ordering for deterministic behavior.
5. Analyze the shape of the data.
6. Select an output format through a frozen profile.
7. Emit the rendered payload and a size estimate.

For the same input bytes, profile, and flags, the selected format and payload should be stable.

## Input formats

Supported inputs:

- JSON
- YAML
- TOML
- CSV
- TOON

Detection priority:

1. JSON
2. TOON
3. CSV/TSV-style delimited text
4. YAML
5. TOML

If auto-detection fails, pass `--input-format`.

## Output formats

- `toon`
  Best for uniform arrays of objects.
- `tsv`
  Best for flat dense tables with scalar cells only.
- `yaml`
  Best for moderately nested but still human-scannable structures.
- `json-compact`
  Fallback for deep, irregular, or mixed structures.

`csv` is supported as input, but TSV is the preferred tabular output.

## Routing

Routing is fully delegated to the active profile.

Current built-in profile:

- `20260410`
- `20260411`
- `latest` resolves to `20260411`

Current routing intent:

- flat dense object arrays with few fields prefer TSV
- wider uniform object arrays prefer TOON
- moderate nesting prefers YAML
- deep or irregular structures fall back to compact JSON

Profile ids are emitted in stats output so behavior is diagnosable and reproducible.

## Data analysis

The router currently works from this summary:

```rust
pub struct DataAnalysis {
    pub depth: usize,
    pub row_count: usize,
    pub field_count: usize,
    pub sparsity: f32,
    pub uniformity: f32,
    pub has_nested_arrays: bool,
    pub is_uniform_object_array: bool,
    pub is_flat_object_array: bool,
    pub is_deeply_nested: bool,
}
```

These fields are internal routing inputs, not a stable public API contract yet.

## CLI

```text
llmfmt [INPUT]
  --input-format <json|yaml|toml|csv|toon>
  --output-format <toon|tsv|yaml|json-compact>
  --profile <latest|YYYYMMDD>
  --wrap
  --stats <text|json|off>
  -o <file>
```

Defaults:

- profile: `latest`
- stats: `text`

## Example

```bash
printf '[{"id":1,"name":"alice"},{"id":2,"name":"bob"}]' | llmfmt
```

Stdout:

```text
id	name
1	alice
2	bob
```

Stderr:

```text
tsv | size 47B→22B (-53%) [auto|estimate:bytes|profile:20260410]
```

## Quick commands

These examples assume `llmfmt` is already installed and available on `PATH`.

Convert a file with auto-routing:

```bash
llmfmt users.json
```

Force TOON output from a JSON file:

```bash
llmfmt users.json --output-format toon
```

Force compact JSON from TOON input:

```bash
llmfmt users.toon --output-format json-compact --stats off
```

Convert YAML config data:

```bash
llmfmt config.yaml
```

Convert CSV input to TSV:

```bash
llmfmt metrics.csv --output-format tsv
```

Write output to a file:

```bash
llmfmt users.json --output-format toon -o /tmp/users.toon
```

Wrap the selected output in a fenced block:

```bash
llmfmt users.json --wrap
```

Use `curl` in a shell pipeline:

```bash
curl -s https://jsonplaceholder.typicode.com/users | llmfmt --wrap
```

Use `curl` with a forced format and no stats:

```bash
curl -s https://jsonplaceholder.typicode.com/users \
  | llmfmt --output-format toon --stats off
```

Wrapped output:

```bash
printf '{"users":[{"id":1,"name":"alice","role":"admin"}]}' \
  | llmfmt --output-format toon --wrap
```

```text
```toon
users[1]{id,name,role}:
  1,alice,admin
```
```

## Stats

Stats are byte-based estimates. They are useful for relative comparisons but should not be called token counts.

Text mode:

```text
tsv | size 47B→22B (-53%) [auto|estimate:bytes|profile:20260410]
```

JSON mode emits:

- `input_format`
- `output_format`
- `profile`
- `forced`
- `estimate_kind`
- `input_bytes`
- `output_bytes`
- `delta_percent`

To suppress stats entirely:

```bash
llmfmt users.json --stats off
```

Or suppress `stderr` at the shell level:

```bash
llmfmt users.json 2>/dev/null
```

## Validation

TSV output is only allowed when:

- the top level is an array of objects, or a single-key object containing such an array
- every row is an object
- every field is scalar

Nested arrays and objects are rejected for TSV output.

TOON parsing and emission use the official `toon-format` crate.

## Automation

This repo includes:

- GitHub Actions CI for `check`, `clippy`, `fmt`, and `test`
- GitHub Actions release packaging for Linux and macOS tarballs
- a `just update-tap` helper to update a Homebrew formula in a sibling tap repo

Release assets are expected under `mlnja/llmfmt`. The tap updater defaults to that repo and can still be overridden if needed.

## Non-goals

- exact model-specific token counting
- field filtering or truncation
- runtime-loaded routing configs
- lossy abbreviation of keys or values
- replacing JSON in storage or APIs

## Development

```bash
cargo test
```
