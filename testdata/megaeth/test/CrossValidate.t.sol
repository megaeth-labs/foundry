// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";

/// @notice Cross-validate forge --megaeth against mega-evme.
///
/// NOTE: Cheatcodes are silently skipped under --megaeth, so all assertions
/// use `require()` instead of `assertEq()`.
///
/// mega-evme defaults: --gas 10000000, --spec Rex4, --basefee 0
contract CrossValidate is Test {
    /// Deploy raw runtime bytecode via CREATE.
    function _deployRuntime(bytes memory runtime) internal returns (address addr) {
        // Init code layout (11 bytes header + runtime):
        //   PUSH1 <len>   60 xx
        //   DUP1          80
        //   PUSH1 11      60 0b   ← runtime starts at offset 11
        //   PUSH1 0       60 00
        //   CODECOPY      39
        //   PUSH1 0       60 00
        //   RETURN        f3
        //   <runtime bytes...>
        bytes memory creationCode = abi.encodePacked(
            uint8(0x60), uint8(runtime.length),
            uint8(0x80),
            uint8(0x60), uint8(11),
            uint8(0x60), uint8(0),
            uint8(0x39),
            uint8(0x60), uint8(0),
            uint8(0xf3),
            runtime
        );
        assembly {
            addr := create(0, add(creationCode, 0x20), mload(creationCode))
        }
        require(addr != address(0), "deploy failed");
    }

    /// @dev mega-evme reference:
    ///   mega-evme run --json --spec Rex4 0x604260005260206000f3
    ///   → success=true, output=0x...42
    function test_basicReturn() public {
        address target = _deployRuntime(hex"604260005260206000f3");

        (bool ok, bytes memory ret) = target.call("");
        require(ok, "call failed");
        require(ret.length == 32, "unexpected return length");

        uint256 val;
        assembly { val := mload(add(ret, 0x20)) }
        require(val == 0x42, "wrong return value");
    }

    /// @dev mega-evme reference:
    ///   mega-evme run --json --spec Rex4 --input 0xdeadbeef 0x366000600037366000f3
    ///   → success=true, output=0xdeadbeef
    function test_inputEcho() public {
        address target = _deployRuntime(hex"366000600037366000f3");

        (bool ok, bytes memory ret) = target.call(hex"deadbeef");
        require(ok, "call failed");
        require(ret.length == 4, "unexpected return length");
        require(
            keccak256(ret) == keccak256(hex"deadbeef"),
            "wrong echo"
        );
    }

    /// @dev mega-evme reference:
    ///   mega-evme run --json --spec Rex4 0x5f5ffd
    ///   → success=false
    function test_revert() public {
        address target = _deployRuntime(hex"5f5ffd");

        (bool ok, ) = target.call("");
        require(!ok, "should have reverted");
    }

    /// @dev Cross-validate GasProbe self-measurement.
    ///
    ///   mega-evme run --json --spec Rex4 --gas 10000000 --input 0xc0406226 <GasProbe bytecode>
    ///   → output = 0x...1736e = 95086
    ///
    ///   forge test --megaeth trace → GasProbe::run() returns 95086
    ///
    /// Both engines agree on the internal gas measurement. This is the
    /// strongest cross-validation: the contract measures its own gas via
    /// gasleft(), which exercises SSTORE, compute loop, and STATICCALL gas.
    function test_gasProbe_crossValidate() public {
        GasProbe probe = new GasProbe();
        uint256 reported = probe.run();

        // Must match mega-evme's reference value exactly.
        require(reported == 95086, "gasProbe mismatch vs mega-evme");
    }
}

/// @dev Composite gas measurement contract.
/// Exercises SSTORE (cold + warm), compute loop, and STATICCALL.
/// run() returns the gasleft() delta — the same value mega-evme reports
/// when invoked with selector 0xc0406226.
contract GasProbe {
    uint256 public slot0;
    uint256 public slot1;

    function run() external returns (uint256 gasUsed) {
        uint256 g0 = gasleft();
        slot0 = 42;
        slot1 = 84;
        uint256 x = 1;
        for (uint256 i = 0; i < 100; i++) {
            x = x * 3 + 1;
        }
        address(this).staticcall(abi.encodeWithSelector(this.slot0.selector));
        uint256 g1 = gasleft();
        gasUsed = g0 - g1;
        slot0 = gasUsed;
    }
}
