use std::marker::PhantomData;
use crate::frame_graph::temporal::Id;

pub struct Handle<T> {
    pub(crate) id: Id,
    pub(crate) _marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData
        }
    }
}

impl<T> Copy for Handle<T> {}
impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self { 
            id: Id::default(), 
            _marker: PhantomData 
        }
    }
}
impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}