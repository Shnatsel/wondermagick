//! Helper functions for using `quickcheck`'s `Arbitrary` trait

use quickcheck::Arbitrary;

#[must_use]
pub fn optional_positive_float(gen: &mut quickcheck::Gen) -> Option<f64> {
    if bool::arbitrary(gen) {
        Some(positive_float(gen))
    } else {
        None
    }
}

#[must_use]
pub fn positive_float(gen: &mut quickcheck::Gen) -> f64 {
    nonzero_float(gen).abs()
}

#[must_use]
pub fn finite_float(gen: &mut quickcheck::Gen) -> f64 {
    let raw = f64::arbitrary(gen);
    if raw.is_infinite() || raw.is_nan() {
        0.0
    } else {
        raw
    }
}

#[must_use]
pub fn nonzero_float(gen: &mut quickcheck::Gen) -> f64 {
    let float = finite_float(gen);
    if float == 0.0 || float == -0.0 {
        1.0
    } else {
        float
    }
}

#[must_use]
pub fn optional_nonzero_float(gen: &mut quickcheck::Gen) -> Option<f64> {
    if bool::arbitrary(gen) {
        Some(nonzero_float(gen))
    } else {
        None
    }
}
