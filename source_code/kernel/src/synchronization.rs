//! Sincrhronization file for Sync and Send threads

/// # UnsafeCell<T> from core lib
/// The core primitive for interior mutability in Rust.
///
/// If you have a reference &T, then normally in Rust the compiler performs optimizations based on
/// the knowledge that &T points to immutable data. Mutating that data, for example through an alias
/// or by transmuting an &T into an &mut T, is considered undefined behavior. UnsafeCell<T> opts-out
/// of the immutability guarantee for &T: a shared reference &UnsafeCell<T> may point to data that is being mutated.
/// This is called “interior mutability”.
///
/// The first entry into CoreCell<T> is used with "[X].into()">
use core::cell::UnsafeCell;

/// # Synchronization interfaces.
/// ## Embedded mod
pub mod interface {
    /// Any object implementing this trait guarantees exclusive access to the data wrapped within
    /// the Mutex for the duration of the provided closure.
    pub trait Mutex {
        /// The type of the data that is wrapped by this mutex.
        /// As this is just the signature, is not specified yet to which data is wrapped.
        type Data;

        /// Locks the mutex and grants the closure temporary mutable access to the wrapped data.
        fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R;
    }
}

/// A pseudo-lock for teaching purposes.
///
/// In contrast to a real Mutex implementation, does not protect against
/// concurrent access from other cores to the contained data.
///
/// The lock will only be used as long as it is safe to do so, i.e. as long as the kernel is
/// executing single-threaded, aka only running on a single core with interrupts disabled.
/// The "Sized" type is one in which the size (in RAM) is know at compile time
/// But "?Sized" is for unknown
pub struct NullLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

/// Send type: when it is "safe" to send it to another thread.
///		Send is used main for object which need used (accessed)
///		between threads ONLY AT ONCE.
///		Becuase Rust ownership model, when "A" thread send the object
///		to "B", "A" loss the ownership over it.
/// 	"Send" and "Sync" types (really; traits) are unsafe at default.
/// 	and are marker traits; they have no associated items like methods
/// 	only intrisic properties an implementor should have.
unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}

/// Sync type: when it is "safe" to share between threads.
///		Sync is used mainly for mutable objects which need syncronization
///		between threads AT THE SAME TIME.
///		This can be done thankfully to "Mutex" and "RwLock" which allow one thread
///		to proceed while others must wait.
///		To avoid data race (which to threads try to modify the data at same time)
///		Rust use a model in which you can use many immutable references to an object
unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}

impl<T> NullLock<T> {
    /// Create an instance.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

/// OS Interface Code
impl<T> interface::Mutex for NullLock<T> {
    type Data = T;

    fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R {
        // In a real lock, there would be code encapsulating this line that ensures that this
        // mutable reference will ever only be given out once at a time.
        let data = unsafe { &mut *self.data.get() };

        f(data)
    }
}
