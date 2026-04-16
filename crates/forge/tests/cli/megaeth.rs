// Tests for MegaETH (`--megaeth`) flag interactions and cross-validation
// against mega-evm reference outputs.

use foundry_test_utils::util::OutputExt;
use std::path::PathBuf;

// ---------- Flag conflict tests ----------

// Verifies that `--megaeth --isolate` is rejected at the CLI level.
forgetest_init!(megaeth_isolate_rejected, |_prj, cmd| {
    cmd.args(["test", "--megaeth", "--isolate"]).assert_failure().stderr_eq(str![[r#"
Error: `--isolate` is not supported with `--megaeth` (MegaETH v1 does not implement isolation mode)

"#]]);
});

// Verifies that `--megaeth --fork-url` is rejected.
forgetest_init!(megaeth_fork_rejected, |_prj, cmd| {
    cmd.args(["test", "--megaeth", "--fork-url", "http://127.0.0.1:1"])
        .assert_failure()
        .stderr_eq(str![[r#"
Error: `--fork-url` is not supported with `--megaeth` (MegaETH v1 does not implement fork-aware external environments)

"#]]);
});

// Verifies that `--megaeth --gas-report` is rejected with a clear message
// attributing the cause to `--gas-report` (which implies isolation), not a
// misleading "--isolate is not supported" from the builder layer.
forgetest_init!(megaeth_gas_report_rejected, |_prj, cmd| {
    cmd.args(["test", "--megaeth", "--gas-report"])
        .assert_failure()
        .stderr_eq(str![[r#"
Error: `--gas-report` is not supported with `--megaeth` (gas reports require isolation mode, which MegaETH v1 does not implement)

"#]]);
});

// Verifies that inline `isolate = true` under --megaeth is rejected at test time
// (not silently ignored). The error surfaces as a test failure with the guard
// message embedded in the failure reason.
forgetest_init!(megaeth_inline_isolate_rejected, |prj, cmd| {
    prj.wipe_contracts();

    prj.add_test(
        "MegaInline.t.sol",
        r#"
import {Test} from "forge-std/Test.sol";

contract Dummy {
    uint256 public x;
    function setX(uint256 v) public { x = v; }
}

/// forge-config: default.isolate = true
contract InlineIsolateTest is Test {
    Dummy dummy;

    function setUp() public {
        dummy = new Dummy();
    }

    function test_runs_without_isolation() public {
        dummy.setX(42);
        require(dummy.x() == 42, "mismatch");
    }
}
    "#,
    )
    .unwrap();

    // The command itself succeeds (test runner runs) but the suite fails because
    // the inline config conflict is surfaced as a setUp/test error.
    let out = cmd.args(["test", "--megaeth"]).assert_failure().get_output().stdout_lossy();
    assert!(
        out.contains("isolate = true") && out.contains("--megaeth"),
        "expected inline-isolate rejection message, got:\n{out}"
    );
});

// Verifies `forge coverage --megaeth --fork-url` is rejected before any network
// request (matches the `forge test` behavior).
forgetest_init!(megaeth_coverage_fork_rejected, |_prj, cmd| {
    cmd.args(["coverage", "--megaeth", "--fork-url", "http://127.0.0.1:1"])
        .assert_failure()
        .stderr_eq(str![[r#"
Error: `--fork-url` is not supported with `--megaeth` (MegaETH v1 does not implement fork-aware external environments)

"#]]);
});

// Verifies `forge coverage --megaeth --isolate` is rejected (previously silently
// ignored because the coverage runner never forwarded `--isolate`).
forgetest_init!(megaeth_coverage_isolate_rejected, |_prj, cmd| {
    cmd.args(["coverage", "--megaeth", "--isolate"]).assert_failure().stderr_eq(str![[r#"
Error: `--isolate` is not supported with `--megaeth` (MegaETH v1 does not implement isolation mode)

"#]]);
});

// Basic sanity: --megaeth alone works for simple tests.
forgetest_init!(megaeth_basic_pass, |prj, cmd| {
    prj.wipe_contracts();

    prj.add_test(
        "Basic.t.sol",
        r#"
import {Test} from "forge-std/Test.sol";

contract MegaBasic is Test {
    function test_addition() public pure {
        require(1 + 1 == 2, "math broke");
    }
}
    "#,
    )
    .unwrap();

    cmd.args(["test", "--megaeth"]).assert_success();
});

// ---------- Cross-validation against mega-evm reference values ----------

// Cross-validate forge --megaeth against mega-evm reference outputs.
//
// The Solidity test file embeds reference values produced by `mega-evme run`
// (the standalone mega-evm CLI) for fixed bytecodes. If forge's mega-evm
// integration diverges from the library's behavior, the `require()` checks
// fail and this test fails.
//
// Reference values are regenerated offline via `megaeth_live_cross_validate`
// (requires mega-evme binary, cache-installed from crates.io on first run).
forgetest_init!(megaeth_cross_validate, |prj, cmd| {
    prj.wipe_contracts();

    prj.add_test(
        "CrossValidate.t.sol",
        r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";

/// Reference values sourced from:
///   mega-evme run --json --spec Rex4 <bytecode> [--input <calldata>]
contract CrossValidate is Test {
    function _deployRuntime(bytes memory runtime) internal returns (address addr) {
        bytes memory code = abi.encodePacked(
            uint8(0x60), uint8(runtime.length),
            uint8(0x80),
            uint8(0x60), uint8(11),
            uint8(0x60), uint8(0),
            uint8(0x39),
            uint8(0x60), uint8(0),
            uint8(0xf3),
            runtime
        );
        assembly { addr := create(0, add(code, 0x20), mload(code)) }
        require(addr != address(0), "deploy failed");
    }

    // mega-evme: output = 0x...42, success = true
    // bytecode: PUSH1 0x42 | MSTORE | RETURN 32 bytes at offset 0
    function test_basicReturn() public {
        address t = _deployRuntime(hex"604260005260206000f3");
        (bool ok, bytes memory ret) = t.call("");
        require(ok, "call failed");
        require(ret.length == 32, "bad len");
        uint256 val;
        assembly { val := mload(add(ret, 0x20)) }
        require(val == 0x42, "wrong val");
    }

    // mega-evme: output = 0xdeadbeef (echo of input), success = true
    // bytecode: CALLDATACOPY then RETURN
    function test_inputEcho() public {
        address t = _deployRuntime(hex"366000600037366000f3");
        (bool ok, bytes memory ret) = t.call(hex"deadbeef");
        require(ok, "call failed");
        require(ret.length == 4, "bad len");
        require(keccak256(ret) == keccak256(hex"deadbeef"), "wrong echo");
    }

    // mega-evme: success = false
    // bytecode: PUSH0 | PUSH0 | REVERT
    function test_revert() public {
        address t = _deployRuntime(hex"5f5ffd");
        (bool ok, ) = t.call("");
        require(!ok, "should revert");
    }
}

/// GasProbe measures its own gas consumption across SSTORE + compute + STATICCALL.
/// This exercises SALT bucket gas, opcode gas, and call gas forwarding — the core
/// semantics that differ between Ethereum and MegaETH.
contract GasProbe {
    uint256 public slot0;
    uint256 public slot1;

    function run() external returns (uint256 gasUsed) {
        uint256 g0 = gasleft();
        slot0 = 42;
        slot1 = 84;
        uint256 x = 1;
        for (uint256 i = 0; i < 100; i++) { x = x * 3 + 1; }
        address(this).staticcall(abi.encodeWithSelector(this.slot0.selector));
        uint256 g1 = gasleft();
        gasUsed = g0 - g1;
        slot0 = gasUsed;
    }
}

contract CrossValidateGasProbe is Test {
    // mega-evme reference: `run --json --spec Rex4 --input 0xc0406226 <GasProbe bytecode>`
    //   → output = 0x...1736e = 95086
    uint256 constant MEGA_EVME_REFERENCE_GAS = 95086;

    function test_gasProbe() public {
        GasProbe probe = new GasProbe();
        uint256 reported = probe.run();
        require(
            reported == MEGA_EVME_REFERENCE_GAS,
            "GasProbe gas drift: forge --megaeth output does not match mega-evme"
        );
    }
}
    "#,
    )
    .unwrap();

    cmd.args(["test", "--megaeth"]).assert_success();
});

// ---------- mega-evme binary management ----------

/// mega-evme version pinned in our cross-validation tests.
/// Must match the `mega-evm` version in workspace Cargo.toml.
const MEGA_EVME_VERSION: &str = "1.5.1";

/// Ensure `mega-evme` is installed and return its path.
///
/// Resolution order:
///   1. `MEGA_EVME` env var (explicit override — point at any prebuilt binary)
///   2. `<foundry>/target/mega-evme-<version>/bin/mega-evme` (cached crates.io install)
///   3. Install from crates.io into the cache path above (one-time cost)
fn ensure_mega_evme() -> PathBuf {
    // 1. Explicit override
    if let Ok(p) = std::env::var("MEGA_EVME") {
        let path = PathBuf::from(p);
        if path.exists() {
            return path;
        }
    }

    // 2. Cached crates.io install under foundry's target dir
    let cache_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join(format!("mega-evme-{MEGA_EVME_VERSION}"));
    let cached_bin = cache_root.join("bin/mega-evme");
    if cached_bin.exists() {
        return cached_bin;
    }

    // 3. Install from crates.io (slow: compiles revm, alloy, etc.).
    // `cargo install` prints its own progress, so no extra status line needed.
    let status = std::process::Command::new("cargo")
        .args([
            "install",
            "mega-evme",
            "--version",
            MEGA_EVME_VERSION,
            "--root",
            cache_root.to_str().unwrap(),
            "--locked",
        ])
        .status()
        .expect("failed to spawn `cargo install mega-evme`");
    assert!(status.success(), "cargo install mega-evme failed");
    assert!(cached_bin.exists(), "mega-evme not found after install");
    cached_bin
}

fn run_mega_evme(bin: &PathBuf, bytecode: &str, input: Option<&str>) -> serde_json::Value {
    let mut args = vec!["run", "--json", "--spec", "Rex4"];
    if let Some(inp) = input {
        args.push("--input");
        args.push(inp);
    }
    args.push(bytecode);
    let out =
        std::process::Command::new(bin).args(&args).output().expect("failed to run mega-evme");
    serde_json::from_slice(&out.stdout).expect("mega-evme bad JSON")
}

// Live cross-validation: run the same bytecodes through both engines and
// assert they agree. Installs `mega-evme` from crates.io on first run
// (cached under `target/mega-evme-<version>/`).
//
// Marked `#[ignore]` because the first run compiles mega-evme (~2 minutes)
// and requires network. Opt-in via `cargo test -- --ignored`.
forgetest_init!(
    #[ignore = "slow: installs mega-evme on first run; use --ignored to opt in"]
    megaeth_live_cross_validate,
    |prj, cmd| {
        let bin = ensure_mega_evme();

        // Raw bytecodes with known mega-evme outputs — these must match the
        // values encoded in `megaeth_cross_validate`'s CrossValidate.t.sol.
        let raw_cases: &[(&str, &str, Option<&str>, &str)] = &[
            (
                "basicReturn",
                "0x604260005260206000f3",
                None,
                "0x0000000000000000000000000000000000000000000000000000000000000042",
            ),
            ("inputEcho", "0x366000600037366000f3", Some("0xdeadbeef"), "0xdeadbeef"),
        ];
        for (name, bytecode, input, expected) in raw_cases {
            let r = run_mega_evme(&bin, bytecode, *input);
            assert_eq!(r["success"], true, "{name} should succeed");
            assert_eq!(r["output"].as_str().unwrap(), *expected, "{name} output drift");
        }
        // Revert case
        let r = run_mega_evme(&bin, "0x5f5ffd", None);
        assert_eq!(r["success"], false, "revert case should fail");

        // GasProbe: extract bytecode from forge, run through mega-evme,
        // assert the returned gas matches our hardcoded reference (95086).
        prj.wipe_contracts();
        prj.add_test(
            "GasProbe.t.sol",
            r#"
contract GasProbe {
    uint256 public slot0;
    uint256 public slot1;
    function run() external returns (uint256 gasUsed) {
        uint256 g0 = gasleft();
        slot0 = 42; slot1 = 84;
        uint256 x = 1;
        for (uint256 i = 0; i < 100; i++) { x = x * 3 + 1; }
        address(this).staticcall(abi.encodeWithSelector(this.slot0.selector));
        uint256 g1 = gasleft();
        gasUsed = g0 - g1; slot0 = gasUsed;
    }
}
            "#,
        )
        .unwrap();

        let bytecode_out = cmd
            .args(["inspect", "GasProbe", "deployedBytecode"])
            .assert_success()
            .get_output()
            .stdout_lossy();
        let bytecode = bytecode_out.trim();

        let r = run_mega_evme(&bin, bytecode, Some("0xc0406226"));
        assert_eq!(r["success"], true, "GasProbe should succeed");
        let output_hex = r["output"].as_str().unwrap();
        let output_clean = output_hex.trim_start_matches("0x").trim_start_matches('0');
        let mega_gas = u64::from_str_radix(output_clean, 16).expect("bad hex");
        assert_eq!(
            mega_gas, 95086,
            "GasProbe gas drift: mega-evme {MEGA_EVME_VERSION} now reports {mega_gas}, \
             but CrossValidate.t.sol hardcodes 95086. \
             Update MEGA_EVME_REFERENCE_GAS in CrossValidate.t.sol."
        );
    }
);
