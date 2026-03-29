pub enum Viewport {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
}

pub enum Scissor {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
}
