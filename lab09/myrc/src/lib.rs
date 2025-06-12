use std::ops::Deref;

struct Inner<T> {
    refcount: usize,
    data: T,
}

pub struct MyRc<T> {
    inner: *mut Inner<T>,
}

impl<T> MyRc<T> {
    pub fn new(value: T) -> Self {
        // TODO: Create a MyRc. You will need to:
        //  - use Box::into_raw to create an Inner
        //  - set refcount to an appropriate value.

        // Create a boxed Inner with refcount 1
        let inner = Box::new(Inner {
            refcount: 1,
            data: value,
        });

        // Convert Box into a raw pointer and store it
        MyRc {
            inner: Box::into_raw(inner),
        }
    }
}

impl<T> Clone for MyRc<T> {
    fn clone(&self) -> Self {
        // TODO: Increment the refcount,
        // and return another MyRc<T> by copying the
        // inner struct of this MyRc.

        // SAFETY: self.inner is a valid pointer to Inner<T>
        unsafe {
            (*self.inner).refcount += 1;
        }

        // Create a new MyRc pointing to the same inner
        MyRc {
            inner: self.inner,
        }
    }
}

impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        // TODO: Decrement the refcount..
        // If it's 0, drop the Rc. You will need to use
        // Box::from_raw to do this.

        // SAFETY: self.inner is a valid pointer from Box::into_raw
        // We ensure it is only deallocated when refcount reaches 0
        unsafe {
            (*self.inner).refcount -= 1;
            if (*self.inner).refcount == 0 {
                // SAFETY: deallocate the memory once no references remain
                drop(Box::from_raw(self.inner));
            }
        }
    }
}

impl<T> Deref for MyRc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // TODO: Return a &T.

        // SAFETY: self.inner is valid, and points to an Inner<T>
        // which is guaranteed to outlive this reference
        unsafe {
            &(*self.inner).data
        }
    }
}
