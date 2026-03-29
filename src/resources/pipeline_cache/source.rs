use std::path::PathBuf;

#[allow(missing_docs)]
#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Source<'a> {
    Path(PathBuf),
    SpirvU32(&'a [u32]),
    SpirvU8(&'a [u8]),
}

impl From<&'static str> for Source<'static> {
    fn from(val: &'static str) -> Self {
        Source::Path(PathBuf::from(val))
    }
}

impl From<String> for Source<'static> {
    fn from(val: String) -> Self {
        Source::Path(PathBuf::from(val))
    }
}

impl<'a> From<&'a [u32]> for Source<'a> {
    fn from(val: &'a [u32]) -> Self {
        Source::SpirvU32(val)
    }
}

impl<'a> From<&'a [u8]> for Source<'a> {
    fn from(val: &'a [u8]) -> Self {
        Source::SpirvU8(val)
    }
}
