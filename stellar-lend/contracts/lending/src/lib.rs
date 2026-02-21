#![no_std]
#![allow(deprecated)]
use soroban_sdk::{contract, contractimpl, Address, Env,};

mod borrow;
mod pause;

use borrow::{
    borrow, get_admin, get_user_collateral, get_user_debt, initialize_borrow_settings, set_admin,
    BorrowCollateral, BorrowError, DebtPosition,
};
use pause::{is_paused, set_pause, PauseType};

mod deposit;
use deposit::{
    deposit, get_user_collateral as get_deposit_collateral, initialize_deposit_settings,
    DepositCollateral, DepositError,
};

#[cfg(test)]
mod borrow_test;
#[cfg(test)]
mod pause_test;

#[cfg(test)]
mod deposit_test;

#[contract]
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    /// Initialize the protocol with admin and settings
    pub fn initialize(
        env: Env,
        admin: Address,
        debt_ceiling: i128,
        min_borrow_amount: i128,
    ) -> Result<(), BorrowError> {
        if get_admin(&env).is_some() {
            return Err(BorrowError::Unauthorized);
        }
        set_admin(&env, &admin);
        initialize_borrow_settings(&env, debt_ceiling, min_borrow_amount)?;
        Ok(())
    }

    /// Borrow assets against deposited collateral
    pub fn borrow(
        env: Env,
        user: Address,
        asset: Address,
        amount: i128,
        collateral_asset: Address,
        collateral_amount: i128,
    ) -> Result<(), BorrowError> {
        borrow(
            &env,
            user,
            asset,
            amount,
            collateral_asset,
            collateral_amount,
        )
    }

    /// Set protocol pause state for a specific operation (admin only)
    pub fn set_pause(
        env: Env,
        admin: Address,
        pause_type: PauseType,
        paused: bool,
    ) -> Result<(), BorrowError> {
        let current_admin = get_admin(&env).ok_or(BorrowError::Unauthorized)?;
        if admin != current_admin {
            return Err(BorrowError::Unauthorized);
        }
        admin.require_auth();
        set_pause(&env, admin, pause_type, paused);
        Ok(())
    }

    /// Repay borrowed assets
    pub fn repay(
        env: Env,
        user: Address,
        _asset: Address,
        _amount: i128,
    ) -> Result<(), BorrowError> {
        user.require_auth();
        if is_paused(&env, PauseType::Repay) {
            return Err(BorrowError::ProtocolPaused);
        }
        // Stub implementation
        Ok(())
    }

    /// Withdraw collateral
    pub fn withdraw(
        env: Env,
        user: Address,
        _asset: Address,
        _amount: i128,
    ) -> Result<(), BorrowError> {
        user.require_auth();
        if is_paused(&env, PauseType::Withdraw) {
            return Err(BorrowError::ProtocolPaused);
        }
        // Stub implementation
        Ok(())
    }

    /// Liquidate a position
    pub fn liquidate(
        env: Env,
        liquidator: Address,
        _user: Address,
        _debt_asset: Address,
        _collateral_asset: Address,
        _amount: i128,
    ) -> Result<(), BorrowError> {
        liquidator.require_auth();
        if is_paused(&env, PauseType::Liquidation) {
            return Err(BorrowError::ProtocolPaused);
        }
        // Stub implementation
        Ok(())
    }

    /// Get user's debt position
    pub fn get_user_debt(env: Env, user: Address) -> DebtPosition {
        get_user_debt(&env, &user)
    }

    /// Get user's collateral position
    pub fn get_user_collateral(env: Env, user: Address) -> BorrowCollateral {
        get_user_collateral(&env, &user)
    }

    /// Deposit collateral into the protocol
    pub fn deposit(
        env: Env,
        user: Address,
        asset: Address,
        amount: i128,
    ) -> Result<i128, DepositError> {
        if is_paused(&env, PauseType::Deposit) {
            return Err(DepositError::DepositPaused);
        }
        deposit(&env, user, asset, amount)
    }

    /// Initialize deposit settings (admin only)
    pub fn initialize_deposit_settings(
        env: Env,
        deposit_cap: i128,
        min_deposit_amount: i128,
    ) -> Result<(), DepositError> {
        initialize_deposit_settings(&env, deposit_cap, min_deposit_amount)
    }

    /// Set deposit pause state (admin only)
    /// Deprecated: use set_pause instead
    pub fn set_deposit_paused(env: Env, paused: bool) -> Result<(), DepositError> {
        env.storage()
            .persistent()
            .set(&pause::PauseDataKey::State(PauseType::Deposit), &paused);
        Ok(())
    }

    /// Get user's deposit collateral position
    pub fn get_user_collateral_deposit(
        env: Env,
        user: Address,
        asset: Address,
    ) -> DepositCollateral {
        get_deposit_collateral(&env, &user, &asset)
    }

    /// Get protocol admin
    pub fn get_admin(env: Env) -> Option<Address> {
        get_admin(&env)
    }
}
