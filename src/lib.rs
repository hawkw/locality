#![feature(const_fn)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[doc(hidden)]
pub mod local;
mod stdlib;

#[cfg(not(feature = "std"))]
use stdlib::{cell::UnsafeCell, fmt, marker::PhantomData, sync::atomic::Ordering};

#[cfg(not(feature = "std"))]
pub use local::{reset, set_locality};

#[cfg(not(feature = "std"))]
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Id {
    value: usize,
    _not_send: PhantomData<UnsafeCell<()>>,
}

/// A locality implementation.
#[cfg(not(feature = "std"))]
pub trait Locality {
    const MAX_LOCALITIES: usize;

    fn current() -> Id;
}

#[cfg(not(feature = "std"))]
impl Id {
    /// # Safety
    ///
    /// The caller is _required_ to uphold the guarantee that the provided usize
    /// value uniquely identifies a locality context. If two or more
    /// concurrently executing contexts can be assigned the same  guarantee that each context that can
    /// concurrently access local data has its own unique ID value.
    pub unsafe fn from_usize(value: usize) -> Self {
        assert!(value <= local::MAX_LOCALITIES.load(Ordering::Acquire));
        Self {
            value,
            _not_send: PhantomData,
        }
    }

    pub(crate) unsafe fn into_usize(self) -> usize {
        self.value
    }
}

#[cfg(not(feature = "std"))]
impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Id").field("value", &self.value).finish()
    }
}
