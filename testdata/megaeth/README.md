# MegaETH Test Cases

Verify `forge test --megaeth` produces correct MegaETH EVM semantics.

For automated E2E tests covering CLI flag rejection and cross-validation, see
[`crates/forge/tests/cli/megaeth.rs`](../../crates/forge/tests/cli/megaeth.rs) —
those run in CI via `.github/workflows/build-and-test.yml`, plus a daily live
cross-validation in `.github/workflows/megaeth-live-validate.yml`.

This directory is for **manual** exploration and gas inspection.

## Setup

```bash
cd /path/to/foundry

# Build
cargo build -p forge
export PATH="$(pwd)/target/debug:$PATH"

# Install forge-std (ignored via .gitignore — do not commit)
cd testdata/megaeth
forge install foundry-rs/forge-std --no-git
```

## Run

```bash
# Ethereum mode (baseline)
forge test -vvv

# MegaETH mode
forge test --megaeth -vvv

# Coverage
forge coverage --megaeth --report lcov
```

## Expected

11 tests across 2 files pass in both modes. Gas values differ between modes:

### `MegaETH.t.sol` — gas semantics divergence (7 tests)

| Test | Ethereum | MegaETH | What it verifies |
|---|---|---|---|
| test_gasForwardingRatio | ~50k | ~90k | 63/64 vs 98/100 gas forwarding |
| test_maxRecursionDepth | ~925k | ~667k | Different retention → different depth |
| test_twoHopGasDecay | ~106k | ~145k | Multi-hop gas decay |
| test_mint | ~11k | ~51k | Storage gas metering |
| test_transfer | ~37k | ~77k | Storage gas metering |
| test_transferInsufficientBalance | ~8k | ~48k | Revert handling |
| test_crossValidate | ~123k | ~162k | Composite: SSTORE + compute + CALL |

### `CrossValidate.t.sol` — mega-evme reference values (4 tests)

| Test | Verifies |
|---|---|
| test_basicReturn | `0x604260005260206000f3` → return `0x...42` |
| test_inputEcho | `0x366000600037366000f3` with `0xdeadbeef` input → echoes back |
| test_revert | `0x5f5ffd` → reverts |
| test_gasProbe_crossValidate | `GasProbe.run()` returns exactly **95086** (hardcoded mega-evme ref) |

Reference values are pinned to `mega-evm = 1.5.1`. See the automated
`megaeth_live_cross_validate` test for how to detect drift.

## Unsupported flag combinations

These now error out rather than silently degrade:

| Combination | Behavior |
|---|---|
| `forge test --megaeth --isolate` | rejected — isolation not implemented |
| `forge test --megaeth --fork-url <URL>` | rejected — no fork-aware external env |
| `forge coverage --megaeth --isolate` | rejected |
| `forge coverage --megaeth --fork-url <URL>` | rejected |
| `/// forge-config: default.isolate = true` + `--megaeth` | rejected at test time |

## Cross-Validate with mega-evme (manual)

For deeper debugging. The automated version lives in
`crates/forge/tests/cli/megaeth.rs::megaeth_live_cross_validate`.

```bash
# 1. Get runtime bytecode for GasProbe
forge inspect GasProbe deployedBytecode

# 2. Run with mega-evme (reference)
#    selector for run() = 0xc0406226
mega-evme run --json --spec Rex4 --gas 10000000 \
  --input 0xc0406226 "0x<runtime_bytecode>"

# 3. Run with forge --megaeth
forge test --megaeth --match-test test_gasProbe_crossValidate -vvvv

# 4. Compare:
#    mega-evme "output" field (hex, decoded) = forge trace return value
#    Both must return 95086 (0x1736e) for GasProbe::run().
```

## Limitations

- **Cheatcodes silently skipped**: `vm.prank`, `vm.deal`, `vm.expectRevert` do nothing under `--megaeth`. Tests here use pure Solidity assertions only.
- **console.log disabled**: use `-vvvv` trace output instead.
- **Isolation mode**: not implemented — `--isolate` and inline `isolate = true` are rejected (see table above).
- **Fork mode**: not implemented — external environment reads (Oracle / SALT bucket) use stubbed defaults, so `--fork-url` is rejected to prevent incorrect results.
