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
pub struct Id {
    value: usize,
    _p: PhantomData<UnsafeCell<()>>,
}

pub unsafe trait Locality {
    fn current() -> Id;
}

pub struct SetLocalityError {
    _p: (),
}

#[cfg(not(feature = "std"))]
pub(crate) static PROVIDER: AtomicPtr<fn() -> Id> = AtomicPtr::new(ptr::null_mut());
#[cfg(feature = "std")]
pub(crate) static PROVIDER: AtomicPtr<fn() -> Id> =
    AtomicPtr::new(crate::ThreadLocality::current as *const fn() -> Id as *mut _);

pub fn set_locality<L: Locality>() -> Result<(), SetLocalityError> {
    let locality = L::current as *const fn() -> Id as *mut _;
    PROVIDER
        .compare_exchange(
            ptr::null_mut(),
            locality,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .map_err(|_| SetLocalityError { _p: () })?;
    Ok(())
}

impl Id {
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
        let f = PROVIDER.load(Ordering::Acquire);
        if f.is_null() {
            panic!(
                "when the standard library is not in use, the locality must \
                 be set by calling `set_locality` before calling \
                 `Id::current()`!"
            );
        }
        unsafe { (mem::transmute::<_, fn() -> Id>(f))() }
    }

    pub(crate) fn into_usize(self) -> usize {
        self.value
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Id")
            .field(&self.value)
            // .field("locality", &any::type_name::<L>())
            .finish()
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::current()
    }
}

impl Hash for Id {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
