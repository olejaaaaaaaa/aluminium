use std::{path::PathBuf, str::FromStr};


#[allow(missing_docs)]
#[derive(Debug)]
pub enum Source {
    None,
    Path(PathBuf),
    SpirvU32(Vec<u32>),
    SpirvU8(Vec<u8>)
}

impl Into<Source> for &'static str {
    fn into(self) -> Source {
        Source::Path(PathBuf::from(self))
    }
}

impl Into<Source> for String {
    fn into(self) -> Source {
        Source::Path(PathBuf::from(self))
    }
}

impl Into<Source> for &[u32] {
    fn into(self) -> Source {
        Source::SpirvU32(self.to_vec())
    }
}

impl Into<Source> for &[u8] {
    fn into(self) -> Source {
        Source::SpirvU8(self.to_vec())
    }
}