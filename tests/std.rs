#![cfg_attr(not(feature = "std"), no_std)]

// Need an global allocator in no_std
#[cfg(not(feature = "std"))]
extern crate wee_alloc;

#[cfg(not(feature = "std"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(not(feature = "std"))]
use core::cell::RefCell;
#[cfg(feature = "std")]
use std::cell::RefCell;

use locality::locality;

type Span = u32;

locality! {
    static SPAN: RefCell<Option<Span>> = RefCell::new(None)
}

#[cfg(feature = "std")]
#[test]
fn span_stores_data() {
    use std::thread;

    SPAN.with(|s| {
        *s.borrow_mut() = Some(8);
    });

    let handler = thread::spawn(|| {
        SPAN.with(|span| {
            assert!(span.borrow().is_none());
        });
    });

    handler.join().unwrap();
}

#[cfg(not(feature = "std"))]
#[test]
#[should_panic]
fn span_stores_data_without_setting_locality() {
    locality::reset();
    SPAN.with(|s| {
        *s.borrow_mut() = Some(8);
    });
}

#[cfg(not(feature = "std"))]
#[test]
fn span_stores_data_after_setting_locality() {
    static mut CORE_ID: usize = 0;

    struct Foo;
    impl locality::Locality for Foo {
        const MAX_LOCALITIES: usize = 2;
        fn current() -> locality::Id {
            unsafe { locality::Id::from_usize(CORE_ID) }
        }
    }
    locality::set_locality::<Foo>().expect("first call to set_locality should succeed");

    SPAN.with(|s| {
        *s.borrow_mut() = Some(8);
    });

    SPAN.with(|s| assert_eq!(*s.borrow(), Some(8)));
    unsafe {
        CORE_ID = 1;
    }
    SPAN.with(|s| assert_eq!(*s.borrow(), None));
}
