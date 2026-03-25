//! # crowdfund_initialize_function
//!
//! Validation helpers for the `CrowdfundContract::initialize` entry-point.
//!
//! ## Responsibility
//! This module owns all *pre-storage* checks that must pass before any state
//! is written during campaign initialization.  Keeping them here makes the
//! logic unit-testable without deploying a full contract environment.
//!
//! ## Security assumptions
//! * `goal` and `min_contribution` are denominated in the token's smallest
//!   indivisible unit (stroops for XLM-based tokens).
//! * `deadline` is a ledger UNIX timestamp (seconds since epoch).
//! * The caller is responsible for calling `creator.require_auth()` **before**
//!   invoking these helpers.
//! * Platform fee is expressed in basis points (1 bp = 0.01 %).  The maximum
//!   allowed value is 10 000 (= 100 %).

use soroban_sdk::Env;

// ── Error codes ──────────────────────────────────────────────────────────────

/// Reasons why `validate_initialization_params` can fail.
///
/// Each variant maps to a human-readable message returned by
/// [`validation_error_message`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InitError {
    /// `goal` must be a positive integer.
    GoalNotPositive,
    /// `deadline` must be strictly after the current ledger timestamp.
    DeadlineInPast,
    /// `min_contribution` must be a positive integer.
    MinContributionNotPositive,
    /// `min_contribution` must not exceed `goal`.
    MinContributionExceedsGoal,
    /// Platform fee basis points must be ≤ 10 000.
    PlatformFeeExceedsMax,
    /// `bonus_goal`, when provided, must be strictly greater than `goal`.
    BonusGoalNotGreaterThanGoal,
}

impl InitError {
    /// Returns a developer-facing description of the error.
    pub fn message(self) -> &'static str {
        match self {
            Self::GoalNotPositive => "goal must be greater than zero",
            Self::DeadlineInPast => "deadline must be in the future",
            Self::MinContributionNotPositive => "min_contribution must be greater than zero",
            Self::MinContributionExceedsGoal => "min_contribution must not exceed goal",
            Self::PlatformFeeExceedsMax => "platform fee cannot exceed 100% (10 000 bps)",
            Self::BonusGoalNotGreaterThanGoal => "bonus_goal must be greater than primary goal",
        }
    }
}

// ── Core validation ──────────────────────────────────────────────────────────

/// Validates all initialization parameters for a new crowdfunding campaign.
///
/// # Parameters
/// | Name                | Type          | Description                                      |
/// |---------------------|---------------|--------------------------------------------------|
/// | `env`               | `&Env`        | Soroban environment (used for ledger timestamp). |
/// | `goal`              | `i128`        | Funding target in token's smallest unit.         |
/// | `deadline`          | `u64`         | Campaign end time as a UNIX ledger timestamp.    |
/// | `min_contribution`  | `i128`        | Minimum single contribution amount.              |
/// | `platform_fee_bps`  | `Option<u32>` | Optional platform fee in basis points (0–10 000).|
/// | `bonus_goal`        | `Option<i128>`| Optional stretch goal; must exceed `goal`.       |
///
/// # Returns
/// `Ok(())` when all parameters are valid.
/// `Err(InitError)` with the first failing constraint.
///
/// # Errors
/// * [`InitError::GoalNotPositive`]            – `goal <= 0`
/// * [`InitError::DeadlineInPast`]             – `deadline <= current_timestamp`
/// * [`InitError::MinContributionNotPositive`] – `min_contribution <= 0`
/// * [`InitError::MinContributionExceedsGoal`] – `min_contribution > goal`
/// * [`InitError::PlatformFeeExceedsMax`]      – `fee_bps > 10_000`
/// * [`InitError::BonusGoalNotGreaterThanGoal`]– `bonus_goal <= goal`
///
/// # Security
/// * Does **not** write any storage — purely read-only.
/// * Checks are ordered from cheapest to most expensive.
/// * Integer comparisons use native Rust operators; no overflow is possible
///   because all values are validated against known bounds before arithmetic.
pub fn validate_initialization_params(
    env: &Env,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    platform_fee_bps: Option<u32>,
    bonus_goal: Option<i128>,
) -> Result<(), InitError> {
    if goal <= 0 {
        return Err(InitError::GoalNotPositive);
    }

    let current_time = env.ledger().timestamp();
    if deadline <= current_time {
        return Err(InitError::DeadlineInPast);
    }

    if min_contribution <= 0 {
        return Err(InitError::MinContributionNotPositive);
    }

    if min_contribution > goal {
        return Err(InitError::MinContributionExceedsGoal);
    }

    if let Some(fee_bps) = platform_fee_bps {
        if fee_bps > 10_000 {
            return Err(InitError::PlatformFeeExceedsMax);
        }
    }

    if let Some(bg) = bonus_goal {
        if bg <= goal {
            return Err(InitError::BonusGoalNotGreaterThanGoal);
        }
    }

    Ok(())
}

/// Convenience wrapper that mirrors the old `bool`-returning signature.
///
/// Kept for backward compatibility with call-sites that only need a pass/fail
/// answer and do not need to distinguish between error kinds.
///
/// # Deprecation
/// Prefer [`validate_initialization_params`] which returns a typed error.
pub fn validate_initialization_params_bool(
    env: &Env,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
) -> bool {
    validate_initialization_params(env, goal, deadline, min_contribution, None, None).is_ok()
}
