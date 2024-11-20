type Num = u64;
type Float = f64;

pub trait Bounded {
    fn current(&self) -> Num;
    fn max(&self) -> Num;
    fn num_steps(&self) -> Num;
    fn with_current(&self, current: Num) -> Self;
}

pub trait Stepper {
    fn step_up(&self) -> Num;
    fn step_down(&self) -> Num;
}

impl<B> Stepper for B
where
    B: Bounded,
{
    fn step_up(&self) -> Num {
        if self.current() == 0 {
            return 1;
        }
        let mut step = self.current_step();
        let mut new_b = self.current();

        while new_b <= self.current() && step <= self.num_steps() as _ {
            step += 1;
            new_b = self.brightness_at(step);
        }

        let new_b = new_b.min(self.max());

        let lower = self.with_current(new_b).step_down();
        if new_b > lower && lower > self.current() {
            return lower;
        }
        new_b
    }

    fn step_down(&self) -> Num {
        if self.current() == 1 {
            return 0;
        }
        let mut step = self.current_step();
        let mut new_b = self.current();

        while new_b >= self.current() && step >= 1 {
            step -= 1;
            new_b = self.brightness_at(step);
        }
        new_b
    }
}

trait LogLux {
    fn current_step(&self) -> isize;
    fn brightness_at(&self, step_no: isize) -> Num;
}

impl<B> LogLux for B
where
    B: Bounded,
{
    fn current_step(&self) -> isize {
        let log_ = (self.current().max(1) as Float).log(self.max() as Float);
        (self.num_steps() as Float * log_).ceil() as _
    }

    fn brightness_at(&self, step_no: isize) -> Num {
        (self.max() as Float).powf(step_no as Float / self.num_steps() as Float) as _
    }
}

// Quickcheck-based test suite that verifies the following properties:
//
// 1. The brightness after `step_up()` is *always* greater than or equal to the old brightness.
//
// 2. The brightness after `step_down()` is *always* less than or equal to the old brightness.
//
// 3. Walking the entire scale from 0 to max with `step_up()` results in *precisely*
// the same number of steps AND the same step values as walking the entire scale
// from max to 0 with `step_down()`.
//
// This offers sufficient proof that the resulting scale is completely determined
// by just the max value and the number of steps, and thus we don't need to pre-compute it
// for correctness.
#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use quickcheck::*;

    use super::*;

    type BaseNum = u16;

    const MAX_MAX: BaseNum = 1 << 12;

    #[derive(Copy, Clone, Debug)]
    struct MockBounded {
        current: Num,
        max: Num,
        num_steps: Num,
    }

    impl Bounded for MockBounded {
        fn current(&self) -> Num { self.current }
        fn max(&self) -> Num { self.max }
        fn num_steps(&self) -> Num { self.num_steps }
        fn with_current(&self, current: Num) -> Self { Self { current, ..*self } }
    }

    impl Arbitrary for MockBounded {
        fn arbitrary(g: &mut Gen) -> Self {
            let current = BaseNum::arbitrary(g) % MAX_MAX;
            Self {
                current: current as _,
                max: (current + BaseNum::arbitrary(g) % MAX_MAX) as _,
                num_steps: (BaseNum::arbitrary(g) % MAX_MAX) as _,
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let orig = *self;
            Box::new(orig.num_steps.shrink().filter(|&num_steps| num_steps > 0).flat_map(
                move |num_steps| {
                    orig.current.shrink().flat_map(move |current| {
                        orig.max
                            .shrink()
                            .filter(move |&max| max >= current)
                            .map(move |max| MockBounded { current, max, num_steps })
                    })
                },
            ))
        }
    }

    fn step_up_higher(sut: MockBounded) -> TestResult {
        TestResult::from_bool(sut.step_up() >= sut.current())
    }

    #[test]
    fn test_step_up_higher() { quickcheck(step_up_higher as fn(_) -> TestResult); }

    fn step_down_lower(sut: MockBounded) -> TestResult {
        TestResult::from_bool(sut.step_down() <= sut.current())
    }

    #[test]
    fn test_step_down_lower() { quickcheck(step_down_lower as fn(_) -> TestResult); }

    fn step_invariantly(mut sut: MockBounded) -> TestResult {
        if sut.num_steps == 0 {
            return TestResult::discard();
        }
        sut.current = 0;

        let mut up_set = HashSet::new();
        up_set.insert(0);
        while sut.current < sut.max {
            sut = sut.with_current(sut.step_up());
            up_set.insert(sut.current);
        }

        let mut down_set = HashSet::new();
        down_set.insert(sut.max);
        while sut.current > 0 {
            sut = sut.with_current(sut.step_down());
            down_set.insert(sut.current);
        }

        TestResult::from_bool(up_set.eq(&down_set))
    }

    #[test]
    fn test_step_invariantly() { quickcheck(step_invariantly as fn(_) -> TestResult); }
}
