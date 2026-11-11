// tests/api/stats.rs
use statrs::stats_tests::chisquare::chisquare;

/// Seeded tests are deterministic, so this is how surprising a fixed sample
/// may look before we call the generator biased, not a flake rate.
const P_VALUE_CUTOFF: f64 = 1e-4;

pub fn assert_uniform(counts: &[usize], degrees_of_freedom: Option<usize>) {
    /* chi-squared, assert p > P_VALUE_CUTOFF */
    let (_, pvalue) = chisquare(counts, None, degrees_of_freedom)
        .expect("used only in tests: manually pass valid values");

    assert!(pvalue > P_VALUE_CUTOFF, "{}", pvalue);
}
