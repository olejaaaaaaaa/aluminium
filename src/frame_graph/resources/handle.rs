use std::marker::PhantomData;


#[derive(Clone, Copy, Default, Debug )]
pub struct Handle<T> {
    data: usize,
    _marker: PhantomData<T>,
}

