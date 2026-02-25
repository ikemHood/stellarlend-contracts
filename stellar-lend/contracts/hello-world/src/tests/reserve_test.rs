//! # Reserve and Treasury Tests
//!
//! Comprehensive test suite for the reserve and treasury module.
//!
//! ## Test Coverage
//! - Reserve factor configuration (set, get, bounds validation)
//! - Reserve accrual from interest payments
//! - Treasury address management
//! - Treasury withdrawals (success and failure cases)
//! - Authorization checks (admin-only operations)
//! - Edge cases (zero amounts, maximum values, boundary conditions)
//! - Security validations (user fund protection, overflow prevention)
//!
//! ## Security Assumptions
//! 1. Only admin can modify reserve factors
//! 2. Only admin can withdraw reserves
//! 3. Reserve factor is capped at 50% (5000 bps)
//! 4. Withdrawals cannot exceed accrued reserves
//! 5. User funds are never accessible via treasury operations
//! 6. All arithmetic uses checked operations to prevent overflow
//! 7. Treasury address cannot be the contract itself

#![cfg(test)]

use crate::deposit::DepositDataKey;
use crate::reserve::{
    accrue_reserve, get_reserve_balance, get_reserve_factor, get_reserve_stats,
    get_treasury_address, initialize_reserve_config, set_reserve_factor, set_treasury_address,
    withdraw_reserve_to_treasury, ReserveError, BASIS_POINTS_SCALE, DEFAULT_RESERVE_FACTOR_BPS,
    MAX_RESERVE_FACTOR_BPS,
};
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Helper function to create a test environment with an admin
fn setup_test_env() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, crate::HelloContract);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let treasury = Address::generate(&env);

    // Set admin in storage (wrapped in as_contract)
    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&DepositDataKey::Admin, &admin);
    });

    (env, contract_id, admin, user, treasury)
}

// Helper wrappers that handle as_contract internally
fn test_initialize_reserve_config(
    env: &Env,
    contract_id: &Address,
    asset: Option<Address>,
    reserve_factor_bps: i128,
) -> Result<(), ReserveError> {
    env.as_contract(contract_id, || {
        initialize_reserve_config(env, asset, reserve_factor_bps)
    })
}

fn test_get_reserve_factor(env: &Env, contract_id: &Address, asset: Option<Address>) -> i128 {
    env.as_contract(contract_id, || get_reserve_factor(env, asset))
}

fn test_get_reserve_balance(env: &Env, contract_id: &Address, asset: Option<Address>) -> i128 {
    env.as_contract(contract_id, || get_reserve_balance(env, asset))
}

fn test_set_reserve_factor(
    env: &Env,
    contract_id: &Address,
    caller: Address,
    asset: Option<Address>,
    reserve_factor_bps: i128,
) -> Result<(), ReserveError> {
    env.as_contract(contract_id, || {
        set_reserve_factor(env, caller, asset, reserve_factor_bps)
    })
}

fn test_accrue_reserve(
    env: &Env,
    contract_id: &Address,
    asset: Option<Address>,
    interest_amount: i128,
) -> Result<(i128, i128), ReserveError> {
    env.as_contract(contract_id, || accrue_reserve(env, asset, interest_amount))
}

fn test_set_treasury_address(
    env: &Env,
    contract_id: &Address,
    caller: Address,
    treasury: Address,
) -> Result<(), ReserveError> {
    env.as_contract(contract_id, || set_treasury_address(env, caller, treasury))
}

fn test_get_treasury_address(env: &Env, contract_id: &Address) -> Option<Address> {
    env.as_contract(contract_id, || get_treasury_address(env))
}

fn test_withdraw_reserve_to_treasury(
    env: &Env,
    contract_id: &Address,
    caller: Address,
    asset: Option<Address>,
    amount: i128,
) -> Result<i128, ReserveError> {
    env.as_contract(contract_id, || {
        withdraw_reserve_to_treasury(env, caller, asset, amount)
    })
}

fn test_get_reserve_stats(
    env: &Env,
    contract_id: &Address,
    asset: Option<Address>,
) -> (i128, i128, Option<Address>) {
    env.as_contract(contract_id, || get_reserve_stats(env, asset))
}

// ============================================================================
// Initialization Tests
// ============================================================================

#[test]
fn test_initialize_reserve_config_success() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with default reserve factor
    let result = test_initialize_reserve_config(&env, &contract_id, &contract_id,contract_id,contract_id, asset.clone(), DEFAULT_RESERVE_FACTOR_BPS);
    assert!(result.is_ok());

    // Verify reserve factor is set
    let factor = test_get_reserve_factor(&env, &contract_id, &contract_id,contract_id,contract_id, asset.clone());
    assert_eq!(factor, DEFAULT_RESERVE_FACTOR_BPS);

    // Verify reserve balance is initialized to zero
    let balance = test_get_reserve_balance(&env, &contract_id, &contract_id,contract_id,contract_id, asset);
    assert_eq!(balance, 0);
}

#[test]
fn test_initialize_reserve_config_custom_factor() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with custom reserve factor (20%)
    let custom_factor = 2000i128;
    let result = test_initialize_reserve_config(&env, &contract_id, asset.clone(), custom_factor);
    assert!(result.is_ok());

    // Verify custom factor is set
    let factor = test_get_reserve_factor(&env, &contract_id, asset);
    assert_eq!(factor, custom_factor);
}

#[test]
fn test_initialize_reserve_config_zero_factor() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with zero reserve factor (0%)
    let result = test_initialize_reserve_config(&env, &contract_id, asset.clone(), 0);
    assert!(result.is_ok());

    let factor = test_get_reserve_factor(&env, &contract_id, asset);
    assert_eq!(factor, 0);
}

#[test]
fn test_initialize_reserve_config_max_factor() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with maximum reserve factor (50%)
    let result = test_initialize_reserve_config(&env, &contract_id, asset.clone(), MAX_RESERVE_FACTOR_BPS);
    assert!(result.is_ok());

    let factor = test_get_reserve_factor(&env, &contract_id, asset);
    assert_eq!(factor, MAX_RESERVE_FACTOR_BPS);
}

#[test]
fn test_initialize_reserve_config_exceeds_max() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Try to initialize with reserve factor > 50%
    let result = test_initialize_reserve_config(&env, &contract_id, asset, MAX_RESERVE_FACTOR_BPS + 1);
    assert_eq!(result, Err(ReserveError::InvalidReserveFactor));
}

#[test]
fn test_initialize_reserve_config_negative_factor() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Try to initialize with negative reserve factor
    let result = test_initialize_reserve_config(&env, &contract_id, asset, -100);
    assert_eq!(result, Err(ReserveError::InvalidReserveFactor));
}

#[test]
fn test_initialize_reserve_config_native_asset() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();

    // Initialize for native asset (None)
    let result = test_initialize_reserve_config(&env, &contract_id, None, DEFAULT_RESERVE_FACTOR_BPS);
    assert!(result.is_ok());

    let factor = test_get_reserve_factor(&env, &contract_id, None);
    assert_eq!(factor, DEFAULT_RESERVE_FACTOR_BPS);
}

// ============================================================================
// Reserve Factor Management Tests
// ============================================================================

#[test]
fn test_set_reserve_factor_by_admin() {
    let (env, contract_id, admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize first
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), DEFAULT_RESERVE_FACTOR_BPS).unwrap();

    // Admin sets new reserve factor (25%)
    let new_factor = 2500i128;
    let result = test_set_reserve_factor(&env, &contract_id, admin, asset.clone(), new_factor);
    assert!(result.is_ok());

    // Verify factor is updated
    let factor = test_get_reserve_factor(&env, &contract_id, asset);
    assert_eq!(factor, new_factor);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction(0))")]
fn test_set_reserve_factor_by_non_admin() {
    let (env, contract_id, _admin, user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize first
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), DEFAULT_RESERVE_FACTOR_BPS).unwrap();

    // Non-admin tries to set reserve factor - should fail
    let _ = test_set_reserve_factor(&env, &contract_id, user, asset, 2000);
}

#[test]
fn test_set_reserve_factor_exceeds_max() {
    let (env, contract_id, admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize first
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), DEFAULT_RESERVE_FACTOR_BPS).unwrap();

    // Try to set reserve factor > 50%
    let result = test_set_reserve_factor(&env, &contract_id, admin, asset, MAX_RESERVE_FACTOR_BPS + 1);
    assert_eq!(result, Err(ReserveError::InvalidReserveFactor));
}

#[test]
fn test_set_reserve_factor_to_zero() {
    let (env, contract_id, admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize first
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), DEFAULT_RESERVE_FACTOR_BPS).unwrap();

    // Set reserve factor to zero (disable reserves)
    let result = test_set_reserve_factor(&env, &contract_id, admin, asset.clone(), 0);
    assert!(result.is_ok());

    let factor = test_get_reserve_factor(&env, &contract_id, asset);
    assert_eq!(factor, 0);
}

#[test]
fn test_get_reserve_factor_default() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Get reserve factor without initialization - should return default
    let factor = test_get_reserve_factor(&env, &contract_id, asset);
    assert_eq!(factor, DEFAULT_RESERVE_FACTOR_BPS);
}

// ============================================================================
// Reserve Accrual Tests
// ============================================================================

#[test]
fn test_accrue_reserve_basic() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with 10% reserve factor
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();

    // Accrue reserves from 1000 units of interest
    let interest = 1000i128;
    let result = test_accrue_reserve(&env, &contract_id, asset.clone(), interest);
    assert!(result.is_ok());

    let (reserve_amount, lender_amount) = result.unwrap();

    // 10% to reserves, 90% to lenders
    assert_eq!(reserve_amount, 100); // 1000 * 1000 / 10000 = 100
    assert_eq!(lender_amount, 900); // 1000 - 100 = 900

    // Verify reserve balance is updated
    let balance = test_get_reserve_balance(&env, &contract_id, asset);
    assert_eq!(balance, 100);
}

#[test]
fn test_accrue_reserve_zero_interest() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();

    // Accrue with zero interest
    let result = test_accrue_reserve(&env, &contract_id, asset, 0);
    assert!(result.is_ok());

    let (reserve_amount, lender_amount) = result.unwrap();
    assert_eq!(reserve_amount, 0);
    assert_eq!(lender_amount, 0);
}

#[test]
fn test_accrue_reserve_zero_factor() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with 0% reserve factor
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 0).unwrap();

    // Accrue reserves from 1000 units of interest
    let interest = 1000i128;
    let result = test_accrue_reserve(&env, &contract_id, asset.clone(), interest);
    assert!(result.is_ok());

    let (reserve_amount, lender_amount) = result.unwrap();

    // 0% to reserves, 100% to lenders
    assert_eq!(reserve_amount, 0);
    assert_eq!(lender_amount, 1000);

    let balance = test_get_reserve_balance(&env, &contract_id, asset);
    assert_eq!(balance, 0);
}

#[test]
fn test_accrue_reserve_max_factor() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with 50% reserve factor
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), MAX_RESERVE_FACTOR_BPS).unwrap();

    // Accrue reserves from 1000 units of interest
    let interest = 1000i128;
    let result = test_accrue_reserve(&env, &contract_id, asset.clone(), interest);
    assert!(result.is_ok());

    let (reserve_amount, lender_amount) = result.unwrap();

    // 50% to reserves, 50% to lenders
    assert_eq!(reserve_amount, 500); // 1000 * 5000 / 10000 = 500
    assert_eq!(lender_amount, 500); // 1000 - 500 = 500

    let balance = test_get_reserve_balance(&env, &contract_id, asset);
    assert_eq!(balance, 500);
}

#[test]
fn test_accrue_reserve_multiple_times() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with 10% reserve factor
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();

    // First accrual: 1000 interest
    test_accrue_reserve(&env, &contract_id, asset.clone(), 1000).unwrap();
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 100);

    // Second accrual: 500 interest
    test_accrue_reserve(&env, &contract_id, asset.clone(), 500).unwrap();
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 150); // 100 + 50

    // Third accrual: 2000 interest
    test_accrue_reserve(&env, &contract_id, asset.clone(), 2000).unwrap();
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 350); // 150 + 200
}

#[test]
fn test_accrue_reserve_large_amounts() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with 10% reserve factor
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();

    // Accrue large interest amount
    let large_interest = 1_000_000_000i128; // 1 billion
    let result = test_accrue_reserve(&env, &contract_id, asset.clone(), large_interest);
    assert!(result.is_ok());

    let (reserve_amount, lender_amount) = result.unwrap();
    assert_eq!(reserve_amount, 100_000_000); // 10% of 1 billion
    assert_eq!(lender_amount, 900_000_000); // 90% of 1 billion

    let balance = test_get_reserve_balance(&env, &contract_id, asset);
    assert_eq!(balance, 100_000_000);
}

#[test]
fn test_accrue_reserve_rounding() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with 10% reserve factor
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();

    // Accrue with amount that doesn't divide evenly
    let interest = 999i128;
    let result = test_accrue_reserve(&env, &contract_id, asset.clone(), interest);
    assert!(result.is_ok());

    let (reserve_amount, lender_amount) = result.unwrap();
    // 999 * 1000 / 10000 = 99 (integer division)
    assert_eq!(reserve_amount, 99);
    assert_eq!(lender_amount, 900); // 999 - 99
}

// ============================================================================
// Treasury Address Management Tests
// ============================================================================

#[test]
fn test_set_treasury_address_by_admin() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();

    // Admin sets treasury address
    let result = test_set_treasury_address(&env, &contract_id, admin, treasury.clone());
    assert!(result.is_ok());

    // Verify treasury address is set
    let stored_treasury = test_get_treasury_address(&env, &contract_id);
    assert_eq!(stored_treasury, Some(treasury));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction(0))")]
fn test_set_treasury_address_by_non_admin() {
    let (env, contract_id, _admin, user, treasury) = setup_test_env();

    // Non-admin tries to set treasury address - should fail
    let _ = test_set_treasury_address(&env, &contract_id, user, treasury);
}

#[test]
fn test_set_treasury_address_to_contract() {
    let (env, contract_id, admin, _user, _treasury) = setup_test_env();

    // Try to set treasury to contract address - should fail
    let contract_addr = env.current_contract_address();
    let result = test_set_treasury_address(&env, &contract_id, admin, contract_addr);
    assert_eq!(result, Err(ReserveError::InvalidTreasury));
}

#[test]
fn test_get_treasury_address_not_set() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();

    // Get treasury address before it's set
    let treasury = test_get_treasury_address(&env, &contract_id);
    assert_eq!(treasury, None);
}

#[test]
fn test_update_treasury_address() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();

    // Set initial treasury address
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury.clone()).unwrap();
    assert_eq!(test_get_treasury_address(&env, &contract_id), Some(treasury));

    // Update to new treasury address
    let new_treasury = Address::generate(&env);
    test_set_treasury_address(&env, &contract_id, admin, new_treasury.clone()).unwrap();
    assert_eq!(test_get_treasury_address(&env, &contract_id), Some(new_treasury));
}

// ============================================================================
// Treasury Withdrawal Tests
// ============================================================================

#[test]
fn test_withdraw_reserve_to_treasury_success() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup: initialize, set treasury, accrue reserves
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap(); // Accrues 1000 to reserves

    // Withdraw 500 to treasury
    let result = test_withdraw_reserve_to_treasury(&env, &contract_id, admin, asset.clone(), 500);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 500);

    // Verify reserve balance is reduced
    let balance = test_get_reserve_balance(&env, &contract_id, asset);
    assert_eq!(balance, 500); // 1000 - 500
}

#[test]
fn test_withdraw_reserve_full_balance() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap(); // Accrues 1000

    // Withdraw full balance
    let result = test_withdraw_reserve_to_treasury(&env, &contract_id, admin, asset.clone(), 1000);
    assert!(result.is_ok());

    // Verify reserve balance is zero
    let balance = test_get_reserve_balance(&env, &contract_id, asset);
    assert_eq!(balance, 0);
}

#[test]
fn test_withdraw_reserve_exceeds_balance() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap(); // Accrues 1000

    // Try to withdraw more than available
    let result = test_withdraw_reserve_to_treasury(&env, &contract_id, admin, asset, 1001);
    assert_eq!(result, Err(ReserveError::InsufficientReserve));
}

#[test]
fn test_withdraw_reserve_zero_amount() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap();

    // Try to withdraw zero
    let result = test_withdraw_reserve_to_treasury(&env, &contract_id, admin, asset, 0);
    assert_eq!(result, Err(ReserveError::InvalidAmount));
}

#[test]
fn test_withdraw_reserve_negative_amount() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap();

    // Try to withdraw negative amount
    let result = test_withdraw_reserve_to_treasury(&env, &contract_id, admin, asset, -100);
    assert_eq!(result, Err(ReserveError::InvalidAmount));
}

#[test]
fn test_withdraw_reserve_treasury_not_set() {
    let (env, contract_id, admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup without setting treasury
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap();

    // Try to withdraw without treasury set
    let result = test_withdraw_reserve_to_treasury(&env, &contract_id, admin, asset, 500);
    assert_eq!(result, Err(ReserveError::TreasuryNotSet));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction(0))")]
fn test_withdraw_reserve_by_non_admin() {
    let (env, contract_id, admin, user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_set_treasury_address(&env, &contract_id, admin, treasury).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap();

    // Non-admin tries to withdraw - should fail
    let _ = test_withdraw_reserve_to_treasury(&env, &contract_id, user, asset, 500);
}

#[test]
fn test_withdraw_reserve_multiple_times() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap(); // Accrues 1000

    // First withdrawal: 300
    test_withdraw_reserve_to_treasury(&env, &contract_id, admin.clone(), asset.clone(), 300).unwrap();
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 700);

    // Second withdrawal: 200
    test_withdraw_reserve_to_treasury(&env, &contract_id, admin.clone(), asset.clone(), 200).unwrap();
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 500);

    // Third withdrawal: 500 (remaining)
    test_withdraw_reserve_to_treasury(&env, &contract_id, admin, asset.clone(), 500).unwrap();
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset), 0);
}

#[test]
fn test_withdraw_reserve_from_zero_balance() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup without accruing reserves
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();

    // Try to withdraw from zero balance
    let result = test_withdraw_reserve_to_treasury(&env, &contract_id, admin, asset, 100);
    assert_eq!(result, Err(ReserveError::InsufficientReserve));
}

// ============================================================================
// Reserve Statistics Tests
// ============================================================================

#[test]
fn test_get_reserve_stats() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 2000).unwrap();
    test_set_treasury_address(&env, &contract_id, admin, treasury.clone()).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 5000).unwrap(); // Accrues 1000

    // Get stats
    let (balance, factor, treasury_addr) = test_get_reserve_stats(&env, &contract_id, asset);

    assert_eq!(balance, 1000);
    assert_eq!(factor, 2000);
    assert_eq!(treasury_addr, Some(treasury));
}

#[test]
fn test_get_reserve_stats_no_treasury() {
    let (env, contract_id, _admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Setup without treasury
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1500).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap();

    // Get stats
    let (balance, factor, treasury_addr) = test_get_reserve_stats(&env, &contract_id, asset);

    assert_eq!(balance, 1500);
    assert_eq!(factor, 1500);
    assert_eq!(treasury_addr, None);
}

// ============================================================================
// Integration and Edge Case Tests
// ============================================================================

#[test]
fn test_complete_reserve_lifecycle() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // 1. Initialize reserve config
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();

    // 2. Set treasury address
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();

    // 3. Accrue reserves multiple times
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap(); // +1000
    test_accrue_reserve(&env, &contract_id, asset.clone(), 5000).unwrap(); // +500
    test_accrue_reserve(&env, &contract_id, asset.clone(), 2000).unwrap(); // +200
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 1700);

    // 4. Withdraw partial reserves
    test_withdraw_reserve_to_treasury(&env, &contract_id, admin.clone(), asset.clone(), 700).unwrap();
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 1000);

    // 5. Accrue more reserves
    test_accrue_reserve(&env, &contract_id, asset.clone(), 3000).unwrap(); // +300
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 1300);

    // 6. Update reserve factor
    test_set_reserve_factor(&env, &contract_id, admin.clone(), asset.clone(), 2000).unwrap();

    // 7. Accrue with new factor
    test_accrue_reserve(&env, &contract_id, asset.clone(), 5000).unwrap(); // +1000 (20%)
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 2300);

    // 8. Withdraw remaining
    test_withdraw_reserve_to_treasury(&env, &contract_id, admin, asset.clone(), 2300).unwrap();
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset), 0);
}

#[test]
fn test_multiple_assets_independent_reserves() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();
    let asset1 = Some(Address::generate(&env));
    let asset2 = Some(Address::generate(&env));

    // Initialize both assets with different factors
    test_initialize_reserve_config(&env, &contract_id, asset1.clone(), 1000).unwrap(); // 10%
    test_initialize_reserve_config(&env, &contract_id, asset2.clone(), 2000).unwrap(); // 20%

    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();

    // Accrue reserves for both assets
    test_accrue_reserve(&env, &contract_id, asset1.clone(), 10000).unwrap(); // +1000
    test_accrue_reserve(&env, &contract_id, asset2.clone(), 10000).unwrap(); // +2000

    // Verify independent balances
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset1.clone()), 1000);
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset2.clone()), 2000);

    // Withdraw from asset1
    test_withdraw_reserve_to_treasury(&env, &contract_id, admin.clone(), asset1.clone(), 500).unwrap();

    // Verify asset2 is unaffected
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset1), 500);
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset2), 2000);
}

#[test]
fn test_native_asset_reserves() {
    let (env, contract_id, admin, _user, treasury) = setup_test_env();

    // Test with native asset (None)
    test_initialize_reserve_config(&env, &contract_id, None, 1500).unwrap();
    test_set_treasury_address(&env, &contract_id, admin.clone(), treasury).unwrap();

    test_accrue_reserve(&env, &contract_id, None, 10000).unwrap(); // +1500
    assert_eq!(test_get_reserve_balance(&env, &contract_id, None), 1500);

    test_withdraw_reserve_to_treasury(&env, &contract_id, admin, None, 1000).unwrap();
    assert_eq!(test_get_reserve_balance(&env, &contract_id, None), 500);
}

#[test]
fn test_reserve_factor_change_does_not_affect_existing_balance() {
    let (env, contract_id, admin, _user, _treasury) = setup_test_env();
    let asset = Some(Address::generate(&env));

    // Initialize with 10% factor
    test_initialize_reserve_config(&env, &contract_id, asset.clone(), 1000).unwrap();
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap(); // +1000
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 1000);

    // Change factor to 20%
    test_set_reserve_factor(&env, &contract_id, admin, asset.clone(), 2000).unwrap();

    // Existing balance should remain unchanged
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset.clone()), 1000);

    // New accruals use new factor
    test_accrue_reserve(&env, &contract_id, asset.clone(), 10000).unwrap(); // +2000 (20%)
    assert_eq!(test_get_reserve_balance(&env, &contract_id, asset), 3000); // 1000 + 2000
}
