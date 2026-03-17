


pub struct Handle<T> {
    key: ResourceKey,
    _marker: PhantomData<T>
}