use crate::stdlib::prelude::*;
use crate::stdlib::{cell::UnsafeCell, marker::PhantomData};
use crate::{DefaultLocality, Locality};

pub struct Local<T, L: Locality = DefaultLocality> {
    items: UnsafeCell<Vec<Box<T>>>,
    init: fn() -> T,
    _p: PhantomData<fn(L)>,
}

impl<T, L: Locality> Local<T, L> {
    pub const fn new() -> Self
    where
        T: Default,
    {
        Self::new_with_init(T::default)
    }

    pub const fn new_with_init(init: fn() -> T) -> Self {
        Self {
            items: UnsafeCell::new(Vec::new()),
            init,
            _p: PhantomData,
        }
    }

    pub fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        let idx = unsafe {
            // safety: hopefully, the provided `Locality` upholds its end of the
            // contract...
            L::current().into_usize()
        };
        unimplemented!()
        // let item = if let Some(i) = unsafe { (*self.items[idx].get()).as_ref() } {
        //     i
        // } else {
        //     let ptr = self.items[idx].get();
        //     unsafe {
        //         (*ptr) = Some((self.init)());
        //         (*ptr).as_ref().expect("we just set the pointed value!")
        //     }
        // };
        // f(item)
    }
}
