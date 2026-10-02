alloy_core::sol! {
    /// ArbWasm: Stylus WASM program management.
    ///
    /// Provides methods to query and manage Stylus (WASM) programs.
    ///
    /// Nitro reference: `nitro/precompiles/ArbWasm.go`.
    #[sol(rpc)]
    interface ArbWasm {
        /// Returns the current Stylus version.
        function stylusVersion() external view returns (uint16);

        /// Returns the ink price (computational cost unit).
        function inkPrice() external view returns (uint32);

        /// Returns the maximum WASM stack depth.
        function maxStackDepth() external view returns (uint32);

        /// Returns the number of free WASM pages.
        function freePages() external view returns (uint16);

        /// Returns the gas cost per WASM page.
        function pageGas() external view returns (uint16);

        /// Returns the page ramp parameter.
        function pageRamp() external view returns (uint64);

        /// Returns the WASM page limit.
        function pageLimit() external view returns (uint16);

        /// Returns the minimum init gas (gas, cachedGas).
        function minInitGas() external view returns (uint64, uint64);

        /// Returns the init cost scalar (percentage).
        function initCostScalar() external view returns (uint64);

        /// Returns the number of days before inactive programs expire.
        function expiryDays() external view returns (uint16);

        /// Returns the number of days a keepalive extends a program.
        function keepaliveDays() external view returns (uint16);

        /// Returns the block cache size.
        function blockCacheSize() external view returns (uint16);

        /// Returns the gas burned up front by every Stylus activation (ArbOS 59).
        function activationGas() external view returns (uint64);

        /// Returns the Stylus version for a given codehash.
        function codehashVersion(bytes32 codehash) external view returns (uint16);

        /// Returns the compiled ASM size for a given codehash.
        function codehashAsmSize(bytes32 codehash) external view returns (uint32);

        /// Returns the Stylus version of a deployed program.
        function programVersion(address program) external view returns (uint16);

        /// Returns the init gas for a deployed program (gas, cachedGas).
        function programInitGas(address program) external view returns (uint64, uint64);

        /// Returns the memory footprint (pages) of a deployed program.
        function programMemoryFootprint(address program) external view returns (uint16);

        /// Returns the remaining time (seconds) before a program expires.
        function programTimeLeft(address program) external view returns (uint64);

        /// Activates a Stylus program. Returns (version, dataFee).
        function activateProgram(address program)
            external
            payable
            returns (uint16, uint256);

        /// Extends the lifetime of a cached codehash.
        function codehashKeepalive(bytes32 codehash) external payable;
    }
}
