#[cfg(not(feature = "std"))]
extern crate alloc;

pub(crate) mod local;
pub(crate) mod stdlib;
pub use self::local::Local;

#[cfg(feature = "std")]
pub(crate) mod thread;
#[cfg(feature = "std")]
pub use self::thread::ThreadLocal;

use stdlib::{any, cell::UnsafeCell, fmt, marker::PhantomData};

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Id<L: ?Sized> {
    value: usize,
    _markers: PhantomData<(fn(L), UnsafeCell<()>)>,
}

/// A locality implementation.
pub trait Locality {
    const MAX_LOCALITIES: usize;
    fn current() -> Id<Self>;
}

impl<L: Locality> Id<L> {
    /// # Safety
    ///
    /// The caller is _required_ to uphold the guarantee that the provided usize
    /// value uniquely identifies a locality context. If two or more
    /// concurrently executing contexts can be assigned the same  guarantee that each context that can
    /// concurrently access local data has its own unique ID value.
    pub unsafe fn from_usize(value: usize) -> Self {
        assert!(value <= L::MAX_LOCALITIES);
        Self {
            value,
            _markers: PhantomData,
        }
    }

    pub fn current() -> Self {
        L::current()
    }

    pub(crate) unsafe fn into_usize(self) -> usize {
        self.value
    }
}

impl<L> fmt::Debug for Id<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Id")
            .field("value", &self.value)
            .field("locality", &any::type_name::<L>())
            .finish()
    }
}
