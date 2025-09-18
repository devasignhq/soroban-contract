use soroban_sdk::{Address, Env, String};

/// Test configuration constants and utilities for consistent testing
pub struct TestConfig;

impl TestConfig {
    /// Standard USDC amounts for testing (in stroops - 7 decimal places)
    pub const SMALL_AMOUNT: i128 = 100_0000000; // 100 USDC
    pub const MEDIUM_AMOUNT: i128 = 1000_0000000; // 1,000 USDC
    pub const LARGE_AMOUNT: i128 = 10000_0000000; // 10,000 USDC
    pub const HUGE_AMOUNT: i128 = 100000_0000000; // 100,000 USDC

    /// Minimum valid USDC amount (0.01 USDC)
    pub const MIN_VALID_AMOUNT: i128 = 100000;

    /// Maximum: Address valid USDC amount (1 billion USDC)
    pub const MAX_VALID_AMOUNT: i128 = 1_000_000_000_0000000;

    /// Standard task ID prefixes for different test scenarios
    pub const HAPPY_PATH_PREFIX: &'static str = "happy-path";
    pub const DISPUTE_PREFIX: &'static str = "dispute";
    pub const REFUND_PREFIX: &'static str = "refund";
    pub const ERROR_PREFIX: &'static str = "error";
    pub const PERFORMANCE_PREFIX: &'static str = "perf";
    pub const CONCURRENT_PREFIX: &'static str = "concurrent";

    /// Standard dispute reasons for testing
    pub const QUALITY_DISPUTE: &'static str = "Work quality does not meet requirements";
    pub const DEADLINE_DISPUTE: &'static str = "Deadline was missed without communication";
    pub const SCOPE_DISPUTE: &'static str = "Deliverables do not match agreed scope";
    pub const PAYMENT_DISPUTE: &'static str = "Payment terms were not honored";

    /// Time constants for testing (in seconds)
    pub const ONE_HOUR: u64 = 3600;
    pub const ONE_DAY: u64 = 86400;
    pub const ONE_WEEK: u64 = 604800;

    /// Performance testing constants
    pub const SMALL_BENCHMARK_SIZE: u32 = 5;
    pub const MEDIUM_BENCHMARK_SIZE: u32 = 10;
    pub const LARGE_BENCHMARK_SIZE: u32 = 25;

    /// Concurrent testing constants
    pub const SMALL_CONCURRENT_COUNT: u32 = 3;
    pub const MEDIUM_CONCURRENT_COUNT: u32 = 10;
    pub const LARGE_CONCURRENT_COUNT: u32 = 25;
}

/// Test scenario templates for consistent test setup
pub struct TestScenarios;

impl TestScenarios {
    /// Standard amounts for comprehensive testing
    pub fn standard_amounts() -> Vec<i128> {
        vec![
            TestConfig::MIN_VALID_AMOUNT,
            TestConfig::SMALL_AMOUNT,
            TestConfig::MEDIUM_AMOUNT,
            TestConfig::LARGE_AMOUNT,
            TestConfig::HUGE_AMOUNT,
        ]
    }

    /// Boundary test amounts
    pub fn boundary_amounts() -> Vec<i128> {
        vec![
            TestConfig::MIN_VALID_AMOUNT,
            TestConfig::MIN_VALID_AMOUNT + 1,
            TestConfig::MAX_VALID_AMOUNT - 1,
            TestConfig::MAX_VALID_AMOUNT,
        ]
    }

    /// Invalid amounts for negative testing
    pub fn invalid_amounts() -> Vec<i128> {
        vec![
            0,                                // Zero
            -1,                               // Negative
            TestConfig::MIN_VALID_AMOUNT - 1, // Below minimum
            TestConfig::MAX_VALID_AMOUNT + 1, // Above maximum
        ]
    }

    /// Standard dispute resolution scenarios
    pub fn dispute_resolutions() -> Vec<(&'static str, devasign_task_escrow::DisputeResolution)> {
        vec![
            (
                "pay_contributor",
                devasign_task_escrow::DisputeResolution::PayContributor,
            ),
            (
                "refund_creator",
                devasign_task_escrow::DisputeResolution::RefundCreator,
            ),
            (
                "partial_60_40",
                devasign_task_escrow::DisputeResolution::PartialPayment(600_0000000),
            ), // 60% to contributor
            (
                "partial_30_70",
                devasign_task_escrow::DisputeResolution::PartialPayment(300_0000000),
            ), // 30% to contributor
        ]
    }
}

/// Test assertion helpers for common validation patterns
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a value is within expected range
    pub fn assert_within_range<T: PartialOrd + std::fmt::Debug>(
        value: T,
        min: T,
        max: T,
        message: &str,
    ) {
        assert!(
            value >= min && value <= max,
            "{}: expected {} to be between {} and {}",
            message,
            format!("{:?}", value),
            format!("{:?}", min),
            format!("{:?}", max)
        );
    }

    /// Assert that gas usage is reasonable
    pub fn assert_reasonable_gas_usage(gas_used: u64, max_expected: u64) {
        assert!(gas_used > 0, "Gas usage should be greater than 0");
        assert!(
            gas_used <= max_expected,
            "Gas usage {} exceeds maximum expected {}",
            gas_used,
            max_expected
        );
    }

    /// Assert that performance is consistent (coefficient of variation < threshold)
    pub fn assert_performance_consistency(measurements: &[u64], max_cv_percent: f64) {
        if measurements.len() < 2 {
            return; // Can't calculate variance with less than 2 measurements
        }

        let mean = measurements.iter().sum::<u64>() as f64 / measurements.len() as f64;
        let variance = measurements
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / measurements.len() as f64;

        let std_dev = variance.sqrt();
        let cv_percent = (std_dev / mean) * 100.0;

        assert!(
            cv_percent <= max_cv_percent,
            "Performance inconsistent: CV {}% > {}%",
            cv_percent,
            max_cv_percent
        );
    }
}

/// Test data validation helpers
pub struct TestValidation;

impl TestValidation {
    /// Validate USDC amount format
    pub fn is_valid_usdc_amount(amount: i128) -> bool {
        amount >= TestConfig::MIN_VALID_AMOUNT && amount <= TestConfig::MAX_VALID_AMOUNT
    }

    /// Validate task ID format
    pub fn is_valid_task_id(task_id: &str) -> bool {
        task_id.len() == 25
    }

    /// Validate dispute reason format
    pub fn is_valid_dispute_reason(reason: &str) -> bool {
        reason.len() >= 10 && reason.len() <= 500
    }

    /// Generate valid task ID for test scenario
    pub fn generate_task_id(env: &Env, prefix: &str, index: u32) -> String {
        // Create the base format with prefix and index
        let base_id = format!("{}-{:06}", prefix, index);

        // Pad or truncate to exactly 25 characters
        let formatted_id = if base_id.len() >= 25 {
            // If too long, truncate to 25 characters
            &base_id[..25]
        } else {
            // If too short, pad with zeros to reach 25 characters
            &format!("{:0<25}", base_id)
        };

        String::from_str(&env, formatted_id)
    }

    /// Generate valid dispute reason for testing
    pub fn generate_dispute_reason(env: &Env, scenario: &str) -> String {
        match scenario {
            "quality" => String::from_str(&env, TestConfig::QUALITY_DISPUTE),
            "deadline" => String::from_str(&env, TestConfig::DEADLINE_DISPUTE),
            "scope" => String::from_str(&env, TestConfig::SCOPE_DISPUTE),
            "payment" => String::from_str(&env, TestConfig::PAYMENT_DISPUTE),
            _ => String::from_str(
                &env,
                format!("Test dispute reason for scenario: {}", scenario).as_str(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_constants() {
        // Verify amount constants are valid
        assert!(TestConfig::MIN_VALID_AMOUNT > 0);
        assert!(TestConfig::SMALL_AMOUNT > TestConfig::MIN_VALID_AMOUNT);
        assert!(TestConfig::MEDIUM_AMOUNT > TestConfig::SMALL_AMOUNT);
        assert!(TestConfig::LARGE_AMOUNT > TestConfig::MEDIUM_AMOUNT);
        assert!(TestConfig::MAX_VALID_AMOUNT > TestConfig::LARGE_AMOUNT);

        // Verify time constants
        assert_eq!(TestConfig::ONE_HOUR, 3600);
        assert_eq!(TestConfig::ONE_DAY, 24 * TestConfig::ONE_HOUR);
        assert_eq!(TestConfig::ONE_WEEK, 7 * TestConfig::ONE_DAY);
    }

    #[test]
    fn test_scenarios() {
        let standard = TestScenarios::standard_amounts();
        assert!(!standard.is_empty());
        assert!(standard
            .iter()
            .all(|&amount| TestValidation::is_valid_usdc_amount(amount)));

        let boundary = TestScenarios::boundary_amounts();
        assert!(!boundary.is_empty());

        let invalid = TestScenarios::invalid_amounts();
        assert!(!invalid.is_empty());
        assert!(invalid
            .iter()
            .all(|&amount| !TestValidation::is_valid_usdc_amount(amount)));
    }

    #[test]
    fn test_validation_helpers() {
        // Test USDC amount validation
        assert!(TestValidation::is_valid_usdc_amount(
            TestConfig::MIN_VALID_AMOUNT
        ));
        assert!(TestValidation::is_valid_usdc_amount(
            TestConfig::MAX_VALID_AMOUNT
        ));
        assert!(!TestValidation::is_valid_usdc_amount(0));
        assert!(!TestValidation::is_valid_usdc_amount(-1));

        // Test task ID validation
        assert!(TestValidation::is_valid_task_id("cmdkipba20002yl0v8pro56h9"));
        assert!(TestValidation::is_valid_task_id("cmdkrs3w200010p0wqo0rm6j3"));
        assert!(!TestValidation::is_valid_task_id("ab")); // Too short
        assert!(!TestValidation::is_valid_task_id("")); // Empty

        // Test dispute reason validation
        assert!(TestValidation::is_valid_dispute_reason(
            "This is a valid dispute reason"
        ));
        assert!(!TestValidation::is_valid_dispute_reason("Too short")); // Too short
        assert!(!TestValidation::is_valid_dispute_reason("")); // Empty
    }

    // #[test]
    // fn test_task_id_generation() {
    //     let task_id = TestValidation::generate_task_id("test", 123);
    //     assert!(TestValidation::is_valid_task_id(&task_id));
    //     assert!(task_id.starts_with("test-"));
    //     assert!(task_id.contains("000123"));
    // }

    // #[test]
    // fn test_dispute_reason_generation() {
    //     let reason = TestValidation::generate_dispute_reason("quality");
    //     assert!(TestValidation::is_valid_dispute_reason(&reason));
    //     assert_eq!(reason, TestConfig::QUALITY_DISPUTE);

    //     let custom_reason = TestValidation::generate_dispute_reason("custom");
    //     assert!(TestValidation::is_valid_dispute_reason(&custom_reason));
    //     assert!(custom_reason.contains("custom"));
    // }
}
