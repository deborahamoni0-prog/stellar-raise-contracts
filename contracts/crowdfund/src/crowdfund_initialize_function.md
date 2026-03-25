# `initialize` — Crowdfund Contract

Initializes a new crowdfunding campaign. Must be called exactly once after deployment.

---

## Signature

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    creator: Address,
    token: Address,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    platform_config: Option<PlatformConfig>,
    bonus_goal: Option<i128>,
    bonus_goal_description: Option<String>,
) -> Result<(), ContractError>
```

---

## Parameters

| Parameter               | Type                    | Required | Description                                                                 |
|-------------------------|-------------------------|----------|-----------------------------------------------------------------------------|
| `admin`                 | `Address`               | Yes      | Address authorized to call `upgrade`. Typically the deployer.               |
| `creator`               | `Address`               | Yes      | Campaign creator. Must sign the transaction (`require_auth`).               |
| `token`                 | `Address`               | Yes      | Token contract address used for contributions and payouts.                  |
| `goal`                  | `i128`                  | Yes      | Funding target in the token's smallest unit (e.g. stroops). Must be > 0.   |
| `deadline`              | `u64`                   | Yes      | Campaign end time as a UNIX ledger timestamp. Must be in the future.        |
| `min_contribution`      | `i128`                  | Yes      | Minimum single contribution. Must be > 0 and ≤ `goal`.                     |
| `platform_config`       | `Option<PlatformConfig>`| No       | Optional platform fee config. `fee_bps` must be ≤ 10 000 (100 %).          |
| `bonus_goal`            | `Option<i128>`          | No       | Optional stretch goal. Must be strictly greater than `goal`.                |
| `bonus_goal_description`| `Option<String>`        | No       | Human-readable description of the bonus goal reward.                        |

### `PlatformConfig` fields

| Field     | Type      | Description                                              |
|-----------|-----------|----------------------------------------------------------|
| `address` | `Address` | Platform wallet that receives the fee on withdrawal.     |
| `fee_bps` | `u32`     | Fee in basis points (100 bps = 1 %). Maximum: 10 000.   |

---

## Return value

`Ok(())` on success. The contract is now in `Status::Active` and ready to accept contributions.

---

## Errors

| Error                    | Code | Condition                                                  |
|--------------------------|------|------------------------------------------------------------|
| `AlreadyInitialized`     | 1    | `initialize` has already been called on this contract.     |

Additional panics (not `ContractError`):

| Condition                                        | Message                                      |
|--------------------------------------------------|----------------------------------------------|
| `platform_config.fee_bps > 10_000`               | `"platform fee cannot exceed 100%"`          |
| `bonus_goal` is set but `bonus_goal <= goal`     | `"bonus goal must be greater than primary goal"` |
| `bonus_goal_description` fails length validation | validation error from `contract_state_size`  |

---

## State written

| Storage key              | Type              | Value set                        |
|--------------------------|-------------------|----------------------------------|
| `Admin`                  | `Address`         | `admin`                          |
| `Creator`                | `Address`         | `creator`                        |
| `Token`                  | `Address`         | `token`                          |
| `Goal`                   | `i128`            | `goal`                           |
| `Deadline`               | `u64`             | `deadline`                       |
| `MinContribution`        | `i128`            | `min_contribution`               |
| `TotalRaised`            | `i128`            | `0`                              |
| `Status`                 | `Status`          | `Status::Active`                 |
| `BonusGoalReachedEmitted`| `bool`            | `false`                          |
| `Contributors`           | `Vec<Address>`    | empty list (persistent storage)  |
| `Roadmap`                | `Vec<RoadmapItem>`| empty list                       |
| `PlatformConfig`         | `PlatformConfig`  | set only if `platform_config` is `Some` |
| `BonusGoal`              | `i128`            | set only if `bonus_goal` is `Some`      |
| `BonusGoalDescription`   | `String`          | set only if `bonus_goal_description` is `Some` |

---

## Security notes

- **One-time call**: The guard `env.storage().instance().has(&DataKey::Creator)` prevents re-initialization. Any subsequent call returns `Err(ContractError::AlreadyInitialized)`.
- **Creator auth**: `creator.require_auth()` is called unconditionally, ensuring the transaction must be signed by the creator's key.
- **Admin separation**: `admin` and `creator` can be different addresses. The admin is only used for contract upgrades; the creator manages the campaign lifecycle.
- **No arithmetic in initialize**: All values are stored as-is. No overflow risk at this stage.
- **Platform fee cap**: A fee above 10 000 bps would allow the platform to drain the entire campaign. The hard cap prevents misconfiguration.
- **Bonus goal ordering**: Enforcing `bonus_goal > goal` prevents a bonus goal that is already met at launch.

---

## Example — CLI invocation

```bash
DEADLINE=$(date -d "+30 days" +%s)

stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <CREATOR_SECRET_KEY> \
  -- initialize \
  --admin   <ADMIN_ADDRESS> \
  --creator <CREATOR_ADDRESS> \
  --token   <TOKEN_CONTRACT_ADDRESS> \
  --goal    1000000000 \
  --deadline $DEADLINE \
  --min_contribution 1000000
```

> Amounts are in stroops (1 XLM = 10 000 000 stroops).

---

## Example — Rust integration test

```rust
use soroban_sdk::{testutils::Address as _, Address, Env};
use crowdfund::{CrowdfundContract, CrowdfundContractClient};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let admin   = Address::generate(&env);
    let creator = Address::generate(&env);
    let token   = Address::generate(&env);

    env.ledger().set_timestamp(1_000);

    client.initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,   // goal
        &10_000,      // deadline (timestamp)
        &1_000,       // min_contribution
        &None,        // platform_config
        &None,        // bonus_goal
        &None,        // bonus_goal_description
    );

    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.total_raised(), 0);
}
```

---

## Validation helper

The `crowdfund_initialize_function` module exposes a standalone validation function that can be used before calling `initialize`:

```rust
use crowdfund::crowdfund_initialize_function::{
    validate_initialization_params, InitError,
};

let result = validate_initialization_params(
    &env,
    goal,
    deadline,
    min_contribution,
    Some(fee_bps),   // platform fee in bps, or None
    Some(bonus),     // bonus goal, or None
);

match result {
    Ok(()) => { /* safe to call initialize */ }
    Err(InitError::GoalNotPositive)            => { /* handle */ }
    Err(InitError::DeadlineInPast)             => { /* handle */ }
    Err(InitError::MinContributionNotPositive) => { /* handle */ }
    Err(InitError::MinContributionExceedsGoal) => { /* handle */ }
    Err(InitError::PlatformFeeExceedsMax)      => { /* handle */ }
    Err(InitError::BonusGoalNotGreaterThanGoal)=> { /* handle */ }
}
```

### `InitError` variants

| Variant                        | Condition                                  |
|--------------------------------|--------------------------------------------|
| `GoalNotPositive`              | `goal <= 0`                                |
| `DeadlineInPast`               | `deadline <= current_ledger_timestamp`     |
| `MinContributionNotPositive`   | `min_contribution <= 0`                    |
| `MinContributionExceedsGoal`   | `min_contribution > goal`                  |
| `PlatformFeeExceedsMax`        | `fee_bps > 10_000`                         |
| `BonusGoalNotGreaterThanGoal`  | `bonus_goal <= goal`                       |

---

## Related functions

| Function          | Description                                              |
|-------------------|----------------------------------------------------------|
| `contribute`      | Pledge tokens to the active campaign.                    |
| `withdraw`        | Creator claims funds after a successful campaign.        |
| `refund_single`   | Contributor reclaims tokens if the goal was not met.     |
| `upgrade`         | Admin replaces the contract WASM without changing state. |
| `update_metadata` | Creator updates title, description, or social links.     |
