#![cfg(test)]

use crate::{HelloContract, HelloContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

use crate::deposit::{DepositDataKey, Position, ProtocolAnalytics, UserAnalytics};

// Helper functions
fn create_test_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn get_collateral_balance(env: &Env, contract_id: &Address, user: &Address) -> i128 {
    env.as_contract(contract_id, || {
        let key = DepositDataKey::CollateralBalance(user.clone());
        env.storage().persistent().get(&key).unwrap_or(0)
    })
}

fn get_user_position(env: &Env, contract_id: &Address, user: &Address) -> Option<Position> {
    env.as_contract(contract_id, || {
        let key = DepositDataKey::Position(user.clone());
        env.storage().persistent().get(&key)
    })
}

fn get_user_analytics(env: &Env, contract_id: &Address, user: &Address) -> Option<UserAnalytics> {
    env.as_contract(contract_id, || {
        let key = DepositDataKey::UserAnalytics(user.clone());
        env.storage().persistent().get(&key)
    })
}

fn get_protocol_analytics(env: &Env, contract_id: &Address) -> Option<ProtocolAnalytics> {
    env.as_contract(contract_id, || {
        let key = DepositDataKey::ProtocolAnalytics;
        env.storage().persistent().get(&key)
    })
}

// ==================== BASIC WITHDRAW TESTS ====================

#[test]
fn test_withdraw_success() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit collateral
    let deposit_amount = 1000;
    client.deposit_collateral(&user, &None, &deposit_amount);

    // Withdraw
    let withdraw_amount = 500;
    let result = client.withdraw_collateral(&user, &None, &withdraw_amount);

    // Verify result
    assert_eq!(result, deposit_amount - withdraw_amount);

    // Verify collateral balance
    let balance = get_collateral_balance(&env, &contract_id, &user);
    assert_eq!(balance, deposit_amount - withdraw_amount);

    // Verify position
    let position = get_user_position(&env, &contract_id, &user).unwrap();
    assert_eq!(position.collateral, deposit_amount - withdraw_amount);
}

#[test]
fn test_withdraw_full_amount_no_debt() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    let deposit_amount = 1000;
    client.deposit_collateral(&user, &None, &deposit_amount);

    // Withdraw all (maximum withdrawal when no debt)
    let result = client.withdraw_collateral(&user, &None, &deposit_amount);

    assert_eq!(result, 0);

    // Verify collateral balance is zero
    let balance = get_collateral_balance(&env, &contract_id, &user);
    assert_eq!(balance, 0);
}

#[test]
fn test_withdraw_multiple_times() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    let deposit_amount = 1000;
    client.deposit_collateral(&user, &None, &deposit_amount);

    // First withdrawal
    let withdraw1 = 300;
    let result1 = client.withdraw_collateral(&user, &None, &withdraw1);
    assert_eq!(result1, deposit_amount - withdraw1);

    // Second withdrawal
    let withdraw2 = 200;
    let result2 = client.withdraw_collateral(&user, &None, &withdraw2);
    assert_eq!(result2, deposit_amount - withdraw1 - withdraw2);

    // Third withdrawal
    let withdraw3 = 100;
    let result3 = client.withdraw_collateral(&user, &None, &withdraw3);
    assert_eq!(result3, deposit_amount - withdraw1 - withdraw2 - withdraw3);

    // Verify final balance
    let balance = get_collateral_balance(&env, &contract_id, &user);
    assert_eq!(balance, 400);
}

// ==================== INPUT VALIDATION TESTS ====================

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_withdraw_zero_amount() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit first
    client.deposit_collateral(&user, &None, &1000);

    // Try to withdraw zero
    client.withdraw_collateral(&user, &None, &0);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_withdraw_negative_amount() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit first
    client.deposit_collateral(&user, &None, &1000);

    // Try to withdraw negative amount
    client.withdraw_collateral(&user, &None, &(-100));
}

#[test]
#[should_panic(expected = "InsufficientCollateral")]
fn test_withdraw_insufficient_balance() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    client.deposit_collateral(&user, &None, &500);

    // Try to withdraw more than balance
    client.withdraw_collateral(&user, &None, &1000);
}

#[test]
#[should_panic(expected = "InsufficientCollateral")]
fn test_withdraw_no_collateral() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Try to withdraw without depositing
    client.withdraw_collateral(&user, &None, &100);
}

// ==================== COLLATERAL RATIO TESTS ====================

#[test]
fn test_withdraw_with_debt_maintains_ratio() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit collateral
    let collateral = 2000;
    client.deposit_collateral(&user, &None, &collateral);

    // Set debt
    env.as_contract(&contract_id, || {
        let position_key = DepositDataKey::Position(user.clone());
        let mut position = env
            .storage()
            .persistent()
            .get::<DepositDataKey, Position>(&position_key)
            .unwrap();
        position.debt = 500;
        env.storage().persistent().set(&position_key, &position);
    });

    // Withdraw should work if ratio is maintained
    // Current: 2000/500 = 400%
    // After: 1500/500 = 300% (still > 150%)
    let withdraw_amount = 500;
    let result = client.withdraw_collateral(&user, &None, &withdraw_amount);
    assert_eq!(result, collateral - withdraw_amount);
}

#[test]
#[should_panic(expected = "InsufficientCollateralRatio")]
fn test_withdraw_violates_collateral_ratio() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit collateral
    let collateral = 1000;
    client.deposit_collateral(&user, &None, &collateral);

    // Set debt
    env.as_contract(&contract_id, || {
        let position_key = DepositDataKey::Position(user.clone());
        let mut position = env
            .storage()
            .persistent()
            .get::<DepositDataKey, Position>(&position_key)
            .unwrap();
        position.debt = 500;
        env.storage().persistent().set(&position_key, &position);
    });

    // Try to withdraw too much
    // Current: 1000/500 = 200%
    // After: 400/500 = 80% (< 150% minimum)
    client.withdraw_collateral(&user, &None, &600);
}

#[test]
#[should_panic(expected = "InsufficientCollateralRatio")]
fn test_withdraw_at_minimum_ratio_boundary() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit collateral
    let collateral = 1500;
    client.deposit_collateral(&user, &None, &collateral);

    // Set debt
    env.as_contract(&contract_id, || {
        let position_key = DepositDataKey::Position(user.clone());
        let mut position = env
            .storage()
            .persistent()
            .get::<DepositDataKey, Position>(&position_key)
            .unwrap();
        position.debt = 1000;
        env.storage().persistent().set(&position_key, &position);
    });

    // Withdraw to exactly 150% ratio
    // Current: 1500/1000 = 150%
    // After withdrawing 1: 1499/1000 = 149.9% (just below minimum, should fail)
    client.withdraw_collateral(&user, &None, &1);
}

#[test]
fn test_withdraw_with_interest_accrued() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit collateral
    let collateral = 3000;
    client.deposit_collateral(&user, &None, &collateral);

    // Set debt and interest
    env.as_contract(&contract_id, || {
        let position_key = DepositDataKey::Position(user.clone());
        let mut position = env
            .storage()
            .persistent()
            .get::<DepositDataKey, Position>(&position_key)
            .unwrap();
        position.debt = 500;
        position.borrow_interest = 100; // Total debt = 600
        env.storage().persistent().set(&position_key, &position);
    });

    // Withdraw considering total debt (principal + interest)
    // Current: 3000/600 = 500%
    // After: 2000/600 = 333% (still > 150%)
    let withdraw_amount = 1000;
    let result = client.withdraw_collateral(&user, &None, &withdraw_amount);
    assert_eq!(result, collateral - withdraw_amount);
}

// ==================== PAUSE MECHANISM TESTS ====================

#[test]
#[should_panic(expected = "WithdrawPaused")]
fn test_withdraw_when_paused() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    client.deposit_collateral(&user, &None, &1000);

    // Set pause switch
    env.as_contract(&contract_id, || {
        let pause_key = DepositDataKey::PauseSwitches;
        let mut pause_map = soroban_sdk::Map::new(&env);
        pause_map.set(Symbol::new(&env, "pause_withdraw"), true);
        env.storage().persistent().set(&pause_key, &pause_map);
    });

    // Try to withdraw (should fail)
    client.withdraw_collateral(&user, &None, &500);
}

#[test]
fn test_withdraw_when_not_paused() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    client.deposit_collateral(&user, &None, &1000);

    // Set pause switch to false
    env.as_contract(&contract_id, || {
        let pause_key = DepositDataKey::PauseSwitches;
        let mut pause_map = soroban_sdk::Map::new(&env);
        pause_map.set(Symbol::new(&env, "pause_withdraw"), false);
        env.storage().persistent().set(&pause_key, &pause_map);
    });

    // Withdraw should succeed
    let result = client.withdraw_collateral(&user, &None, &500);
    assert_eq!(result, 500);
}

// ==================== ANALYTICS TESTS ====================

#[test]
fn test_withdraw_updates_user_analytics() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    let deposit_amount = 1000;
    client.deposit_collateral(&user, &None, &deposit_amount);

    // Withdraw
    let withdraw_amount = 300;
    client.withdraw_collateral(&user, &None, &withdraw_amount);

    // Verify analytics
    let analytics = get_user_analytics(&env, &contract_id, &user).unwrap();
    assert_eq!(analytics.total_withdrawals, withdraw_amount);
    assert_eq!(analytics.collateral_value, deposit_amount - withdraw_amount);
    assert_eq!(analytics.transaction_count, 2); // deposit + withdraw
}

#[test]
fn test_withdraw_updates_protocol_analytics() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    let deposit_amount = 1000;
    client.deposit_collateral(&user, &None, &deposit_amount);

    // Get initial TVL
    let initial_analytics = get_protocol_analytics(&env, &contract_id).unwrap();
    let initial_tvl = initial_analytics.total_value_locked;

    // Withdraw
    let withdraw_amount = 300;
    client.withdraw_collateral(&user, &None, &withdraw_amount);

    // Verify protocol analytics updated
    let final_analytics = get_protocol_analytics(&env, &contract_id).unwrap();
    assert_eq!(
        final_analytics.total_value_locked,
        initial_tvl - withdraw_amount
    );
}

#[test]
fn test_withdraw_multiple_users_analytics() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // User 1 deposits and withdraws
    client.deposit_collateral(&user1, &None, &1000);
    client.withdraw_collateral(&user1, &None, &300);

    // User 2 deposits and withdraws
    client.deposit_collateral(&user2, &None, &2000);
    client.withdraw_collateral(&user2, &None, &500);

    // Verify user 1 analytics
    let analytics1 = get_user_analytics(&env, &contract_id, &user1).unwrap();
    assert_eq!(analytics1.total_withdrawals, 300);
    assert_eq!(analytics1.collateral_value, 700);

    // Verify user 2 analytics
    let analytics2 = get_user_analytics(&env, &contract_id, &user2).unwrap();
    assert_eq!(analytics2.total_withdrawals, 500);
    assert_eq!(analytics2.collateral_value, 1500);

    // Verify protocol analytics
    let protocol_analytics = get_protocol_analytics(&env, &contract_id).unwrap();
    assert_eq!(protocol_analytics.total_value_locked, 700 + 1500);
}

// ==================== EDGE CASE TESTS ====================

#[test]
fn test_withdraw_large_amounts() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit large amount
    let large_amount = i128::MAX / 2;
    client.deposit_collateral(&user, &None, &large_amount);

    // Withdraw large amount
    let withdraw_amount = large_amount / 2;
    let result = client.withdraw_collateral(&user, &None, &withdraw_amount);
    assert_eq!(result, large_amount - withdraw_amount);
}

#[test]
fn test_withdraw_after_multiple_deposits() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Multiple deposits
    client.deposit_collateral(&user, &None, &100);
    client.deposit_collateral(&user, &None, &200);
    client.deposit_collateral(&user, &None, &300);

    // Total deposited: 600
    let balance = get_collateral_balance(&env, &contract_id, &user);
    assert_eq!(balance, 600);

    // Withdraw
    let result = client.withdraw_collateral(&user, &None, &400);
    assert_eq!(result, 200);
}

#[test]
fn test_withdraw_position_timestamp_updated() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    client.deposit_collateral(&user, &None, &1000);

    let initial_position = get_user_position(&env, &contract_id, &user).unwrap();
    let initial_time = initial_position.last_accrual_time;

    // Withdraw
    client.withdraw_collateral(&user, &None, &500);

    let final_position = get_user_position(&env, &contract_id, &user).unwrap();
    let final_time = final_position.last_accrual_time;

    // Timestamp should be updated
    assert!(final_time >= initial_time);
}

// ==================== INTEGRATION TESTS ====================

#[test]
fn test_withdraw_deposit_withdraw_cycle() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    client.deposit_collateral(&user, &None, &1000);

    // Withdraw
    client.withdraw_collateral(&user, &None, &500);

    // Deposit again
    client.deposit_collateral(&user, &None, &300);

    // Withdraw again
    let result = client.withdraw_collateral(&user, &None, &400);

    // Final balance: 1000 - 500 + 300 - 400 = 400
    assert_eq!(result, 400);
}

#[test]
fn test_withdraw_collateralization_ratio_calculation() {
    let env = create_test_env();
    let contract_id = env.register(HelloContract, ());
    let client = HelloContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Deposit
    client.deposit_collateral(&user, &None, &2000);

    // Set debt and update analytics to reflect it
    env.as_contract(&contract_id, || {
        let position_key = DepositDataKey::Position(user.clone());
        let mut position = env
            .storage()
            .persistent()
            .get::<DepositDataKey, Position>(&position_key)
            .unwrap();
        position.debt = 500;
        env.storage().persistent().set(&position_key, &position);

        // Update analytics to reflect the debt
        let analytics_key = DepositDataKey::UserAnalytics(user.clone());
        let mut analytics = env
            .storage()
            .persistent()
            .get::<DepositDataKey, UserAnalytics>(&analytics_key)
            .unwrap();
        analytics.debt_value = 500;
        analytics.collateralization_ratio = (2000 * 10000) / 500; // 40000 (400%)
        env.storage().persistent().set(&analytics_key, &analytics);
    });

    // Withdraw
    client.withdraw_collateral(&user, &None, &500);

    // Verify analytics shows correct ratio after withdrawal
    let analytics = get_user_analytics(&env, &contract_id, &user).unwrap();
    // Ratio = (1500 * 10000) / 500 = 30000 (300%)
    assert_eq!(analytics.collateralization_ratio, 30000);
}
