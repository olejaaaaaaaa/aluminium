use std::marker::PhantomData;
use slotmap::new_key_type;

new_key_type! {
    pub struct Id;
}

pub struct Handle<T> {
    pub(crate) id: Id,
    pub(crate) version: u64,
    pub(crate) _marker: PhantomData<T>,
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            version: self.version,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Handle<T> {}
impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self {
            id: Id::default(),
            version: 0,
            _marker: PhantomData,
        }
    }
}
impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}, {}", self.id, self.version)
    }
}
