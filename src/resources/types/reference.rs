pub struct Ref<'a, T>(pub(crate) parking_lot::MappedRwLockReadGuard<'a, T>);

impl<'a, T> std::ops::Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

pub struct RefMut<'a, T>(pub(crate) parking_lot::MappedRwLockWriteGuard<'a, T>);

impl<'a, T> std::ops::Deref for RefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<'a, T> std::ops::DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

