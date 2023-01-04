use crate::stdlib::{
    cell::Cell,
    mem, ptr,
    sync::atomic::{AtomicPtr, Ordering},
    any::TypeId
};

use crate::{Id, Locality};

#[cfg(not(feature = "std"))]
pub(crate) static DEFAULT: AtomicPtr<fn() -> usize> = AtomicPtr::new(ptr::null_mut());
#[cfg(feature = "std")]
pub(crate) static DEFAULT: AtomicPtr<fn() -> usize> =
    AtomicPtr::new(crate::ThreadLocality::current_id as *const fn() -> Id as *mut _);


pub struct SetDefaultError {
    _p: (),
}

pub(crate) struct DefaultLocality {
    _p: (),
}

impl DefaultLocality {
    pub unsafe fn set<L: Locality>() -> Result<(), SetDefaultError> {
        assert_ne!(TypeId::of<L>, TypeId::of<DefaultLocality);
        let locality = L::current_id as *const fn() -> usize as *mut _;
        DEFAULT
            .compare_exchange(
                ptr::null_mut(),
                locality,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|_| SetDefaultError { _p: () })?;
        Ok(())
    }
}

unsafe impl Locality for DefaultLocality {
    fn current_id() -> usize {
        let f = DEFAULT_LOCALITY.load(Ordering::Acquire);
        if f.is_null() {
            panic!(
                "when the standard library is not in use, the default locality must \
                 be set by calling `DefaultLocality::set` before calling \
                 `Id::current()`!"
            );
        }
        unsafe { (mem::transmute::<_, fn() -> usize>(f))() }
    }
}
