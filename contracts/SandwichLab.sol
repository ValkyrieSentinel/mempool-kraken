// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

/**
 * @title SandwichLab
 * @notice A sandbox for simulating and studying the mechanics of sandwich attacks on a DEX.
 * @dev This contract is NOT intended for production. Used exclusively for security research.
 */
contract SandwichLab {
    
    event LiquidityAdded(uint reserveA, uint reserveB);
    event FrontRun(address attacker, uint amountIn, uint amountOut);
    event VictimSwap(address victim, uint amountIn, uint amountOut, uint priceImpact);
    event BackRun(address attacker, uint amountIn, uint amountOut, uint profit);

   
    struct Token {
        mapping(address => uint) balances;
        uint totalSupply;
    }

    Token public tokenA; // e.g., WETH
    Token public tokenB; // e.g., PEPE

   
    uint public reserveA;
    uint public reserveB;

    constructor() {
       
        tokenA.totalSupply = 1000 ether;
        tokenB.totalSupply = 1000000 ether;
        
        tokenA.balances[address(this)] = 1000 ether;
        tokenB.balances[address(this)] = 1000000 ether;
        
        reserveA = 1000 ether;
        reserveB = 1000000 ether;
        
        emit LiquidityAdded(reserveA, reserveB);
    }

    
    receive() external payable {}

    /**
     * @dev Simulates a full sandwich attack cycle
     * @param victimAddress The address of the "victim"
     * @param victimAmountIn The amount of tokens the victim intends to swap
     */
    function simulateSandwich(address victimAddress, uint victimAmountIn) external {
        require(victimAmountIn > 0, "Invalid amount");

        
        uint attackerFrontRunAmount = 50 ether; 
        uint victimGetsBefore = getAmountOut(victimAmountIn, reserveA, reserveB);
        
        _swap(attackerFrontRunAmount, true); 
        emit FrontRun(msg.sender, attackerFrontRunAmount, getAmountOut(attackerFrontRunAmount, reserveB, reserveA));

        
        uint victimGetsAfter = getAmountOut(victimAmountIn, reserveA, reserveB);
        uint priceImpact = ((victimGetsBefore - victimGetsAfter) * 10000) / victimGetsBefore; 
        
        _swap(victimAmountIn, true); 
        emit VictimSwap(victimAddress, victimAmountIn, victimGetsAfter, priceImpact);

    
        uint attackerBackRunAmount = getAmountOut(attackerFrontRunAmount, reserveA, reserveB); 
        
        _swap(attackerBackRunAmount, false); 
        uint profit = getAmountOut(attackerBackRunAmount, reserveB, reserveA) - attackerFrontRunAmount;
        
        emit BackRun(msg.sender, attackerBackRunAmount, getAmountOut(attackerBackRunAmount, reserveB, reserveA), profit);
    }

    

    function _swap(uint amountIn, bool isTokenAIn) internal {
        uint reserveIn = isTokenAIn ? reserveA : reserveB;
        uint reserveOut = isTokenAIn ? reserveB : reserveA;
        
        uint amountOut = getAmountOut(amountIn, reserveIn, reserveOut);
        require(amountOut > 0, "Insufficient output amount");

        if (isTokenAIn) {
            reserveA += amountIn;
            reserveB -= amountOut;
        } else {
            reserveB += amountIn;
            reserveA -= amountOut;
        }
    }

    function getAmountOut(uint amountIn, uint reserveIn, uint reserveOut) public pure returns (uint) {
        require(amountIn > 0, "Insufficient input amount");
        require(reserveIn > 0 && reserveOut > 0, "Insufficient liquidity");
        
        // Uniswap V2 formula  amountOut = (amountIn * 997 * reserveOut) / (reserveIn * 1000 + amountIn * 997)
        uint amountInWithFee = amountIn * 997;
        uint numerator = amountInWithFee * reserveOut;
        uint denominator = (reserveIn * 1000) + amountInWithFee;
        
        return numerator / denominator;
    }


    function balanceOfA(address account) public view returns (uint) {
        return tokenA.balances[account];
    }
    
    function balanceOfB(address account) public view returns (uint) {
        return tokenB.balances[account];
    }
}