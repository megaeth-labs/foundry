# Foundry (MegaETH Fork)

MegaETH's fork of [Foundry](https://github.com/foundry-rs/foundry), pinned at **v1.3.0**.

## MegaETH Mode

Run `forge test` and `forge coverage` with MegaETH EVM semantics (gas forwarding, SALT gas metering, system contracts) via the `--megaeth` flag:

```bash
forge test --megaeth -vvv
forge coverage --megaeth --report lcov
```

**Unsupported combinations** (rejected with a clear error rather than silently degrading):

- `--megaeth --isolate` / inline `isolate = true` — isolation mode not implemented.
- `--megaeth --fork-url <URL>` — fork-aware external environment (Oracle / SALT bucket) not implemented.

**Cheatcodes** (`vm.prank`, `vm.deal`, `vm.expectRevert`, etc.) are silently skipped under `--megaeth`. Use pure Solidity assertions (`require`).

See [`testdata/megaeth/`](./testdata/megaeth/) for gas-divergence examples and cross-validation harness.

## MegaETH ↔ mega-evm consistency

`--megaeth` is backed by the [`mega-evm`](https://crates.io/crates/mega-evm) library. To detect library-level drift:

- [`crates/forge/tests/cli/megaeth.rs`](./crates/forge/tests/cli/megaeth.rs) holds E2E tests, including a `megaeth_live_cross_validate` test that installs the upstream `mega-evme` binary from crates.io and compares outputs byte-for-byte.
- [`.github/workflows/megaeth-live-validate.yml`](./.github/workflows/megaeth-live-validate.yml) runs this cross-validation daily and on any PR touching MegaETH integration code.

## Key Dependencies

| Crate | Version |
|---|---|
| revm | 27.0.3 |
| op-revm | 8.0.3 |
| alloy-evm | 0.15.0 |
| alloy | 1.0.23 |
| mega-evm | 1.5.1 |

## Upstream

- **Base version**: [foundry-rs/foundry v1.3.0](https://github.com/foundry-rs/foundry/releases/tag/v1.3.0)
- **Upstream repo**: [foundry-rs/foundry](https://github.com/foundry-rs/foundry)

## License

Licensed under either of [Apache License](./LICENSE-APACHE), Version 2.0 or [MIT License](./LICENSE-MIT) at your option.
