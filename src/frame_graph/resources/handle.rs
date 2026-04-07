use std::marker::PhantomData;

pub struct Handle<T> {
    data: usize,
    _marker: PhantomData<T>,
}
