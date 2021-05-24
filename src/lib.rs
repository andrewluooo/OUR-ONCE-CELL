use std::cell::UnsafeCell;


mod unsync {
    use super::UnsafeCell;

    pub struct OnceCell<T> {
        inner: UnsafeCell<Option<T>>,
    }

    impl <T> OnceCell<T> {
        pub fn new() -> Self {
            Self {
                inner: UnsafeCell::new(None),
            }
        }

        pub fn get(&self) -> Option<&T> {
            let ptr = self.inner.get();
            // SAFETY
            unsafe { &*ptr }.as_ref()
        }

        pub fn get_mut(&mut self) -> Option<&mut T> {
            let ptr = self.inner.get();
            // SAFETY
            unsafe { &mut *ptr }.as_mut()
        }

        pub fn set(&self, value: T) -> Result<(), T> {
            if self.get().is_some() {
                return Err(value);
            }
            let r = unsafe { &mut *self.inner.get() };
            let old = std::mem::replace(r, Some(value));
            debug_assert!(old.is_none());
            Ok(())
        }
    }
}


mod sync {
    use super::UnsafeCell;
    use std::option::Option::Some;
    use std::sync::Once;

    pub struct OnceCell<T> {
        inner: UnsafeCell<Option<T>>,
        once: Once,
    }

    unsafe impl <T> Sync for OnceCell<T> {}

    impl<T> OnceCell<T> {
        pub fn new() -> Self {
            Self {
                inner: UnsafeCell::new(None),
                once: Once::new(),
            }
        }

        pub fn get(&self) -> Option<&T> {
            if self.once.is_completed() {
                unsafe { &(*self.inner.get()) }.as_ref()
            } else {
                None
            }
        }

        pub fn set(&self, value: T) -> Result<(), T> {
            if self.once.is_completed() {
                return Err(value)
            }

            let mut value = Some(value);
            self.once.call_once(|| {
                let inner = unsafe { &mut *self.inner.get() };
                debug_assert!(std::mem::replace(inner, value.take()).is_none());
            });

            match value {
                None => Ok(()),
                Some(v) => {
                    debug_assert!(self.once.is_completed());
                    Err(v)
                },
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut once: unsync::OnceCell<String> = unsync::OnceCell::new();

        assert!(once.get().is_none());
        assert!(once.set(String::new()).is_ok());
        assert!(once.set(String::new()).is_err());
        assert!(once.get().is_some());
    }

    #[test]
    fn sync_works() {
        use std::sync::Arc;
        let once = Arc::new(sync::OnceCell::new());

        let one = Arc::clone(&once);
        std::thread::spawn(move || {
            println!("{:?}", one.set(String::from("Hello")));
        });

        let two = Arc::clone(&once);
        std::thread::spawn(move || {
            println!("{:?}", two.set(String::from("World")));
        });

        std::thread::sleep(std::time::Duration::from_millis(10));

        println!("{:?}", once.get());
    }
}
