use std::cmp::Ordering;

#[derive(Debug, Copy, Clone)]
pub struct Fraction {
    numerator: u32,
    denominator: u32,
}

impl Fraction {
    pub fn new(numerator: u32, denominator: u32) -> Self {
        Fraction {
            numerator,
            denominator,
        }
    }

    pub fn reciprocal(&self) -> Self {
        Fraction {
            numerator: self.denominator,
            denominator: self.numerator,
        }
    }
}

impl PartialEq for Fraction {
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = self.cross_multiply(other);
        a == b
    }
}

impl Eq for Fraction {}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> Ordering {
        let (a, b) = self.cross_multiply(other);
        a.cmp(&b)
    }
}

impl Fraction {
    /// Cross-multiplication to compare fractions without using floating-point arithmetic
    /// `a/b < c/d` is equivalent to `a * d < c * b`
    fn cross_multiply(&self, other: &Self) -> (u64, u64) {
        (
            u64::from(self.numerator) * u64::from(other.denominator),
            u64::from(other.numerator) * u64::from(self.denominator),
        )
    }

    pub fn to_float(&self) -> f64 {
        // We could reduce the fraction first for greater precision, but it's not clear if this will matter in practice
        self.numerator as f64 / self.denominator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraction_equality() {
        let frac1 = Fraction::new(1, 2);
        let frac2 = Fraction::new(2, 4);
        assert_eq!(frac1, frac2);

        let frac3 = Fraction::new(3, 4);
        assert_ne!(frac1, frac3);
    }

    #[test]
    fn test_fraction_comparison() {
        let frac1 = Fraction::new(1, 2);
        let frac2 = Fraction::new(3, 4);
        assert!(frac1 < frac2);
        assert!(frac2 > frac1);

        let frac3 = Fraction::new(2, 3);
        assert!(frac1 < frac3);
        assert!(frac3 > frac1);
        assert!(frac3 < frac2);
        assert!(frac2 > frac3);
    }

    #[test]
    fn test_fraction_sorting() {
        let frac1 = Fraction::new(1, 2);
        let frac2 = Fraction::new(3, 4);
        let frac3 = Fraction::new(2, 3);
        let mut fractions = vec![frac2, frac1, frac3];
        fractions.sort();
        assert_eq!(fractions, vec![frac1, frac3, frac2]);
    }

    #[test]
    fn test_fraction_ordering() {
        let frac1 = Fraction::new(1, 2);
        let frac2 = Fraction::new(3, 4);
        let frac3 = Fraction::new(2, 3);

        assert_eq!(frac1.cmp(&frac2), Ordering::Less);
        assert_eq!(frac2.cmp(&frac1), Ordering::Greater);
        assert_eq!(frac1.cmp(&frac3), Ordering::Less);
        assert_eq!(frac3.cmp(&frac2), Ordering::Less);
        assert_eq!(frac2.cmp(&frac3), Ordering::Greater);
    }
}
