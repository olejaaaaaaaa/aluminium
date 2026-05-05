#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Resolution {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Viewport {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scissor {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
}
