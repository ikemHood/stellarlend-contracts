# Protocol Pause Mechanism

The StellarLend protocol includes a granular pause mechanism to ensure safety during emergency situations or maintenance.

## Features

- **Granular Control**: Pause specific operations (`Deposit`, `Borrow`, `Repay`, `Withdraw`, `Liquidation`) without affecting others.
- **Global Pause**: A master switch (`All`) to pause the entire protocol immediately.
- **Admin Managed**: Only the protocol admin can toggle pause states.
- **Event Driven**: All pause state changes emit `pause_changed` events for transparency.

## Operation Types

The protocol supports the following `PauseType` values:

| Enum Value    | Description                                                       |
| ------------- | ----------------------------------------------------------------- |
| `All`         | Global pause affecting all operations listed below.               |
| `Deposit`     | Prevents users from depositing new collateral.                    |
| `Borrow`      | Prevents users from taking out new loans.                         |
| `Repay`       | Prevents users from repaying loans (should be used with caution). |
| `Withdraw`    | Prevents users from withdrawing collateral.                       |
| `Liquidation` | Prevents liquidations from being performed.                       |

## Contract Interface

### Admin Functions

#### `set_pause(admin: Address, pause_type: PauseType, paused: bool)`

Toggles the pause state for a specific operation or the entire protocol.

- **Requires Authorization**: Yes (by `admin`).
- **Emits**: `pause_changed` event.

### Public Functions

#### `get_admin() -> Option<Address>`

Returns the current protocol admin address.

## Security Assumptions

1. **Admin Trust**: The admin is assumed to be a multisig or a DAO-governed address to prevent centralization risks.
2. **Persistence**: Pause states are stored in persistent storage to survive ledger upgrades and contract updates.
3. **No Bypass**: All user-facing operations (deposit, borrow, etc.) check the pause state before execution.

## Usage Example (Rust SDK)

```rust
// Pause borrowing in an emergency
client.set_pause(&admin, &PauseType::Borrow, &true);

// Re-enable borrowing
client.set_pause(&admin, &PauseType::Borrow, &false);
```
