# AGENTS.md

## Project overview

Scallop is a DataLog-based language for logical, probabilistic, and differentiable
reasoning. It is implemented as a Rust workspace with Python bindings
(`scallopy`), command-line tools, plugins, a WASM build, and a VSCode extension.

- **Primary language:** Rust (2018 edition), with Python bindings via PyO3.
- **Workspace root:** `Cargo.toml` defines members in `core/`, `etc/`, and `lib/`.
- **Toolchain requirement:** Rust **nightly** — the core crate uses unstable
  features (`#![feature(...)]`).
- **Python binding:** `etc/scallopy/`, built with [maturin](https://www.maturin.rs/).
  CI tests against Python 3.10; local docs mention 3.8+.

## Setup commands

- Install Rust nightly:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup default nightly
  ```
- Create a Python virtual environment for `scallopy` work:
  ```bash
  python3 -m venv .env
  source .env/bin/activate
  pip install maturin
  ```
- Optional dependencies for experiments:
  ```bash
  pip install torch torchvision tqdm scikit-learn opencv-python gym transformers matplotlib
  ```

## Build commands

- Build and install core binaries:
  ```bash
  make install-scli      # Scallop interpreter
  make install-sclc      # Scallop compiler
  make install-sclrepl   # Scallop REPL
  ```
- Build the Python binding in-place:
  ```bash
  make develop-scallopy
  ```
- Build and install the Python binding wheel:
  ```bash
  make install-scallopy
  ```
- Quick workspace check:
  ```bash
  make check             # cargo check --workspace
  cargo build --release  # builds default workspace members
  ```

## Test commands

- Rust workspace tests:
  ```bash
  make test-cargo        # cargo test --workspace
  make test-cargo-ignored # cargo test --workspace -- --ignored
  ```
- Python binding tests:
  ```bash
  make test-scallopy     # develop-scallopy + python3 etc/scallopy/tests/test.py
  ```
- Full local test run:
  ```bash
  make test              # cargo tests + scallopy tests
  make test-all          # includes ignored cargo tests
  ```

CI is defined in `.github/workflows/scallop-core.yml` (Rust, release mode,
nightly) and `.github/workflows/scallopy.yml` (Python 3.10, conda, with PyTorch).

## Code style

- **Rust:** run `cargo fmt` before committing. Style is configured in
  `rustfmt.toml`:
  - `tab_spaces = 2`
  - `max_width = 120`
- **Python** in this repo uses 2-space indentation and a loose 120-column target.
  Match the surrounding file rather than imposing a new style.
- Keep changes minimal and scoped; avoid opportunistic reformatting or unrelated
  refactors.

## Working with subprojects

| Path | What it is |
|---|---|
| `core/` | `scallop-core` library: compiler, runtime, provenance semirings |
| `etc/scli/` | Scallop interpreter binary |
| `etc/sclc/` | Scallop compiler binary |
| `etc/sclrepl/` | Scallop REPL binary |
| `etc/scallopy/` | Python binding (`scallopy`) built with PyO3/maturin |
| `etc/scallop-cli/` | Standalone Python CLI package |
| `etc/scallopy-plugins/` | Optional plugins (e.g., GPT/Gemini) |
| `etc/scallop-wasm/` | WASM build for the web demo |
| `etc/vscode-scl/` | VSCode language extension |

If you are doing deep work in one subproject, consider adding a nested
`AGENTS.md` inside that directory with subproject-specific commands.

## Common pitfalls

- **Stable Rust errors:** build failures mentioning `#![feature(...)]`,
  `drain_filter`, `map_first_last`, etc., mean you are not on nightly. Run
  `rustup default nightly` and try again.
- **Torch-dependent scallopy tests:** some tests in `etc/scallopy/tests/` and the
  ignored cargo tests require PyTorch. Install it or skip those tests.
- **Duplicate binary name errors** when building on some platforms usually come
  from workspace membership collisions involving `etc/sclc` or `etc/scallopy`.
  If you hit this, check the workspace `Cargo.toml` and the relevant
  subproject manifests.

## Security & hygiene

- Do not commit secrets, API keys, or large model checkpoints.
- `.env/` is a local Python virtual environment and is gitignored; keep it out
  of version control.
- Be cautious with `unsafe` code and PyO3/Rust FFI boundaries; prefer safe
  wrappers and explicit error handling.

## PR instructions

- Run `cargo fmt` and `cargo test --workspace` before opening a PR.
- For `scallopy` changes, run `make develop-scallopy` and the relevant test
  subset (or `make test-scallopy` if PyTorch is installed).
- Keep diffs focused on the stated change.
- If a PR spans multiple subprojects, mention each one in the description.
