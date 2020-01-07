#[macro_export]
macro_rules! locality {
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => {
        $crate::locality!($(#[$attr])* $vis static $name: $t = $init);
        $crate::locality!($($rest)*);
    };
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr) => {
        #[cfg(feature = "std")]
        thread_local!($(#[$attr])* $vis static $name: $t = $init);
        #[cfg(not(feature = "std"))]
        $crate::__locality_inner!($(#[$attr])* $vis static $name, $t, $init);
    };
}

#[cfg(not(feature = "std"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __locality_inner {
    (@key $(#[$attr:meta])* $vis:vis $name:ident, $t:ty, $init:expr) => {
        {
            #[inline]
            fn __init() -> $t { $init }

            unsafe fn __getit() -> Option<&'static $t> {
                static __KEY: $crate::local::__Store<$t> =
                    $crate::local::__Store::new();

                __KEY.get(__init)
            }

            unsafe {
                $crate::local::LocalKey::new(__getit)
            }
        }
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident, $t:ty, $init:expr) => {
        $(#[$attr])* $vis const $name: $crate::local::LocalKey<$t> =
            $crate::__locality_inner!(@key $(#[$attr])* $vis $name, $t, $init);
    };
}

#[cfg(not(feature = "std"))]
mod no_std {
    use crate::stdlib::cell::UnsafeCell;
    use crate::stdlib::{
        fmt, mem, ptr,
        sync::atomic::{AtomicPtr, AtomicU8, AtomicUsize, Ordering},
    };
    use crate::{Id, Locality};

    pub(crate) static MAX_LOCALITIES: AtomicUsize = AtomicUsize::new(0);

    static DEFAULT_LOCALITY: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

    pub struct SetDefaultError {
        msg: &'static str,
    }

    impl fmt::Debug for SetDefaultError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "SetDefaultError: {}", self.msg)
        }
    }

    pub fn set_locality<L: Locality>() -> Result<(), SetDefaultError> {
        let max = L::MAX_LOCALITIES;
        MAX_LOCALITIES
            .compare_exchange(0, max, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| SetDefaultError {
                msg: "default locality was already set!",
            })?;

        let locality = L::current as *mut ();

        DEFAULT_LOCALITY
            .compare_exchange(
                ptr::null_mut(),
                locality,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|_| SetDefaultError {
                msg: "default locality was already set!",
            })?;
        Ok(())
    }

    pub fn reset() {
        MAX_LOCALITIES.store(0, Ordering::Acquire);
        DEFAULT_LOCALITY.store(ptr::null_mut(), Ordering::Acquire);
    }

    #[doc(hidden)]
    pub struct __Store<T: Sized> {
        items: UnsafeCell<crate::stdlib::vec::Vec<Option<T>>>,
        initialized: AtomicU8,
    }

    unsafe impl<T: Sized> Sync for __Store<T> {}

    impl<T: Sized> __Store<T> {
        pub const fn new() -> Self {
            let items = UnsafeCell::new(crate::stdlib::vec::Vec::new());
            let initialized = AtomicU8::new(0);

            Self { items, initialized }
        }

        fn initialize(&self) {
            match self.initialized.compare_and_swap(0, 1, Ordering::Acquire) {
                2 => return,
                0 => {
                    let size = MAX_LOCALITIES.load(Ordering::Acquire);
                    assert!(size > 0, "you should have called set_locality");
                    unsafe { &mut *(self.items.get()) }.resize_with(size, || None);
                    self.initialized.store(2, Ordering::Release);
                }
                1 => while self.initialized.load(Ordering::Acquire) != 2 {},
                _ => panic!("invalid value"),
            }
        }

        pub unsafe fn get(&self, init: fn() -> T) -> Option<&'static T> {
            self.initialize();

            let f = DEFAULT_LOCALITY.load(Ordering::Relaxed);
            let f = f as *const ();
            let f = mem::transmute::<*const (), fn() -> Id>(f);
            let idx = f().into_usize();

            let value: &mut Option<T> = &mut ((&mut *self.items.get())[idx]);
            if value.is_none() {
                *value = Some(init());
            }
            value.as_ref()
        }
    }

    pub struct LocalKey<T: 'static> {
        inner: unsafe fn() -> Option<&'static T>,
    }

    impl<T: 'static> LocalKey<T> {
        #[doc(hidden)]
        pub const unsafe fn new(inner: unsafe fn() -> Option<&'static T>) -> LocalKey<T> {
            LocalKey { inner }
        }

        pub fn with<F, R>(&'static self, f: F) -> R
        where
            F: FnOnce(&T) -> R,
        {
            self.try_with(f).expect(
                "cannot access a Thread Local Storage value \
                                 during or after destruction",
            )
        }

        pub fn try_with<F, R>(&'static self, f: F) -> Result<R, AccessError>
        where
            F: FnOnce(&T) -> R,
        {
            unsafe {
                let thread_local = (self.inner)().ok_or(AccessError { _private: () })?;
                Ok(f(thread_local))
            }
        }
    }

    #[derive(Clone, Copy, Eq, PartialEq)]
    pub struct AccessError {
        _private: (),
    }

    impl fmt::Debug for AccessError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("AccessError").finish()
        }
    }
}

#[cfg(not(feature = "std"))]
pub use no_std::*;
