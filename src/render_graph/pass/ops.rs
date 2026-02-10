
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum LoadOp {
    Clear,
    Store,
    DontCare,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum StoreOp {
    Clear,
    Store,
    DontCare,
}