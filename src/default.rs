use crate::stdlib::{
    cell::Cell,
    mem, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{Id, Locality};

pub(crate) static DEFAULT_LOCALITY: AtomicPtr<fn() -> Id> = AtomicPtr::new(ptr::null_mut());

pub struct SetDefaultError {
    msg: &'static str,
}

pub(crate) struct DefaultLocality {
    _p: (),
}

impl DefaultLocality {
    pub fn set<L: Locality>() -> Result<(), SetDefaultError> {
        let locality = &mut (L::current as fn() -> Id) as *mut _;
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
}

impl Locality for DefaultLocality {
    fn current() -> Id {
        let f = DEFAULT_LOCALITY.load(Ordering::Acquire);
        if f.is_null() {
            panic!("the default locality must be set by `locality::set_default()`");
        } else {
            unsafe { (*f)() }
        }
    }
}
