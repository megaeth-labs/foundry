# AGENTS.md

This file provides guidance to AI agents (e.g., claude code, codex, cursor, etc.) when working with code in this repository.

## Project Overview

MegaETH's fork of [Foundry](https://github.com/foundry-rs/foundry), pinned at **v1.3.0**.

## Build & Development Commands

```bash
# Build
make build
make build-release                        # optimized build

# Test
make test-forge                           # forge only
make test                                 # CI-equivalent workspace tests

# Check compiler errors (preferred over clippy for quick checks)
make check
make check-forge

# Lint (CI runs all of these)
make fmt
make clippy
make lint

# Local pre-PR checks
make pr
```

CI installs the Rust toolchain, the Foundry toolchain, `cargo-nextest`, Python 3.11, and `vyper==0.4.3`.
Keep the `make test` command in sync with `.github/workflows/build-and-test.yml`.

## Workspace Structure

| Crate        | Path                 | Purpose                                            |
| ------------ | -------------------- | -------------------------------------------------- |
| `forge`      | `crates/forge`       | Solidity testing, build, coverage, and script CLI  |
| `cast`       | `crates/cast`        | Ethereum RPC and utility CLI                       |
| `anvil`      | `crates/anvil`       | Local Ethereum development node                    |
| `chisel`     | `crates/chisel`      | Solidity REPL                                      |
| `cheatcodes` | `crates/cheatcodes`  | Foundry cheatcode implementations                  |
| `evm`        | `crates/evm`         | EVM execution, traces, coverage, and fuzz helpers  |
| `config`     | `crates/config`      | Foundry configuration parsing and defaults         |
| `test-utils` | `crates/test-utils`  | Shared Rust test utilities and RPC helpers         |

## Version Control

The main branch is `main`, but it's protected.
All changes should be made via PRs on GitHub.

### Branch naming convention

The naming convention for git branches is `[DEVELOPER NAME]/[CHANGE CATEGORY]/[SHORT DESCRIPTION]`, where:

- `[DEVELOPER NAME]` is the (nick)name of the developer.
- `[CHANGE CATEGORY]` should indicate what type of modifications this PR is making, e.g., feat, fix, doc, ci, refactor, etc.
- `[SHORT DESCRIPTION]` is a short (a few words) description of the detailed changes in this branch.

## Workflows

### Committing changes

When requested to commit changes, the agent should first review all current changes in the working tree, regardless of whether they are staged or not.
There may be other changes in the worktree in addition to those made by the agent, which may also need to be included.
If the agent is not sure whether some changes should be included in the commit, ask the user.
The commit message should reflect the overall changes of the commit, which may be beyond the existing context of the agent.

The commit message should be short and exclude any information of the agent itself.

### Creating PR

When a PR creation is requested, the agent should:

1. Check if the repo is on a branch other than `main`.
   If not, create and checkout to a new branch.
   Make sure to inform the user about this branch creation.
2. Commit the changes in the worktree before fixing linting issues.
3. Run lint check, fix any lint warnings, and then commit if there are any changes.
4. Format the code and commit if there are any changes.
5. Push to the remote.
6. Use `gh` CLI tool to create a PR.
   When generating the PR title and description, consider the overall changes in this branch across commits.
   In the PR description, make sure a `Summary` section is put on the top.
   The PR will be merged with `Squash and Merge` operation, whose commit description should include the summary.

### Implementing features or bug fixes

When the agent is requested to implement a new feature or bug fix, it should consider the following additional aspects in addition to the feature/fix itself and the other requirements by the user.

1. Should the documentation need to be updated or added?
2. Are there sufficient tests for this feature?

## Caveats for Agents

- **Follow existing Foundry patterns.**
  This is a fork, not a rewrite.
  Minimize modifications to upstream files and keep changes isolated.
- **Do NOT modify upstream test fixtures.**
  Don't modify tests inherited from foundry-rs/foundry unless they conflict with our changes.
- **Use `cargo check` (not `cargo clippy`) for compiler error checking.**
  Use `cargo clippy` only when specifically checking lint warnings.
- **Before finishing a change, always run full lint and format checks.**
  Run `make lint` before completion.
- **Do NOT modify upstream RPC endpoints or test infrastructure without checking upstream first.**
  Test utilities (e.g., `crates/test-utils/src/rpc.rs`) use public RPC endpoints that may change over time.
  Always compare with the latest upstream before modifying.
- **Do NOT change pinned execution dependencies without explicit justification.**
  Changes to `revm`, `op-revm`, `alloy-evm`, `alloy`, or `[patch.crates-io]` must explain why the fork needs them and what tests cover the change.
- **One sentence, one line.**
  When writing markdown or similar format files, put each sentence in a separate line.
- **Review guidelines are in `REVIEW.md`.**
  Refer to it for code review conventions and fork-specific review rules.

## MegaETH Integration

This fork adds `forge test --megaeth` and `forge coverage --megaeth`, routing execution through the [`mega-evm`](https://crates.io/crates/mega-evm) library instead of stock revm.

### Where the integration lives

| Concern | Path |
| --- | --- |
| EVM execution entry point | `crates/evm/core/src/backend/{mod,cow}.rs` (`inspect_mega`) |
| MegaCtx inspector fan-out | `crates/evm/evm/src/inspectors/stack.rs` |
| CLI flag validation | `crates/evm/core/src/opts.rs` (`EvmOpts::validate_megaeth`) |
| Test / coverage command guards | `crates/forge/src/cmd/{test,coverage}/*` |
| Builder + inline-config guards | `crates/forge/src/{multi_runner,runner}.rs` |
| Result/state conversion | `crates/evm/core/src/evm.rs` (`convert_mega_result_and_state`) |
| E2E tests | `crates/forge/tests/cli/megaeth.rs` |
| Manual test fixtures | `testdata/megaeth/` (forge-std lib is gitignored) |
| Live cross-validation CI | `.github/workflows/megaeth-live-validate.yml` (daily + on PR touching mega paths) |

### Caveats when modifying MegaETH code

- **`mega-evm` version is pinned.**
  Bumping it requires updating the `MEGA_EVME_VERSION` constant in `crates/forge/tests/cli/megaeth.rs` AND the hardcoded reference values in `testdata/megaeth/test/CrossValidate.t.sol` (e.g. `MEGA_EVME_REFERENCE_GAS = 95086`).
  Run `cargo nextest run -p forge --test cli megaeth_live_cross_validate --run-ignored=only` locally to regenerate.
- **`--megaeth` + `--isolate` / `--fork-url` must stay rejected.**
  Silent degradation is worse than a hard error.
  Any new code path that builds a runner or executes tests under MegaETH must call `EvmOpts::validate_megaeth()` before any network request.
- **Cheatcodes are skipped under `--megaeth`.**
  Do not add `vm.*` calls to `testdata/megaeth/` tests — they appear to succeed but do nothing.
- **System contract deployment must be idempotent.**
  `Backend::ensure_system_contract` checks `code_hash` before writing and preserves existing `balance` / `nonce`.
  Mirrors mega-evme's pattern — do not revert to `AccountInfo::default()`.
