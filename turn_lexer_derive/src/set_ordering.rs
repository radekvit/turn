use std::cmp::Ordering;

/// A partial set ordering trait.
///
/// By implementing `set_ordering`, we get subset and superset predicates for free.
/// In some cases, we may want to provide a total ordering for storing sets in maps, but we may
/// still need to determine when they are subsets of each other.
pub trait SetOrdering<Rhs = Self> {
    /// Determines the ordering of the two sets.
    fn set_ordering(&self, other: &Rhs) -> Option<Ordering>;

    /// Returns true if the left hand side is a subset of the right hand side.
    fn is_subset(&self, other: &Rhs) -> bool {
        match self.set_ordering(other) {
            Some(Ordering::Less) | Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    /// Returns true if the left hand side is a strict subset of the right hand side.
    fn is_strict_subset(&self, other: &Rhs) -> bool {
        self.set_ordering(other) == Some(Ordering::Less)
    }

    /// Returns true if the two sets are not comparable.
    fn is_uncomparable(&self, other: &Rhs) -> bool {
        self.set_ordering(other) == None
    }

    /// Returns true if the two sets are equal.
    fn is_equal(&self, other: &Rhs) -> bool {
        self.set_ordering(other) == Some(Ordering::Equal)
    }

    /// Returns true if the left hand side is a superset of the left hand side.
    fn is_superset(&self, other: &Rhs) -> bool {
        match self.set_ordering(other) {
            Some(Ordering::Greater) | Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    /// Returns true if the left hand side is a strict superset of the left hand side.
    fn is_strict_superset(&self, other: &Rhs) -> bool {
        self.set_ordering(other) == Some(Ordering::Greater)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use std::cmp::Ordering;

    #[derive(Clone, Copy)]
    struct TwoDimensionalIntegerSet(i32, i32);

    impl SetOrdering for TwoDimensionalIntegerSet {
        fn set_ordering(&self, other: &Self) -> Option<Ordering> {
            if self.0 == other.0 && self.1 == other.1 {
                return Some(Ordering::Equal);
            }
            if self.0 <= other.0 && self.1 <= other.1 {
                return Some(Ordering::Less);
            }
            if self.0 >= other.0 && self.1 >= other.1 {
                return Some(Ordering::Greater);
            }
            None
        }
    }
    #[test]
    fn set_ordering_test() {
        use TwoDimensionalIntegerSet as TDIS;

        // pairs of subset and superset
        let subset0 = (TDIS(0, -1), TDIS(0, 0));
        let subset1 = (TDIS(0, -1), TDIS(1, -1));
        let subset2 = (TDIS(0, -1), TDIS(1, 0));
        // pair of uncomparable structs
        let uncomparable = (TDIS(42, 24), TDIS(24, 42));
        // pair of equal structs
        let equal = (TDIS(42, 24), TDIS(42, 24));
        // pairs of superset and subset
        let superset0 = (TDIS(55, 666), TDIS(55, 0));
        let superset1 = (TDIS(55, 666), TDIS(0, 666));
        let superset2 = (TDIS(55, 666), TDIS(0, 0));

        assert_eq!(subset0.0.set_ordering(&subset0.1), Some(Ordering::Less));
        assert_eq!(subset1.0.set_ordering(&subset1.1), Some(Ordering::Less));
        assert_eq!(subset2.0.set_ordering(&subset2.1), Some(Ordering::Less));
        assert_eq!(uncomparable.0.set_ordering(&uncomparable.1), None);
        assert_eq!(equal.0.set_ordering(&equal.1), Some(Ordering::Equal));
        assert_eq!(
            superset0.0.set_ordering(&superset0.1),
            Some(Ordering::Greater)
        );
        assert_eq!(
            superset1.0.set_ordering(&superset1.1),
            Some(Ordering::Greater)
        );
        assert_eq!(
            superset2.0.set_ordering(&superset2.1),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn is_subset_test() {
        use TwoDimensionalIntegerSet as TDIS;

        // pairs of subset and superset
        let subset0 = (TDIS(0, -1), TDIS(0, 0));
        let subset1 = (TDIS(0, -1), TDIS(1, -1));
        let subset2 = (TDIS(0, -1), TDIS(1, 0));
        // pair of uncomparable structs
        let uncomparable = (TDIS(42, 24), TDIS(24, 42));
        // pair of equal structs
        let equal = (TDIS(42, 24), TDIS(42, 24));
        // pairs of superset and subset
        let superset0 = (TDIS(55, 666), TDIS(55, 0));
        let superset1 = (TDIS(55, 666), TDIS(0, 666));
        let superset2 = (TDIS(55, 666), TDIS(0, 0));

        assert!(subset0.0.is_subset(&subset0.1));
        assert!(subset1.0.is_subset(&subset1.1));
        assert!(subset2.0.is_subset(&subset2.1));
        assert!(!uncomparable.0.is_subset(&uncomparable.1));
        assert!(equal.0.is_subset(&equal.1));
        assert!(!superset0.0.is_subset(&superset0.1));
        assert!(!superset1.0.is_subset(&superset1.1));
        assert!(!superset2.0.is_subset(&superset2.1));
    }
    #[test]
    fn is_strict_subset_test() {
        use TwoDimensionalIntegerSet as TDIS;

        // pairs of subset and superset
        let subset0 = (TDIS(0, -1), TDIS(0, 0));
        let subset1 = (TDIS(0, -1), TDIS(1, -1));
        let subset2 = (TDIS(0, -1), TDIS(1, 0));
        // pair of uncomparable structs
        let uncomparable = (TDIS(42, 24), TDIS(24, 42));
        // pair of equal structs
        let equal = (TDIS(42, 24), TDIS(42, 24));
        // pairs of superset and subset
        let superset0 = (TDIS(55, 666), TDIS(55, 0));
        let superset1 = (TDIS(55, 666), TDIS(0, 666));
        let superset2 = (TDIS(55, 666), TDIS(0, 0));

        assert!(subset0.0.is_strict_subset(&subset0.1));
        assert!(subset1.0.is_strict_subset(&subset1.1));
        assert!(subset2.0.is_strict_subset(&subset2.1));
        assert!(!uncomparable.0.is_strict_subset(&uncomparable.1));
        assert!(!equal.0.is_strict_subset(&equal.1));
        assert!(!superset0.0.is_strict_subset(&superset0.1));
        assert!(!superset1.0.is_strict_subset(&superset1.1));
        assert!(!superset2.0.is_strict_subset(&superset2.1));
    }

    #[test]
    fn is_uncomparable_test() {
        use TwoDimensionalIntegerSet as TDIS;

        // pairs of subset and superset
        let subset0 = (TDIS(0, -1), TDIS(0, 0));
        let subset1 = (TDIS(0, -1), TDIS(1, -1));
        let subset2 = (TDIS(0, -1), TDIS(1, 0));
        // pair of uncomparable structs
        let uncomparable = (TDIS(42, 24), TDIS(24, 42));
        // pair of equal structs
        let equal = (TDIS(42, 24), TDIS(42, 24));
        // pairs of superset and subset
        let superset0 = (TDIS(55, 666), TDIS(55, 0));
        let superset1 = (TDIS(55, 666), TDIS(0, 666));
        let superset2 = (TDIS(55, 666), TDIS(0, 0));

        assert!(!subset0.0.is_uncomparable(&subset0.1));
        assert!(!subset1.0.is_uncomparable(&subset1.1));
        assert!(!subset2.0.is_uncomparable(&subset2.1));
        assert!(uncomparable.0.is_uncomparable(&uncomparable.1));
        assert!(!equal.0.is_uncomparable(&equal.1));
        assert!(!superset0.0.is_uncomparable(&superset0.1));
        assert!(!superset1.0.is_uncomparable(&superset1.1));
        assert!(!superset2.0.is_uncomparable(&superset2.1));
    }

    #[test]
    fn is_equal_test() {
        use TwoDimensionalIntegerSet as TDIS;

        // pairs of subset and superset
        let subset0 = (TDIS(0, -1), TDIS(0, 0));
        let subset1 = (TDIS(0, -1), TDIS(1, -1));
        let subset2 = (TDIS(0, -1), TDIS(1, 0));
        // pair of uncomparable structs
        let uncomparable = (TDIS(42, 24), TDIS(24, 42));
        // pair of equal structs
        let equal = (TDIS(42, 24), TDIS(42, 24));
        // pairs of superset and subset
        let superset0 = (TDIS(55, 666), TDIS(55, 0));
        let superset1 = (TDIS(55, 666), TDIS(0, 666));
        let superset2 = (TDIS(55, 666), TDIS(0, 0));

        assert!(!subset0.0.is_equal(&subset0.1));
        assert!(!subset1.0.is_equal(&subset1.1));
        assert!(!subset2.0.is_equal(&subset2.1));
        assert!(!uncomparable.0.is_equal(&uncomparable.1));
        assert!(equal.0.is_equal(&equal.1));
        assert!(!superset0.0.is_equal(&superset0.1));
        assert!(!superset1.0.is_equal(&superset1.1));
        assert!(!superset2.0.is_equal(&superset2.1));
    }

    #[test]
    fn is_superset_test() {
        use TwoDimensionalIntegerSet as TDIS;

        // pairs of subset and superset
        let subset0 = (TDIS(0, -1), TDIS(0, 0));
        let subset1 = (TDIS(0, -1), TDIS(1, -1));
        let subset2 = (TDIS(0, -1), TDIS(1, 0));
        // pair of uncomparable structs
        let uncomparable = (TDIS(42, 24), TDIS(24, 42));
        // pair of equal structs
        let equal = (TDIS(42, 24), TDIS(42, 24));
        // pairs of superset and subset
        let superset0 = (TDIS(55, 666), TDIS(55, 0));
        let superset1 = (TDIS(55, 666), TDIS(0, 666));
        let superset2 = (TDIS(55, 666), TDIS(0, 0));

        assert!(!subset0.0.is_superset(&subset0.1));
        assert!(!subset1.0.is_superset(&subset1.1));
        assert!(!subset2.0.is_superset(&subset2.1));
        assert!(!uncomparable.0.is_superset(&uncomparable.1));
        assert!(equal.0.is_superset(&equal.1));
        assert!(superset0.0.is_superset(&superset0.1));
        assert!(superset1.0.is_superset(&superset1.1));
        assert!(superset2.0.is_superset(&superset2.1));
    }

    #[test]
    fn is_strict_superset_test() {
        use TwoDimensionalIntegerSet as TDIS;

        // pairs of subset and superset
        let subset0 = (TDIS(0, -1), TDIS(0, 0));
        let subset1 = (TDIS(0, -1), TDIS(1, -1));
        let subset2 = (TDIS(0, -1), TDIS(1, 0));
        // pair of uncomparable structs
        let uncomparable = (TDIS(42, 24), TDIS(24, 42));
        // pair of equal structs
        let equal = (TDIS(42, 24), TDIS(42, 24));
        // pairs of superset and subset
        let superset0 = (TDIS(55, 666), TDIS(55, 0));
        let superset1 = (TDIS(55, 666), TDIS(0, 666));
        let superset2 = (TDIS(55, 666), TDIS(0, 0));

        assert!(!subset0.0.is_strict_superset(&subset0.1));
        assert!(!subset1.0.is_strict_superset(&subset1.1));
        assert!(!subset2.0.is_strict_superset(&subset2.1));
        assert!(!uncomparable.0.is_strict_superset(&uncomparable.1));
        assert!(!equal.0.is_strict_superset(&equal.1));
        assert!(superset0.0.is_strict_superset(&superset0.1));
        assert!(superset1.0.is_strict_superset(&superset1.1));
        assert!(superset2.0.is_strict_superset(&superset2.1));
    }
}
