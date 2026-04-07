use std::marker::PhantomData;



pub struct TemporalFrameGraphResources<'frame> {
    _marker: PhantomData<&'frame ()>
}