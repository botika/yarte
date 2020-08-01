//! # Yarte wasm application
//! A simple single thread reactor pattern with a doubly-linked list as dequeue.
//!
//! Intended to be used as a singleton and static.
//!
//! ## Implement without yarte derive
//! The cycle of App methods is:
//! - `init`:
//!     - `App.__hydrate(&mut self, _addr: &'static Addr<Self>)`
//!     - `update`
//! - `on message`:
//!     - enqueue message
//!     - is ready? -> `update`
//! - `update`
//!     - pop message? -> `App.__dispatch(&mut self, _msg: Self::Message, _addr: &'static Addr<Self>)`
//!     - is queue empty?  -> `App.__render(&mut self, _addr: &'static Addr<Self>)`
//!     - is queue not empty? -> `update`. It is **not** recursive
//!
//!
//! ### Why no backpressure?
//! Because you don't need it because you don't need a runtime to poll wasm futures.
//! Backpressure can be implemented for future it is needed and carry static reference to the
//! Address of the App.
//!
//! ### Why no RC?
//! Because you don't need it because it is thinking to be implemented as singleton and static.
//!
//! ### Why doubly-linked list?
//! Is simpler than grow array implementation and it will never be a bottleneck in a browser.
//! But in the future it can be implemented.
//!
#![no_std]
extern crate alloc;

use core::cell::{Cell, UnsafeCell};
use core::default::Default;
use core::ptr;

use alloc::alloc::{dealloc, Layout};
use alloc::boxed::Box;

mod queue;

use self::queue::Queue;

/// App are object which encapsulate state and behavior
///
/// App communicate exclusively by directional exchanging messages.
///
/// It is recommended not to implement out of WASM Single Page Application context.
// TODO: derive
pub trait App: Default + Sized {
    type BlackBox;
    type Message: 'static;
    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __render(&mut self, _addr: &'static Addr<Self>) {}

    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __hydrate(&mut self, _addr: &'static Addr<Self>) {}

    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __dispatch(&mut self, _msg: Self::Message, _addr: &'static Addr<Self>) {}
}

/// The address of App
#[repr(transparent)]
pub struct Addr<A: App>(Context<A>);

/// Macro to create a `Addr<A: App>` reference to a statically allocated `App`.
///
/// This macro returns a value with type `&'static Addr<$ty>`.
///
/// # Panics
/// Have one type instance
/// Only construct to target arch `wasm32`
///
/// ```ignore
/// #[derive(App)]
/// #[template(path = "index")
/// #[msg(enum Msg { Inc, Reset })]
/// struct MyApp {
///     count: usize,
///     bb: <Self as App>::BlackBox,
/// }
///
/// fn inc(app: &mut MyApp, _addr: &Addr<MyApp>) {
///     set_count!(app, app.count + 1);
/// }
///
/// fn reset(app: &mut MyApp, _addr: &Addr<MyApp>) {
///     if app.count != 0 {
///         set_count!(app, 0);
///     }
/// }
///
/// let addr = run!(MyApp);
/// addr.send(Msg::Reset);
/// ```
#[macro_export]
macro_rules! run {
    ($ty:ty) => {
        $crate::Addr::run(<$ty as core::default::Default>::default())
    };
}

/// Static ref to mutable ptr
///
/// # Safety
/// Broke `'static` lifetime and mutability
const unsafe fn stc_to_ptr<T>(t: &'static T) -> *mut T {
    t as *const T as *mut T
}

/// Constructor and destructor
impl<A: App> Addr<A> {
    /// Make new Address for App
    ///
    /// Use at `run!` macro and for testing
    ///
    /// # Safety
    /// Produce memory leaks if return reference and its copies hasn't owner
    ///
    /// # Panics
    /// Only construct in target arch `wasm32`
    #[inline]
    unsafe fn new(a: A) -> &'static Addr<A> {
        if cfg!(not(target_arch = "wasm32")) {
            panic!("Only construct in 'wasm32'");
        }
        Box::leak(Box::new(Addr(Context::new(a))))
    }

    /// Make new Address for App and run it
    ///
    /// # Panics
    /// Only run it in target arch `wasm32`
    pub fn run(a: A) -> &'static Addr<A> {
        unsafe {
            let addr = Self::new(a);
            addr.hydrate();
            addr
        }
    }

    /// Dealloc Address
    ///
    /// Use for testing
    ///
    /// # Safety
    /// Broke `'static` lifetime
    pub unsafe fn dealloc(&'static self) {
        let p = stc_to_ptr(self);
        ptr::drop_in_place(p);
        dealloc(p as *mut u8, Layout::new::<Addr<A>>());
    }

    /// Sends a message
    ///
    /// The message is always queued
    pub fn send(&'static self, msg: A::Message) {
        self.0.q.push(msg);
        self.update();
    }

    /// Hydrate app
    /// Link events and save closures
    ///
    /// # Safety
    /// Produce **unexpected behaviour** if launched more than one time
    #[inline]
    unsafe fn hydrate(&'static self) {
        debug_assert!(!self.0.ready.get());
        // Only run one time
        self.0.app.get().as_mut().unwrap().__hydrate(self);
        self.0.ready.replace(true);
        self.update();
    }

    #[inline]
    fn update(&'static self) {
        if self.0.ready.get() {
            self.0.ready.replace(false);
            // UB is checked by ready Cell
            let app = unsafe { self.0.app.get().as_mut().unwrap() };
            while let Some(msg) = self.0.q.pop() {
                app.__dispatch(msg, self);
                while let Some(msg) = self.0.q.pop() {
                    app.__dispatch(msg, self);
                }
                app.__render(self);
            }
            self.0.ready.replace(true);
        }
    }
}

/// Encapsulate inner context of the App
pub struct Context<A: App> {
    app: UnsafeCell<A>,
    q: Queue<A::Message>,
    ready: Cell<bool>,
}

impl<A: App> Context<A> {
    fn new(app: A) -> Self {
        Self {
            app: UnsafeCell::new(app),
            q: Queue::new(),
            ready: Cell::new(false),
        }
    }
}
