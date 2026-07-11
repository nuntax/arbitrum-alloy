#![allow(missing_docs)]

use alloy_core::sol_types::{SolCall, SolInterface};
use arbitrum_alloy_precompiles::*;

macro_rules! assert_method {
    ($interface:ident, $calls:ident, $call:ident, $signature:literal, $selector:expr) => {{
        type Call = $interface::$call;
        assert_eq!(<Call as SolCall>::SIGNATURE, $signature);
        assert_eq!(<Call as SolCall>::SELECTOR, $selector);
        assert!(<$interface::$calls as SolInterface>::valid_selector(
            <Call as SolCall>::SELECTOR
        ));
    }};
}

#[test]
fn arb_address_table_method_selectors() {
    assert_method!(
        ArbAddressTable,
        ArbAddressTableCalls,
        addressExistsCall,
        "addressExists(address)",
        [0xa5, 0x02, 0x52, 0x22]
    );
    assert_method!(
        ArbAddressTable,
        ArbAddressTableCalls,
        compressCall,
        "compress(address)",
        [0xf6, 0xa4, 0x55, 0xa2]
    );
    assert_method!(
        ArbAddressTable,
        ArbAddressTableCalls,
        lookupCall,
        "lookup(address)",
        [0xd4, 0xb6, 0xb5, 0xda]
    );
    assert_method!(
        ArbAddressTable,
        ArbAddressTableCalls,
        lookupIndexCall,
        "lookupIndex(uint256)",
        [0x8a, 0x18, 0x67, 0x88]
    );
    assert_method!(
        ArbAddressTable,
        ArbAddressTableCalls,
        sizeCall,
        "size()",
        [0x94, 0x9d, 0x22, 0x5d]
    );
    assert_method!(
        ArbAddressTable,
        ArbAddressTableCalls,
        decompressCall,
        "decompress(bytes,uint256)",
        [0x31, 0x86, 0x2a, 0xda]
    );
    assert_method!(
        ArbAddressTable,
        ArbAddressTableCalls,
        registerCall,
        "register(address)",
        [0x44, 0x20, 0xe4, 0x86]
    );
}

#[test]
fn arb_aggregator_method_selectors() {
    assert_method!(
        ArbAggregator,
        ArbAggregatorCalls,
        getPreferredAggregatorCall,
        "getPreferredAggregator(address)",
        [0x52, 0xf1, 0x07, 0x40]
    );
    assert_method!(
        ArbAggregator,
        ArbAggregatorCalls,
        getDefaultAggregatorCall,
        "getDefaultAggregator()",
        [0x87, 0x58, 0x83, 0xf2]
    );
    assert_method!(
        ArbAggregator,
        ArbAggregatorCalls,
        getBatchPostersCall,
        "getBatchPosters()",
        [0xe1, 0x05, 0x73, 0xa3]
    );
    assert_method!(
        ArbAggregator,
        ArbAggregatorCalls,
        getFeeCollectorCall,
        "getFeeCollector(address)",
        [0x9c, 0x2c, 0x5b, 0xb5]
    );
    assert_method!(
        ArbAggregator,
        ArbAggregatorCalls,
        getTxBaseFeeCall,
        "getTxBaseFee(address)",
        [0x04, 0x97, 0x64, 0xaf]
    );
    assert_method!(
        ArbAggregator,
        ArbAggregatorCalls,
        addBatchPosterCall,
        "addBatchPoster(address)",
        [0xdf, 0x41, 0xe1, 0xe2]
    );
    assert_method!(
        ArbAggregator,
        ArbAggregatorCalls,
        setFeeCollectorCall,
        "setFeeCollector(address,address)",
        [0x29, 0x14, 0x97, 0x99]
    );
    assert_method!(
        ArbAggregator,
        ArbAggregatorCalls,
        setTxBaseFeeCall,
        "setTxBaseFee(address,uint256)",
        [0x5b, 0xe6, 0x88, 0x8b]
    );
}

#[test]
fn arb_debug_method_selectors() {
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbDebug,
        ArbDebugCalls,
        eventsCall,
        "events(bool,bytes32)",
        [0x7b, 0x99, 0x63, 0xef]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbDebug,
        ArbDebugCalls,
        eventsViewCall,
        "eventsView()",
        [0x8e, 0x5f, 0x30, 0xab]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbDebug,
        ArbDebugCalls,
        customRevertCall,
        "customRevert(uint64)",
        [0x7e, 0xa8, 0x9f, 0x8b]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbDebug,
        ArbDebugCalls,
        becomeChainOwnerCall,
        "becomeChainOwner()",
        [0x0e, 0x5b, 0xbc, 0x11]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbDebug,
        ArbDebugCalls,
        overwriteContractCodeCall,
        "overwriteContractCode(address,bytes)",
        [0x1b, 0xe2, 0x50, 0xd6]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbDebug,
        ArbDebugCalls,
        panicCall,
        "panic()",
        [0x47, 0x00, 0xd3, 0x05]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbDebug,
        ArbDebugCalls,
        legacyErrorCall,
        "legacyError()",
        [0x1e, 0x48, 0xfe, 0x82]
    );
}

#[test]
fn arb_function_table_method_selectors() {
    assert_method!(
        ArbFunctionTable,
        ArbFunctionTableCalls,
        sizeCall,
        "size(address)",
        [0x88, 0x98, 0x70, 0x68]
    );
    assert_method!(
        ArbFunctionTable,
        ArbFunctionTableCalls,
        uploadCall,
        "upload(bytes)",
        [0xce, 0x2a, 0xe1, 0x59]
    );
    assert_method!(
        ArbFunctionTable,
        ArbFunctionTableCalls,
        getCall,
        "get(address,uint256)",
        [0xb4, 0x64, 0x63, 0x1b]
    );
}

#[test]
fn arb_gas_info_method_selectors() {
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getPricesInWeiWithAggregatorCall,
        "getPricesInWeiWithAggregator(address)",
        [0xba, 0x9c, 0x91, 0x6e]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getPricesInWeiCall,
        "getPricesInWei()",
        [0x41, 0xb2, 0x47, 0xa8]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getPricesInArbGasWithAggregatorCall,
        "getPricesInArbGasWithAggregator(address)",
        [0x7a, 0x1e, 0xa7, 0x32]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getPricesInArbGasCall,
        "getPricesInArbGas()",
        [0x02, 0x19, 0x9f, 0x34]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getGasAccountingParamsCall,
        "getGasAccountingParams()",
        [0x61, 0x2a, 0xf1, 0x78]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getMaxTxGasLimitCall,
        "getMaxTxGasLimit()",
        [0xaa, 0xe1, 0xcd, 0x4c]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getMinimumGasPriceCall,
        "getMinimumGasPrice()",
        [0xf9, 0x18, 0x37, 0x9a]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1BaseFeeEstimateCall,
        "getL1BaseFeeEstimate()",
        [0xf5, 0xd6, 0xde, 0xd7]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1BaseFeeEstimateInertiaCall,
        "getL1BaseFeeEstimateInertia()",
        [0x29, 0xeb, 0x31, 0xee]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1RewardRateCall,
        "getL1RewardRate()",
        [0x8a, 0x5b, 0x1d, 0x28]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1RewardRecipientCall,
        "getL1RewardRecipient()",
        [0x9e, 0x6d, 0x7e, 0x31]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1GasPriceEstimateCall,
        "getL1GasPriceEstimate()",
        [0x05, 0x5f, 0x36, 0x2f]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getCurrentTxL1GasFeesCall,
        "getCurrentTxL1GasFees()",
        [0xc6, 0xf7, 0xde, 0x0e]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getGasBacklogCall,
        "getGasBacklog()",
        [0x1d, 0x5b, 0x5c, 0x20]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getPricingInertiaCall,
        "getPricingInertia()",
        [0x3d, 0xfb, 0x45, 0xb9]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getGasBacklogToleranceCall,
        "getGasBacklogTolerance()",
        [0x25, 0x75, 0x4f, 0x91]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1PricingSurplusCall,
        "getL1PricingSurplus()",
        [0x52, 0x0a, 0xcd, 0xd7]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getPerBatchGasChargeCall,
        "getPerBatchGasCharge()",
        [0x6e, 0xcc, 0xa4, 0x5a]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getAmortizedCostCapBipsCall,
        "getAmortizedCostCapBips()",
        [0x7a, 0x7d, 0x6b, 0xeb]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1FeesAvailableCall,
        "getL1FeesAvailable()",
        [0x5b, 0x39, 0xd2, 0x3c]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1PricingEquilibrationUnitsCall,
        "getL1PricingEquilibrationUnits()",
        [0xad, 0x26, 0xce, 0x90]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getLastL1PricingUpdateTimeCall,
        "getLastL1PricingUpdateTime()",
        [0x13, 0x8b, 0x47, 0xb4]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1PricingFundsDueForRewardsCall,
        "getL1PricingFundsDueForRewards()",
        [0x96, 0x3d, 0x60, 0x02]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getL1PricingUnitsSinceUpdateCall,
        "getL1PricingUnitsSinceUpdate()",
        [0xef, 0xf0, 0x13, 0x06]
    );
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getLastL1PricingSurplusCall,
        "getLastL1PricingSurplus()",
        [0x29, 0x87, 0xd0, 0x27]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbGasInfo,
        ArbGasInfoCalls,
        getMaxBlockGasLimitCall,
        "getMaxBlockGasLimit()",
        [0x03, 0x71, 0xfd, 0xb4]
    );
}

#[test]
fn arb_info_method_selectors() {
    assert_method!(
        ArbInfo,
        ArbInfoCalls,
        getBalanceCall,
        "getBalance(address)",
        [0xf8, 0xb2, 0xcb, 0x4f]
    );
    assert_method!(
        ArbInfo,
        ArbInfoCalls,
        getCodeCall,
        "getCode(address)",
        [0x7e, 0x10, 0x5c, 0xe2]
    );
}

#[test]
fn arb_owner_method_selectors() {
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        addChainOwnerCall,
        "addChainOwner(address)",
        [0x48, 0x1f, 0x8d, 0xbf]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        removeChainOwnerCall,
        "removeChainOwner(address)",
        [0x87, 0x92, 0x70, 0x1a]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        addNativeTokenOwnerCall,
        "addNativeTokenOwner(address)",
        [0xae, 0xb3, 0xa4, 0x64]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        removeNativeTokenOwnerCall,
        "removeNativeTokenOwner(address)",
        [0x96, 0xa3, 0x75, 0x1d]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setNativeTokenManagementFromCall,
        "setNativeTokenManagementFrom(uint64)",
        [0xbd, 0xb8, 0xf7, 0x07]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setTransactionFilteringFromCall,
        "setTransactionFilteringFrom(uint64)",
        [0x46, 0x06, 0x6e, 0x45]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        addTransactionFiltererCall,
        "addTransactionFilterer(address)",
        [0x59, 0xc8, 0x7a, 0xcc]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        removeTransactionFiltererCall,
        "removeTransactionFilterer(address)",
        [0x67, 0xad, 0xa0, 0x89]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setFilteredFundsRecipientCall,
        "setFilteredFundsRecipient(address)",
        [0xb7, 0x9d, 0xa0, 0xe9]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setNetworkFeeAccountCall,
        "setNetworkFeeAccount(address)",
        [0xfc, 0xdd, 0xe2, 0xb4]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setInfraFeeAccountCall,
        "setInfraFeeAccount(address)",
        [0x57, 0xf5, 0x85, 0xdb]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setL2BaseFeeCall,
        "setL2BaseFee(uint256)",
        [0xd9, 0x9b, 0xc8, 0x0e]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setMinimumL2BaseFeeCall,
        "setMinimumL2BaseFee(uint256)",
        [0xa0, 0x18, 0x8c, 0xdb]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setSpeedLimitCall,
        "setSpeedLimit(uint64)",
        [0x4d, 0x7a, 0x06, 0x0d]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setMaxTxGasLimitCall,
        "setMaxTxGasLimit(uint64)",
        [0x39, 0x67, 0x36, 0x11]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setMaxBlockGasLimitCall,
        "setMaxBlockGasLimit(uint64)",
        [0xae, 0x10, 0x5c, 0x80]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setL2GasPricingInertiaCall,
        "setL2GasPricingInertia(uint64)",
        [0x3f, 0xd6, 0x2a, 0x29]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setL2GasBacklogToleranceCall,
        "setL2GasBacklogTolerance(uint64)",
        [0x19, 0x8e, 0x71, 0x57]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setGasBacklogCall,
        "setGasBacklog(uint64)",
        [0x68, 0xfc, 0x80, 0x8a]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setL1BaseFeeEstimateInertiaCall,
        "setL1BaseFeeEstimateInertia(uint64)",
        [0x71, 0x8f, 0x78, 0x05]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setL1PricingEquilibrationUnitsCall,
        "setL1PricingEquilibrationUnits(uint256)",
        [0x15, 0x2d, 0xb6, 0x96]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setL1PricingInertiaCall,
        "setL1PricingInertia(uint64)",
        [0x77, 0x5a, 0x82, 0xe9]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setL1PricingRewardRecipientCall,
        "setL1PricingRewardRecipient(address)",
        [0x93, 0x4b, 0xe0, 0x7d]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setL1PricingRewardRateCall,
        "setL1PricingRewardRate(uint64)",
        [0xf6, 0x73, 0x95, 0x00]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setL1PricePerUnitCall,
        "setL1PricePerUnit(uint256)",
        [0x2b, 0x35, 0x2f, 0xae]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setParentGasFloorPerTokenCall,
        "setParentGasFloorPerToken(uint64)",
        [0x3a, 0x93, 0x0b, 0x0b]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setPerBatchGasChargeCall,
        "setPerBatchGasCharge(int64)",
        [0xfa, 0xd7, 0xf2, 0x0b]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setAmortizedCostCapBipsCall,
        "setAmortizedCostCapBips(uint64)",
        [0x56, 0x19, 0x1c, 0xc3]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        releaseL1PricerSurplusFundsCall,
        "releaseL1PricerSurplusFunds(uint256)",
        [0x31, 0x4b, 0xcf, 0x05]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setBrotliCompressionLevelCall,
        "setBrotliCompressionLevel(uint64)",
        [0x53, 0x99, 0x12, 0x6f]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setCalldataPriceIncreaseCall,
        "setCalldataPriceIncrease(bool)",
        [0x8e, 0xb9, 0x11, 0xd9]
    );
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        scheduleArbOSUpgradeCall,
        "scheduleArbOSUpgrade(uint64,uint64)",
        [0xe3, 0x88, 0xb3, 0x81]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setChainConfigCall,
        "setChainConfig(bytes)",
        [0x36, 0x88, 0xab, 0xea]
    );
    // ArbOS 50+ multi-constraint gas pricing (Nitro ArbOwner.SetGasPricingConstraints).
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setGasPricingConstraintsCall,
        "setGasPricingConstraints(uint64[3][])",
        [0xcc, 0x0d, 0x55, 0x6a]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setInkPriceCall,
        "setInkPrice(uint32)",
        [0x8c, 0x1d, 0x4f, 0xda]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmMaxStackDepthCall,
        "setWasmMaxStackDepth(uint32)",
        [0x45, 0x67, 0xcc, 0x8e]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmFreePagesCall,
        "setWasmFreePages(uint16)",
        [0x3f, 0x37, 0xa8, 0x46]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmPageGasCall,
        "setWasmPageGas(uint16)",
        [0xaa, 0xa6, 0x19, 0xe0]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmPageLimitCall,
        "setWasmPageLimit(uint16)",
        [0x65, 0x95, 0x38, 0x1a]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmMinInitGasCall,
        "setWasmMinInitGas(uint64,uint64)",
        [0x80, 0xa3, 0xa5, 0xe4]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmInitCostScalarCall,
        "setWasmInitCostScalar(uint64)",
        [0x67, 0xe0, 0x71, 0x8f]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmExpiryDaysCall,
        "setWasmExpiryDays(uint16)",
        [0xaa, 0xc6, 0x80, 0x18]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmKeepaliveDaysCall,
        "setWasmKeepaliveDays(uint16)",
        [0x2a, 0x9c, 0xbe, 0x3e]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmBlockCacheSizeCall,
        "setWasmBlockCacheSize(uint16)",
        [0x38, 0x0f, 0x14, 0x57]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setWasmMaxSizeCall,
        "setWasmMaxSize(uint32)",
        [0x45, 0x5e, 0xc2, 0xeb]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        addWasmCacheManagerCall,
        "addWasmCacheManager(address)",
        [0xff, 0xdc, 0xa5, 0x15]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        removeWasmCacheManagerCall,
        "removeWasmCacheManager(address)",
        [0xbf, 0x19, 0x73, 0x22]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwner,
        ArbOwnerCalls,
        setMaxStylusContractFragmentsCall,
        "setMaxStylusContractFragments(uint8)",
        [0xf1, 0xfe, 0x1a, 0x70]
    );
}

#[test]
fn arb_owner_public_method_selectors() {
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getAllChainOwnersCall,
        "getAllChainOwners()",
        [0x51, 0x6b, 0x4e, 0x0f]
    );
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        isChainOwnerCall,
        "isChainOwner(address)",
        [0x26, 0xef, 0x7f, 0x68]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        isNativeTokenOwnerCall,
        "isNativeTokenOwner(address)",
        [0xc6, 0x86, 0xf4, 0xdb]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getAllNativeTokenOwnersCall,
        "getAllNativeTokenOwners()",
        [0x3f, 0x86, 0x01, 0xe4]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getNativeTokenManagementFromCall,
        "getNativeTokenManagementFrom()",
        [0x3f, 0xec, 0xba, 0xb0]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getTransactionFilteringFromCall,
        "getTransactionFilteringFrom()",
        [0xc1, 0xd3, 0x55, 0xb8]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        isTransactionFiltererCall,
        "isTransactionFilterer(address)",
        [0xb3, 0x23, 0x52, 0xc3]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getAllTransactionFilterersCall,
        "getAllTransactionFilterers()",
        [0x59, 0x5f, 0xbb, 0x5a]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getFilteredFundsRecipientCall,
        "getFilteredFundsRecipient()",
        [0x3c, 0xaa, 0x5f, 0x12]
    );
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getNetworkFeeAccountCall,
        "getNetworkFeeAccount()",
        [0x2d, 0x91, 0x25, 0xe9]
    );
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getInfraFeeAccountCall,
        "getInfraFeeAccount()",
        [0xee, 0x95, 0xa8, 0x24]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getBrotliCompressionLevelCall,
        "getBrotliCompressionLevel()",
        [0x22, 0xd4, 0x99, 0xc7]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getScheduledUpgradeCall,
        "getScheduledUpgrade()",
        [0x81, 0xef, 0x94, 0x4c]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        isCalldataPriceIncreaseEnabledCall,
        "isCalldataPriceIncreaseEnabled()",
        [0x2a, 0xa9, 0x55, 0x1e]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getParentGasFloorPerTokenCall,
        "getParentGasFloorPerToken()",
        [0x49, 0xcc, 0xda, 0xff]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        getMaxStylusContractFragmentsCall,
        "getMaxStylusContractFragments()",
        [0xe5, 0xa7, 0xf8, 0x93]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        ArbOwnerPublic,
        ArbOwnerPublicCalls,
        rectifyChainOwnerCall,
        "rectifyChainOwner(address)",
        [0x6f, 0xe8, 0x63, 0x73]
    );
}

#[test]
fn arb_retryable_tx_method_selectors() {
    assert_method!(
        ArbRetryableTx,
        ArbRetryableTxCalls,
        getLifetimeCall,
        "getLifetime()",
        [0x81, 0xe6, 0xe0, 0x83]
    );
    assert_method!(
        ArbRetryableTx,
        ArbRetryableTxCalls,
        getTimeoutCall,
        "getTimeout(bytes32)",
        [0x9f, 0x10, 0x25, 0xc6]
    );
    assert_method!(
        ArbRetryableTx,
        ArbRetryableTxCalls,
        getBeneficiaryCall,
        "getBeneficiary(bytes32)",
        [0xba, 0x20, 0xdd, 0xa4]
    );
    assert_method!(
        ArbRetryableTx,
        ArbRetryableTxCalls,
        getCurrentRedeemerCall,
        "getCurrentRedeemer()",
        [0xde, 0x4b, 0xa2, 0xb3]
    );
    assert_method!(
        ArbRetryableTx,
        ArbRetryableTxCalls,
        redeemCall,
        "redeem(bytes32)",
        [0xed, 0xa1, 0x12, 0x2c]
    );
    assert_method!(
        ArbRetryableTx,
        ArbRetryableTxCalls,
        keepaliveCall,
        "keepalive(bytes32)",
        [0xf0, 0xb2, 0x1a, 0x41]
    );
    assert_method!(
        ArbRetryableTx,
        ArbRetryableTxCalls,
        cancelCall,
        "cancel(bytes32)",
        [0xc4, 0xd2, 0x52, 0xf5]
    );
}

#[test]
fn arb_statistics_method_selectors() {
    assert_method!(
        ArbStatistics,
        ArbStatisticsCalls,
        getStatsCall,
        "getStats()",
        [0xc5, 0x9d, 0x48, 0x47]
    );
}

#[test]
fn arb_sys_method_selectors() {
    assert_method!(
        ArbSys,
        ArbSysCalls,
        arbBlockNumberCall,
        "arbBlockNumber()",
        [0xa3, 0xb1, 0xb3, 0x1d]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        arbBlockHashCall,
        "arbBlockHash(uint256)",
        [0x2b, 0x40, 0x7a, 0x82]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        arbChainIDCall,
        "arbChainID()",
        [0xd1, 0x27, 0xf5, 0x4a]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        arbOSVersionCall,
        "arbOSVersion()",
        [0x05, 0x10, 0x38, 0xf2]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        getStorageGasAvailableCall,
        "getStorageGasAvailable()",
        [0xa9, 0x45, 0x97, 0xff]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        isTopLevelCallCall,
        "isTopLevelCall()",
        [0x08, 0xbd, 0x62, 0x4c]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        mapL1SenderContractAddressToL2AliasCall,
        "mapL1SenderContractAddressToL2Alias(address,address)",
        [0x4d, 0xbb, 0xd5, 0x06]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        wasMyCallersAddressAliasedCall,
        "wasMyCallersAddressAliased()",
        [0x17, 0x5a, 0x26, 0x0b]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        myCallersAddressWithoutAliasingCall,
        "myCallersAddressWithoutAliasing()",
        [0xd7, 0x45, 0x23, 0xb3]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        sendMerkleTreeStateCall,
        "sendMerkleTreeState()",
        [0x7a, 0xee, 0xcd, 0x2a]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        sendTxToL1Call,
        "sendTxToL1(address,bytes)",
        [0x92, 0x8c, 0x16, 0x9a]
    );
    assert_method!(
        ArbSys,
        ArbSysCalls,
        withdrawEthCall,
        "withdrawEth(address)",
        [0x25, 0xe1, 0x60, 0x63]
    );
}

#[test]
fn arb_wasm_method_selectors() {
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        stylusVersionCall,
        "stylusVersion()",
        [0xa9, 0x96, 0xe0, 0xc2]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        inkPriceCall,
        "inkPrice()",
        [0xd1, 0xc1, 0x7a, 0xbc]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        maxStackDepthCall,
        "maxStackDepth()",
        [0x8c, 0xcf, 0xaa, 0x70]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        freePagesCall,
        "freePages()",
        [0x44, 0x90, 0xc1, 0x9d]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        pageGasCall,
        "pageGas()",
        [0x7a, 0xf4, 0xba, 0x49]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        pageRampCall,
        "pageRamp()",
        [0x11, 0xc8, 0x2a, 0xe8]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        pageLimitCall,
        "pageLimit()",
        [0x97, 0x86, 0xf9, 0x6e]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        minInitGasCall,
        "minInitGas()",
        [0x99, 0xd0, 0xb3, 0x8d]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        initCostScalarCall,
        "initCostScalar()",
        [0x5f, 0xc9, 0x4c, 0x0b]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        expiryDaysCall,
        "expiryDays()",
        [0x30, 0x9f, 0x65, 0x55]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        keepaliveDaysCall,
        "keepaliveDays()",
        [0x0a, 0x93, 0x64, 0x55]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        blockCacheSizeCall,
        "blockCacheSize()",
        [0x7a, 0xf6, 0xe8, 0x19]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        codehashVersionCall,
        "codehashVersion(bytes32)",
        [0xd7, 0x0c, 0x0c, 0xa7]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        codehashAsmSizeCall,
        "codehashAsmSize(bytes32)",
        [0x40, 0x89, 0x26, 0x7f]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        programVersionCall,
        "programVersion(address)",
        [0xcc, 0x8f, 0x4e, 0x88]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        programInitGasCall,
        "programInitGas(address)",
        [0x62, 0xb6, 0x88, 0xaa]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        programMemoryFootprintCall,
        "programMemoryFootprint(address)",
        [0xae, 0xf3, 0x6b, 0xe3]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        programTimeLeftCall,
        "programTimeLeft(address)",
        [0xc7, 0x75, 0xa6, 0x2a]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        activateProgramCall,
        "activateProgram(address)",
        [0x58, 0xc7, 0x80, 0xc2]
    );
    assert_method!(
        ArbWasm,
        ArbWasmCalls,
        codehashKeepaliveCall,
        "codehashKeepalive(bytes32)",
        [0xc6, 0x89, 0xba, 0xd5]
    );
}

#[test]
fn arb_wasm_cache_method_selectors() {
    assert_method!(
        ArbWasmCache,
        ArbWasmCacheCalls,
        isCacheManagerCall,
        "isCacheManager(address)",
        [0x85, 0xe2, 0xde, 0x85]
    );
    assert_method!(
        ArbWasmCache,
        ArbWasmCacheCalls,
        allCacheManagersCall,
        "allCacheManagers()",
        [0x0e, 0xc1, 0xd7, 0x73]
    );
    assert_method!(
        ArbWasmCache,
        ArbWasmCacheCalls,
        codehashIsCachedCall,
        "codehashIsCached(bytes32)",
        [0xa7, 0x2f, 0x17, 0x9b]
    );
    assert_method!(
        ArbWasmCache,
        ArbWasmCacheCalls,
        cacheProgramCall,
        "cacheProgram(address)",
        [0xe7, 0x3a, 0xc9, 0xf2]
    );
    assert_method!(
        ArbWasmCache,
        ArbWasmCacheCalls,
        evictCodehashCall,
        "evictCodehash(bytes32)",
        [0xce, 0x97, 0x20, 0x13]
    );
}

#[test]
fn arbos_acts_method_selectors() {
    assert_method!(
        ArbosActs,
        ArbosActsCalls,
        startBlockCall,
        "startBlock(uint256,uint64,uint64,uint64)",
        [0x6b, 0xf6, 0xa4, 0x2d]
    );
    assert_method!(
        ArbosActs,
        ArbosActsCalls,
        batchPostingReportCall,
        "batchPostingReport(uint256,address,uint64,uint64,uint256)",
        [0xb6, 0x69, 0x37, 0x71]
    );
    assert_method!(
        ArbosActs,
        ArbosActsCalls,
        batchPostingReportV2Call,
        "batchPostingReportV2(uint256,address,uint64,uint64,uint64,uint64,uint256)",
        [0x99, 0x98, 0x26, 0x9e]
    );
}

#[test]
fn node_interface_method_selectors() {
    assert_method!(
        NodeInterface,
        NodeInterfaceCalls,
        nitroGenesisBlockCall,
        "nitroGenesisBlock()",
        [0x93, 0xa2, 0xfe, 0x21]
    );
    assert_method!(
        NodeInterface,
        NodeInterfaceCalls,
        findBatchContainingBlockCall,
        "findBatchContainingBlock(uint64)",
        [0x81, 0xf1, 0xad, 0xaf]
    );
    assert_method!(
        NodeInterface,
        NodeInterfaceCalls,
        getL1ConfirmationsCall,
        "getL1Confirmations(bytes32)",
        [0xe5, 0xca, 0x23, 0x8c]
    );
    assert_method!(
        NodeInterface,
        NodeInterfaceCalls,
        estimateRetryableTicketCall,
        "estimateRetryableTicket(address,uint256,address,uint256,address,address,bytes)",
        [0xc3, 0xdc, 0x58, 0x79]
    );
    assert_method!(
        NodeInterface,
        NodeInterfaceCalls,
        gasEstimateL1ComponentCall,
        "gasEstimateL1Component(address,bool,bytes)",
        [0x77, 0xd4, 0x88, 0xa2]
    );
    assert_method!(
        NodeInterface,
        NodeInterfaceCalls,
        gasEstimateComponentsCall,
        "gasEstimateComponents(address,bool,bytes)",
        [0xc9, 0x4e, 0x6e, 0xeb]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        NodeInterface,
        NodeInterfaceCalls,
        l2BlockRangeForL1Call,
        "l2BlockRangeForL1(uint64)",
        [0x48, 0xe7, 0xf8, 0x11]
    );
    assert_method!(
        NodeInterface,
        NodeInterfaceCalls,
        constructOutboxProofCall,
        "constructOutboxProof(uint64,uint64)",
        [0x42, 0x69, 0x63, 0x50]
    );
    // Not present in Arbiscan selector TSV for this exact signature.
    assert_method!(
        NodeInterface,
        NodeInterfaceCalls,
        blockL1NumCall,
        "blockL1Num(uint64)",
        [0x6f, 0x27, 0x5e, 0xf2]
    );
}
