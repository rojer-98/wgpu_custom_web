use crate::hkt::{HKT, HKT3};

pub trait Functor<A, B>: HKT<A, B> {
    fn fmap<F>(self, f: F) -> <Self as HKT<A, B>>::Target
    where
        F: FnOnce(A) -> B;
}

// Apply
pub trait Apply<A, F, B>: Functor<A, B> + HKT3<A, F, B>
where
    F: FnOnce(A) -> B,
{
    fn ap(self, f: <Self as HKT3<A, F, B>>::Target2) -> <Self as HKT<A, B>>::Target;
}

// Pure
pub trait Pure<A>: HKT<A, A> {
    fn of(self) -> <Self as HKT<A, A>>::Target;
}

// Applicative
pub trait Applicative<A, F, B>: Apply<A, F, B> + Pure<A>
where
    F: FnOnce(A) -> B,
{
} // Simply derives Apply and Pure

pub trait Empty<A> {
    fn empty() -> A;
}

pub trait Monoid<A, F, B>: Empty<A> + Applicative<A, F, B>
where
    F: FnOnce(A) -> B,
{
}
