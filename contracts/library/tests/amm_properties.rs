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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2_000))]

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
