// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";

contract GasReporter {
    uint256 public storedGas;

    function reportGas() external returns (uint256) {
        storedGas = gasleft();
        return storedGas;
    }

    function recurse(uint256 depth) external returns (uint256) {
        if (gasleft() < 10000) return depth;
        try this.recurse(depth + 1) returns (uint256 d) {
            return d;
        } catch {
            return depth;
        }
    }

    function chainCall(address next) external returns (uint256 myGas, uint256 childGas) {
        myGas = gasleft();
        childGas = GasReporter(next).reportGas();
    }
}

// Gas forwarding ratio: Ethereum 63/64 ≈ 98.44% vs MegaETH 98/100 = 98.00%
contract Case1_GasForwarding is Test {
    GasReporter reporter;

    function setUp() public {
        reporter = new GasReporter();
    }

    function test_gasForwardingRatio() public {
        uint256 parentGas = gasleft();
        uint256 childGas = reporter.reportGas();
        uint256 ratioBps = (childGas * 10000) / parentGas;
        assembly { sstore(0xff, ratioBps) }
    }
}

// Recursive depth: different gas retention → different max depth
contract Case2_RecursionDepth is Test {
    GasReporter reporter;

    function setUp() public {
        reporter = new GasReporter();
    }

    function test_maxRecursionDepth() public {
        uint256 depth = reporter.recurse(0);
        assembly { sstore(0xff, depth) }
    }
}

// Chained calls: two-hop gas decay amplifies the forwarding difference
contract Case3_ChainedCalls is Test {
    GasReporter a;
    GasReporter b;
    GasReporter c;

    function setUp() public {
        a = new GasReporter();
        b = new GasReporter();
        c = new GasReporter();
    }

    function test_twoHopGasDecay() public {
        uint256 directGas = c.reportGas();
        (, uint256 twoHopGas) = a.chainCall(address(b));
        assembly {
            sstore(0xfe, directGas)
            sstore(0xff, twoHopGas)
        }
    }
}

// Basic functionality without cheatcodes
contract SimpleToken {
    mapping(address => uint256) public balanceOf;
    uint256 public totalSupply;

    function mint(address to, uint256 amount) external {
        balanceOf[to] += amount;
        totalSupply += amount;
    }

    function transfer(address to, uint256 amount) external {
        require(balanceOf[msg.sender] >= amount, "insufficient balance");
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
    }
}

contract Case4_BasicFunctionality is Test {
    SimpleToken token;

    function setUp() public {
        token = new SimpleToken();
        token.mint(address(this), 1000);
    }

    function test_mint() public view {
        assertEq(token.balanceOf(address(this)), 1000);
        assertEq(token.totalSupply(), 1000);
    }

    function test_transfer() public {
        address bob = address(0xB0B);
        token.transfer(bob, 300);
        assertEq(token.balanceOf(address(this)), 700);
        assertEq(token.balanceOf(bob), 300);
    }

    function test_transferInsufficientBalance() public {
        address bob = address(0xB0B);
        try token.transfer(bob, 2000) {
            fail();
        } catch {}
    }
}

// Cross-validation with mega-evme: run() returns internal gasUsed
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

contract Case5_CrossValidation is Test {
    GasProbe probe;

    function setUp() public {
        probe = new GasProbe();
    }

    function test_crossValidate() public {
        uint256 gasUsed = probe.run();
        assembly { sstore(0xff, gasUsed) }
    }
}
