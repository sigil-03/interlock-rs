#![no_std]

use thiserror::Error;

use core::cell::RefCell;

/// the interlockable trait defines the behavior that the inner type T of the [`Interlock<T>`]
/// is required to implement.
pub trait Interlockable {
    type Error;
    /// return Ok(()) if T is in a state that allows clearing the interlock
    fn is_clear(&self) -> Result<(), Self::Error>;
}

/// interlock crate errors
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Failed to clear interlock")]
    ClearError,
}

/// the interlock state. pretty much what it says on the tin - either active or inactive
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterlockState {
    Inactive,
    Active,
}

/// The interlock struct. Owns a type T which is the underlying value we interlock off of
pub struct Interlock<T: Interlockable + Clone> {
    inner: RefCell<T>,
    state: RefCell<InterlockState>,
}

impl<T> Interlock<T>
where
    T: Interlockable + Clone,
{
    pub fn new(inner: T) -> Self {
        Self {
            inner: RefCell::new(inner),
            state: RefCell::new(InterlockState::Inactive),
        }
    }

    /// attempt to clear the interlock. Returns:
    ///   * Ok(()) if clearing the interlock was successful
    ///   * Err(Error::ClearError) if clearing the interlock was unsuccessful
    pub fn try_clear_interlock(&self) -> Result<(), Error> {
        match self.inner.borrow().is_clear() {
            Ok(_) => {
                self.state.replace(InterlockState::Inactive);
                Ok(())
            }
            Err(_) => Err(Error::ClearError),
        }
    }

    /// sets the inner value, and asserts the interlock if the inner value is no longer clear
    pub fn set(&self, new_value: T) {
        self.inner.replace(new_value);

        // if we aren't in an active interlock state, and we
        // aren't clear anymore, assert the interlock
        if (!self.inner.borrow().is_clear().is_ok())
            && (*self.state.borrow() == InterlockState::Inactive)
        {
            self.state.replace(InterlockState::Active);
        }
    }

    /// get the state of the interlock
    pub fn get_state(&self) -> InterlockState {
        self.state.borrow().clone()
    }

    /// get a clone of the inner value
    pub fn get_inner(&self) -> T {
        self.inner.borrow().clone()
    }

    /// get a ref of the inner value
    pub fn get_inner_ref(&self) -> &T {
        todo!("Work around & vs Ref<'_, T>");
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    impl Interlockable for bool {
        type Error = &'static str;
        fn is_clear(&self) -> Result<(), Self::Error> {
            match self {
                true => Err("Not clear!"),
                false => Ok(()),
            }
        }
    }

    #[test]
    /// test that clearing an interlock fails if the underlying value is not clear
    fn try_clear() {
        // happy case
        let i1: Interlock<bool> = Interlock::new(false);
        let r = i1.try_clear_interlock();
        assert_eq!(r, Ok(()));

        // sad case
        let i1: Interlock<bool> = Interlock::new(true);
        let r = i1.try_clear_interlock();
        assert_eq!(r, Err(Error::ClearError))
    }

    #[test]
    /// test that changing the inner value to a non-clear value asserts the interlock
    /// and that the interlock stays asserted after the value goes back to clear
    fn new_value_sets_interlock() {
        let i1: Interlock<bool> = Interlock::new(false);
        assert_eq!(i1.get_state(), InterlockState::Inactive);
        i1.set(true);
        assert_eq!(i1.get_state(), InterlockState::Active);
        i1.set(false);
        assert_eq!(i1.get_state(), InterlockState::Active);
    }
}
