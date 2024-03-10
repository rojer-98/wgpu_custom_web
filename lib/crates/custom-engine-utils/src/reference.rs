use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Ref<T: Debug> {
    val: *const T,
}

impl<T: Debug> Ref<T> {
    pub fn new(val: &T) -> Self {
        use std::ptr::addr_of;

        Self {
            val: addr_of!(*val),
        }
    }
}

impl<T: Debug> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.val.as_ref().unwrap() }
    }
}

impl<T: Debug> DerefMut for Ref<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.val.cast_mut().as_mut().unwrap() }
    }
}
