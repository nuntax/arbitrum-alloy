alloy_core::sol! {
    /// ArbosTest: deprecated gas-burning precompile at 0x69, still present on every chain.
    ///
    /// Nitro reference: `nitro/precompiles/ArbosTest.go`.
    #[sol(rpc)]
    interface ArbosTest {
        // @notice Unproductively burns the amount of L2 ArbGas
        function burnArbGas(
            uint256 gasAmount
        ) external pure;
    }
}
