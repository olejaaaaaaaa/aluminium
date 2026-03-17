use std::path::PathBuf;

#[allow(missing_docs)]
#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Source {
    None,
    Path(PathBuf),
    SpirvU32(Vec<u32>),
    SpirvU8(Vec<u8>),
}

impl From<&'static str> for Source {
    fn from(val: &'static str) -> Self {
        Source::Path(PathBuf::from(val))
    }
}

impl From<String> for Source {
    fn from(val: String) -> Self {
        Source::Path(PathBuf::from(val))
    }
}

impl From<&[u32]> for Source {
    fn from(val: &[u32]) -> Self {
        Source::SpirvU32(val.to_vec())
    }
}

impl From<&[u8]> for Source {
    fn from(val: &[u8]) -> Self {
        Source::SpirvU8(val.to_vec())
    }
}
