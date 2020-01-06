use crate::{Id, Locality};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread_local;

pub struct ThreadLocal {
    _p: (),
}

impl Locality for ThreadLocal {
    // const MAX_LOCALITIES: usize = crate::stdlib::usize::MAX;
    fn current() -> Id {
        thread_local! {
            static CURRENT_ID: Cell<Option<usize>> = Cell::new(None);
        }
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let id = CURRENT_ID.with(|curr| match curr.get() {
            Some(value) => value,
            None => {
                let id = NEXT.fetch_add(1, Ordering::Relaxed);
                curr.set(Some(id));
                id
            }
        });
        unsafe { Id::from_usize(id) }
    }
}
