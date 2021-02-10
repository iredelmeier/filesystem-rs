use crate::fake::registry::Registry;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct File {
    pub contents: Vec<u8>,
    pub mode: u32,
}

impl File {
    pub fn new(contents: Vec<u8>) -> Self {
        File {
            contents,
            mode: 0o644,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Dir {
    pub mode: u32,
}

impl Dir {
    pub fn new() -> Self {
        Dir { mode: 0o644 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Symlink {
    pub mode: u32,
    pub source: PathBuf,
}

impl Symlink {
    pub fn new(source: PathBuf) -> Self {
        Symlink {
            mode: 0o644,
            source,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    File(File),
    Dir(Dir),
    Symlink(Symlink),
}

impl Node {
    pub fn is_file(&self, registry: &Registry) -> bool {
        match &*self {
            Self::File(_) => true,
            Self::Symlink(symlink) => registry.is_file(&symlink.source),
            _ => false,
        }
    }

    pub fn is_dir(&self, registry: &Registry) -> bool {
        match &*self {
            Self::Dir(_) => true,
            Self::Symlink(symlink) => registry.is_dir(&symlink.source),
            _ => false,
        }
    }
}
