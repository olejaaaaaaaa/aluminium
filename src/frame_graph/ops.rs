
#[derive(Debug)]
pub enum LoadOp {
    Load,
    Clear,
    DontCare,
}

#[derive(Debug)]
pub enum StoreOp {
    Store,
    DontCare
}