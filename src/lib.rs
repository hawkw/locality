#[cfg(any(feature = "std", feature = "alloc"))]
pub(crate) mod local;
pub(crate) mod stdlib;
pub(crate) mod unreachable;
// pub use self::local::Local;

#[cfg(feature = "std")]
pub(crate) mod thread;
#[cfg(feature = "std")]
pub use self::thread::ThreadLocality;

use stdlib::{
    any,
    cell::UnsafeCell,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Id<L: ?Sized = DefaultLocality> {
    value: usize,
    _p: PhantomData<(fn(L), UnsafeCell<()>)>,
}

pub unsafe trait Locality {
    fn current_id() -> usize;
    fn current() -> Id<Self> {
        Id::from_usize(Self::current_id())
    }
}

impl<L: Locality + ?Sized> Id<L> {
    /// # Safety
    ///
    /// The caller is _required_ to uphold the guarantee that the provided usize
    /// value uniquely identifies a locality context. If two or more
    /// concurrently executing contexts can be assigned the same  guarantee that each context that can
    /// concurrently access local data has its own unique ID value.
    pub unsafe fn from_usize(value: usize) -> Self {
        Self {
            value,
            _p: PhantomData,
        }
    }

    pub fn current() -> Self {
        L::current()
    }

    pub(crate) fn into_usize(self) -> usize {
        self.value
    }
}

impl<L: ?Sized> fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Id")
            .field("id", &self.value)
            .field("locality", &any::type_name::<L>())
            .finish()
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::current()
    }
}

impl<L: ?Sized> Hash for Id<L> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
