#![cfg(test)]

use soroban_sdk::{testutils::Ledger, Env};

use crate::crowdfund_initialize_function::{
    validate_initialization_params, validate_initialization_params_bool, InitError,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn env_at(ts: u64) -> Env {
    let env = Env::default();
    env.ledger().set_timestamp(ts);
    env
}

// ── Happy-path ───────────────────────────────────────────────────────────────

#[test]
fn test_valid_minimal_params() {
    let env = env_at(1_000);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 2_000, 10, None, None),
        Ok(())
    );
}

#[test]
fn test_valid_min_contribution_equals_goal() {
    // min_contribution == goal is the boundary — still valid.
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 500, 1_000, 500, None, None),
        Ok(())
    );
}

#[test]
fn test_valid_with_zero_fee_bps() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, Some(0), None),
        Ok(())
    );
}

#[test]
fn test_valid_with_max_fee_bps() {
    // 10 000 bps = 100 % — edge case that must be accepted.
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, Some(10_000), None),
        Ok(())
    );
}

#[test]
fn test_valid_with_bonus_goal() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, None, Some(2_000)),
        Ok(())
    );
}

#[test]
fn test_valid_bonus_goal_just_above_primary() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, None, Some(1_001)),
        Ok(())
    );
}

#[test]
fn test_valid_deadline_one_second_in_future() {
    let env = env_at(999);
    assert_eq!(
        validate_initialization_params(&env, 100, 1_000, 1, None, None),
        Ok(())
    );
}

#[test]
fn test_valid_large_goal() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, i128::MAX, 1_000, 1, None, None),
        Ok(())
    );
}

// ── goal validation ──────────────────────────────────────────────────────────

#[test]
fn test_goal_zero_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 0, 1_000, 1, None, None),
        Err(InitError::GoalNotPositive)
    );
}

#[test]
fn test_goal_negative_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, -1, 1_000, 1, None, None),
        Err(InitError::GoalNotPositive)
    );
}

// ── deadline validation ──────────────────────────────────────────────────────

#[test]
fn test_deadline_in_past_is_invalid() {
    let env = env_at(3_000);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 2_000, 10, None, None),
        Err(InitError::DeadlineInPast)
    );
}

#[test]
fn test_deadline_equal_to_now_is_invalid() {
    let env = env_at(1_000);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 1_000, 10, None, None),
        Err(InitError::DeadlineInPast)
    );
}

// ── min_contribution validation ──────────────────────────────────────────────

#[test]
fn test_min_contribution_zero_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 0, None, None),
        Err(InitError::MinContributionNotPositive)
    );
}

#[test]
fn test_min_contribution_negative_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, -5, None, None),
        Err(InitError::MinContributionNotPositive)
    );
}

#[test]
fn test_min_contribution_exceeds_goal() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 100, 9_999, 150, None, None),
        Err(InitError::MinContributionExceedsGoal)
    );
}

#[test]
fn test_min_contribution_one_above_goal_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1_001, None, None),
        Err(InitError::MinContributionExceedsGoal)
    );
}

// ── platform fee validation ──────────────────────────────────────────────────

#[test]
fn test_platform_fee_above_max_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, Some(10_001), None),
        Err(InitError::PlatformFeeExceedsMax)
    );
}

#[test]
fn test_platform_fee_u32_max_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, Some(u32::MAX), None),
        Err(InitError::PlatformFeeExceedsMax)
    );
}

// ── bonus_goal validation ────────────────────────────────────────────────────

#[test]
fn test_bonus_goal_equal_to_primary_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, None, Some(1_000)),
        Err(InitError::BonusGoalNotGreaterThanGoal)
    );
}

#[test]
fn test_bonus_goal_below_primary_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, None, Some(500)),
        Err(InitError::BonusGoalNotGreaterThanGoal)
    );
}

// ── error ordering (goal checked first) ─────────────────────────────────────

#[test]
fn test_goal_error_takes_priority_over_deadline() {
    // Both goal and deadline are invalid; goal error should surface first.
    let env = env_at(5_000);
    assert_eq!(
        validate_initialization_params(&env, 0, 1_000, 1, None, None),
        Err(InitError::GoalNotPositive)
    );
}

#[test]
fn test_deadline_error_takes_priority_over_min_contribution() {
    let env = env_at(5_000);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 1_000, 0, None, None),
        Err(InitError::DeadlineInPast)
    );
}

// ── InitError::message ───────────────────────────────────────────────────────

#[test]
fn test_error_messages_are_non_empty() {
    let errors = [
        InitError::GoalNotPositive,
        InitError::DeadlineInPast,
        InitError::MinContributionNotPositive,
        InitError::MinContributionExceedsGoal,
        InitError::PlatformFeeExceedsMax,
        InitError::BonusGoalNotGreaterThanGoal,
    ];
    for e in errors {
        assert!(
            !e.message().is_empty(),
            "message for {e:?} must not be empty"
        );
    }
}

// ── bool-returning compat wrapper ────────────────────────────────────────────

#[test]
fn test_bool_wrapper_returns_true_for_valid_params() {
    let env = env_at(1_000);
    assert!(validate_initialization_params_bool(&env, 1_000, 2_000, 10));
}

#[test]
fn test_bool_wrapper_returns_false_for_invalid_goal() {
    let env = env_at(0);
    assert!(!validate_initialization_params_bool(&env, 0, 2_000, 10));
}

#[test]
fn test_bool_wrapper_returns_false_for_past_deadline() {
    let env = env_at(3_000);
    assert!(!validate_initialization_params_bool(&env, 1_000, 2_000, 10));
}

#[test]
fn test_bool_wrapper_returns_false_for_invalid_min_contribution() {
    let env = env_at(0);
    assert!(!validate_initialization_params_bool(&env, 100, 2_000, 150));
}
