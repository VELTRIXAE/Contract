//! Property-based tests for the constant-product quote math.
//!
//! Bounds keep every intermediate product inside i128: the largest is
//! reserve_in * amount_out * 1000 in get_amount_in, so values stay <= 1e17.

use proptest::prelude::*;
use veltrixae_library::{get_amount_in, get_amount_out, quote, SoroswapLibraryError};

const MAX: i128 = 100_000_000_000_000_000; // 1e17

fn fee(amount_in: i128) -> i128 {
    (amount_in * 3 + 999) / 1000
}

fn property_config() -> ProptestConfig {
    let mut config = ProptestConfig::default();
    config.cases = config.cases.max(2_000);
    // Filtered reserve/output combinations legitimately produce inputs outside
    // the bounded domain. Scale that budget with accepted stress-test cases;
    // never reduce the number of accepted cases or weaken an assertion.
    config.max_global_rejects = config.max_global_rejects.max(config.cases);
    config
}

proptest! {
    #![proptest_config(property_config())]

    /// The pool never pays out its whole reserve.
    #[test]
    fn amount_out_is_strictly_less_than_reserve(
        amount_in in 1..=MAX, r_in in 1..=MAX, r_out in 1..=MAX
    ) {
        let out = get_amount_out(amount_in, r_in, r_out).unwrap();
        prop_assert!(out >= 0);
        prop_assert!(out < r_out);
    }

    /// A swap sized by get_amount_out always satisfies the pair's K check.
    #[test]
    fn amount_out_preserves_k(
        amount_in in 1..=MAX, r_in in 1..=MAX, r_out in 1..=MAX
    ) {
        let out = get_amount_out(amount_in, r_in, r_out).unwrap();
        let lhs = (r_in + amount_in - fee(amount_in)) * (r_out - out);
        prop_assert!(lhs >= r_in * r_out);
    }

    /// One more unit of output than get_amount_out returns breaks K
    /// (the quote is tight, i.e. it does not leave value on the table).
    #[test]
    fn amount_out_is_tight(
        amount_in in 1..=MAX, r_in in 1..=MAX, r_out in 2..=MAX
    ) {
        let out = get_amount_out(amount_in, r_in, r_out).unwrap();
        prop_assume!(out + 1 < r_out);
        let lhs = (r_in + amount_in - fee(amount_in)) * (r_out - out - 1);
        prop_assert!(lhs < r_in * r_out);
    }

    /// Paying get_amount_in(out) is always enough to receive `out`.
    #[test]
    fn amount_in_round_trips(
        r_in in 1..=MAX, r_out in 2..=MAX, out_frac in 1u32..=999_999
    ) {
        let out = (r_out - 1) * out_frac as i128 / 1_000_000;
        prop_assume!(out > 0);
        let amount_in = get_amount_in(out, r_in, r_out).unwrap();
        prop_assert!(amount_in > 0);
        prop_assume!(amount_in <= MAX);
        prop_assert!(get_amount_out(amount_in, r_in, r_out).unwrap() >= out);
    }

    /// Existing sufficient quotes retain their exact value; only underfunded
    /// historical quotes may increase. The pool's integer-fee output is the oracle.
    #[test]
    fn sufficient_historical_exact_output_quotes_are_preserved(
        r_in in 1..=MAX, r_out in 2..=MAX, out_frac in 1u32..=999_999
    ) {
        let out = (r_out - 1) * out_frac as i128 / 1_000_000;
        prop_assume!(out > 0);
        let n = r_in * out * 1000;
        let d = (r_out - out) * 997;
        let old = n / d + i128::from(n % d != 0) + 1;
        prop_assume!(old <= MAX);
        let current = get_amount_in(out, r_in, r_out).unwrap();
        prop_assert!(current >= old);
        if get_amount_out(old, r_in, r_out).unwrap() >= out {
            prop_assert_eq!(current, old);
        }
    }

    /// get_amount_in never returns a negative or zero amount and rejects
    /// outputs that would empty the reserve.
    #[test]
    fn amount_in_rejects_draining_outputs(
        r_in in 1..=MAX, r_out in 1..=MAX, extra in 0..=MAX
    ) {
        let out = r_out + extra;
        prop_assert_eq!(
            get_amount_in(out, r_in, r_out),
            Err(SoroswapLibraryError::InsufficientLiquidity)
        );
    }

    /// quote rounds down: amount_b * reserve_a <= amount_a * reserve_b.
    #[test]
    fn quote_rounds_down(
        amount_a in 1..=MAX, r_a in 1..=MAX, r_b in 1..=1_000_000_000_000_000i128
    ) {
        let b = quote(amount_a, r_a, r_b).unwrap();
        prop_assert!(b >= 0);
        prop_assert!(b * r_a <= amount_a * r_b);
        prop_assert!((b + 1) * r_a > amount_a * r_b);
    }

    /// Larger inputs never produce smaller outputs.
    #[test]
    fn amount_out_is_monotonic(
        a in 1..=MAX / 2, d in 0..=MAX / 2, r_in in 1..=MAX, r_out in 1..=MAX
    ) {
        let o1 = get_amount_out(a, r_in, r_out).unwrap();
        let o2 = get_amount_out(a + d, r_in, r_out).unwrap();
        prop_assert!(o2 >= o1);
    }

    /// Non-positive inputs are rejected with typed errors, never a panic.
    #[test]
    fn non_positive_inputs_are_rejected(x in i128::MIN / 2..=0, r in 1..=MAX) {
        prop_assert_eq!(get_amount_out(x, r, r), Err(SoroswapLibraryError::InsufficientInputAmount));
        prop_assert_eq!(get_amount_in(x, r, r), Err(SoroswapLibraryError::InsufficientOutputAmount));
        prop_assert_eq!(quote(x, r, r), Err(SoroswapLibraryError::InsufficientAmount));
    }
}

#[test]
fn exact_output_quote_covers_integer_fee_rounding_counterexample() {
    let (r_in, r_out, out) = (
        72_320_497_631_014_911,
        94_716_774_993_543_284,
        28_883_596_383_956_043,
    );
    let old = 31_825_313_508_430_334;
    assert_eq!(get_amount_out(old, r_in, r_out), Ok(out - 1));
    let corrected = get_amount_in(out, r_in, r_out).unwrap();
    assert_eq!(corrected, old + 1);
    assert_eq!(get_amount_out(corrected, r_in, r_out), Ok(out));
}
