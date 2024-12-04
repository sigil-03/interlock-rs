#![no_std]
use thiserror_no_std::Error;

use core::cell::Cell;

/// the interlockable trait defines the behavior that the inner type T of the [`Interlock<T>`]
/// is required to implement.
pub trait Interlockable {
    type UpdateType;
    /// return true if T is in a state that allows clearing the interlock, false otherwise
    fn is_clear(&self) -> bool;
    fn set(&self, new: Self::UpdateType);
    fn clear(&self, new: Self::UpdateType);
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
    inner: T,
    state: Cell<InterlockState>,
}

impl<T> Interlock<T>
where
    T: Interlockable + Clone,
{
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            state: Cell::new(InterlockState::Inactive),
        }
    }

    /// attempt to clear the interlock. Returns:
    ///   * Ok(()) if clearing the interlock was successful
    ///   * Err(Error::ClearError) if clearing the interlock was unsuccessful
    pub fn try_clear_interlock(&self) -> Result<(), Error> {
        match self.inner.is_clear() {
            true => {
                self.state.replace(InterlockState::Inactive);
                Ok(())
            }
            false => Err(Error::ClearError),
        }
    }

    /// sets the inner value, and asserts the interlock if the inner value is no longer clear
    pub fn set(&self, new_value: T::UpdateType) {
        self.inner.set(new_value);

        // if we aren't in an active interlock state, and we
        // aren't clear anymore, assert the interlock
        if (!self.inner.is_clear()) && (self.state.get() == InterlockState::Inactive) {
            self.state.set(InterlockState::Active);
        }
    }

    /// clear the inner value with an update type
    pub fn clear(&self, new_value: T::UpdateType) {
        self.inner.clear(new_value);
    }

    /// get the state of the interlock
    pub fn get_state(&self) -> InterlockState {
        self.state.get()
    }

    /// get a clone of the inner value
    pub fn get_inner(&self) -> T {
        self.inner.clone()
    }

    /// get a ref of the inner value
    pub fn get_inner_ref(&self) -> &T {
        todo!("Work around & vs Ref<'_, T>");
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Clone)]
    struct InterlockableBool {
        val: Cell<bool>,
    }
    impl InterlockableBool {
        fn new(val: bool) -> Self {
            Self {
                val: Cell::new(val),
            }
        }
    }

    impl Interlockable for InterlockableBool {
        type UpdateType = bool;
        fn is_clear(&self) -> bool {
            !self.val.get()
        }

        fn set(&self, new: Self::UpdateType) {
            self.val.set(new);
        }

        fn clear(&self, new: Self::UpdateType) {
            self.val.set(new);
        }
    }

    #[test]
    /// test that clearing an interlock fails if the underlying value is not clear
    fn try_clear() {
        // happy case
        let i1: Interlock<InterlockableBool> = Interlock::new(InterlockableBool::new(false));
        let r = i1.try_clear_interlock();
        assert_eq!(r, Ok(()));

        // sad case
        let i1: Interlock<InterlockableBool> = Interlock::new(InterlockableBool::new(true));
        let r = i1.try_clear_interlock();
        assert_eq!(r, Err(Error::ClearError))
    }

    #[test]
    /// test that changing the inner value to a non-clear value asserts the interlock
    /// and that the interlock stays asserted after the value goes back to clear
    fn new_value_sets_interlock() {
        let i1: Interlock<InterlockableBool> = Interlock::new(InterlockableBool::new(false));
        assert_eq!(i1.get_state(), InterlockState::Inactive);
        i1.set(true);
        assert_eq!(i1.get_state(), InterlockState::Active);
        i1.set(false);
        assert_eq!(i1.get_state(), InterlockState::Active);
    }
}
