use std::io::{Cursor, ErrorKind, Read, Result, Seek, SeekFrom, Write};

#[derive(Debug, Clone)]
pub struct File {
    contents: Cursor<Vec<u8>>,
    mode: u32,
}

impl File {
    pub fn new() -> Self {
        File {
            contents: Cursor::new(vec![]),
            mode: 0o644,
        }
    }

    pub fn truncate(&mut self) {
        self.contents.get_mut().truncate(0)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.contents.read(buf)
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.contents.seek(pos)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.contents.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.contents.flush()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Dir {
    mode: u32,
}

impl Dir {
    pub fn new() -> Self {
        Dir { mode: 0o644 }
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    File(File),
    Dir(Dir),
}

impl Node {
    pub fn mode(&self) -> u32 {
        match *self {
            Self::File(ref file) => file.mode,
            Self::Dir(ref dir) => dir.mode,
        }
    }

    pub fn set_mode(&mut self, mode: u32) {
        match *self {
            Self::File(ref mut file) => file.mode = mode,
            Self::Dir(ref mut dir) => dir.mode = mode,
        }
    }

    pub fn len(&self) -> u64 {
        match *self {
            Self::File(ref file) => file.contents.get_ref().len() as u64,
            Self::Dir(_) => 4096,
        }
    }

    pub fn is_file(&self) -> bool {
        match *self {
            Self::File(_) => true,
            _ => false,
        }
    }

    pub fn is_dir(&self) -> bool {
        match *self {
            Self::Dir(_) => true,
            _ => false,
        }
    }

    pub fn as_readable_file(&mut self) -> Result<&mut File> {
        match *self {
            Self::File(ref mut file) if super::is_readable(file.mode) => Ok(file),
            Self::File(_) => Err(super::create_error(ErrorKind::PermissionDenied)),
            Self::Dir(_) => Err(super::create_error(ErrorKind::Other)),
        }
    }

    pub fn as_writable_file(&mut self) -> Result<&mut File> {
        match *self {
            Self::File(ref mut file) if super::is_writable(file.mode) => Ok(file),
            Self::File(_) => Err(super::create_error(ErrorKind::PermissionDenied)),
            Self::Dir(_) => Err(super::create_error(ErrorKind::Other)),
        }
    }

    pub fn as_dir(&self) -> Result<&Dir> {
        match *self {
            Self::Dir(ref dir) => Ok(dir),
            Self::File(_) => Err(super::create_error(ErrorKind::Other)),
        }
    }

    pub fn as_writable_dir(&self) -> Result<&Dir> {
        match *self {
            Self::Dir(ref dir) if super::is_writable(dir.mode) => Ok(dir),
            Self::Dir(_) => Err(super::create_error(ErrorKind::PermissionDenied)),
            Self::File(_) => Err(super::create_error(ErrorKind::Other)),
        }
    }
}
