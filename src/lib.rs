#[cfg(test)]
mod tests {
    extern crate quickcheck;
    extern crate rand;

    use self::quickcheck::{Arbitrary, Gen, QuickCheck, StdGen};
    use super::*;

    fn simple_property(i: i64) -> bool {
        true
    }

    #[derive(Clone, Debug)]
    struct Xs(Vec<i64>);

    impl Arbitrary for Xs {
        fn arbitrary<G: Gen>(g: &mut G) -> Xs {
            let len = g.gen_range(5, 6);

            let mut xs = vec![];

            for _ in 0..len {
                xs.push(g.gen_range(1000, 7000));
            }

            Xs(xs)
        }

        fn shrink(&self) -> Box<Iterator<Item = Xs>> {
            // this is basically the default Vec shrink impl,
            // so you don't need to implement shrink for Vec's
            // if this is all you're doing

            let mut smaller = vec![];

            for i in 0..self.0.len() {
                let mut clone = self.clone();
                clone.0.remove(i);
                smaller.push(clone);
            }

            Box::new(smaller.into_iter())
        }
    }

    fn medium_property(xs: Xs) -> bool {
        for i in xs.0 {
            if i > 5000 {
                return false;
            }
        }
        true
    }

    #[test]
    fn simple_property_works() {
        QuickCheck::new()
            .gen(StdGen::new(rand::thread_rng(), 1))
            .tests(1000)
            .max_tests(10000)
            .quickcheck(simple_property as fn(i64) -> bool);
    }

    #[test]
    fn medium_property_works() {
        QuickCheck::new()
            .gen(StdGen::new(rand::thread_rng(), 1))
            .tests(1000)
            .max_tests(10000)
            .quickcheck(medium_property as fn(Xs) -> bool);
    }
}
