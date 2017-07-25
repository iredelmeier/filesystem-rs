use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct File {
    pub contents: Vec<u8>,
    pub mode: u32,
}

impl File {
    pub fn new(contents: Vec<u8>) -> Self {
        File {
            contents: contents,
            mode: 0o644,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Dir {
    pub children: BTreeSet<PathBuf>,
    pub mode: u32,
}

impl Dir {
    pub fn new() -> Self {
        Dir {
            children: BTreeSet::new(),
            mode: 0o644,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FakeFile {
    File(File),
    Dir(Dir),
}

impl FakeFile {
    pub fn is_file(&self) -> bool {
        match *self {
            FakeFile::File(_) => true,
            _ => false,
        }
    }

    pub fn is_dir(&self) -> bool {
        match *self {
            FakeFile::Dir(_) => true,
            _ => false,
        }
    }
}
