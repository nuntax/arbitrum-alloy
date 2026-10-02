alloy_core::sol! {
    /// ArbNativeTokenManager: native token mint/burn for authorised owners (ArbOS 41).
    ///
    /// Nitro reference: `nitro/precompiles/ArbNativeTokenManager.go`.
    #[sol(rpc)]
    interface ArbNativeTokenManager {
        /*
         * @notice Emitted when some amount of the native gas token is minted to a NativeTokenOwner.
         */
        event NativeTokenMinted(address indexed to, uint256 amount);

        /*
         * @notice Emitted when some amount of the native gas token is burned from a NativeTokenOwner.
         */
        event NativeTokenBurned(address indexed from, uint256 amount);

        /*
         * @notice In case the caller is authorized,
         * mints some amount of the native gas token for this chain to the caller.
         */
        function mintNativeToken(
            uint256 amount
        ) external;

        /*
         * @notice In case the caller is authorized,
         * burns some amount of the native gas token for this chain from the caller.
         */
        function burnNativeToken(
            uint256 amount
        ) external;
    }
}
