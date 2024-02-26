use crate::{
    functor::{Applicative, Functor},
    hkt::HKT,
};

pub trait Chain<A, B>: HKT<A, B> {
    fn chain<F>(self, f: F) -> <Self as HKT<A, B>>::Target
    where
        F: FnOnce(A) -> <Self as HKT<A, B>>::Target;
}

pub trait Monad<A, F, B>: Chain<A, B> + Applicative<A, F, B>
where
    F: FnOnce(A) -> B,
{
}

pub trait Extend<A, B>: Functor<A, B> + Sized {
    fn extend<W>(self, f: W) -> <Self as HKT<A, B>>::Target
    where
        W: FnOnce(Self) -> B;
}

pub trait Extract<A> {
    fn extract(self) -> A;
}

pub trait Comonad<A, B>: Extend<A, B> + Extract<A> {}
