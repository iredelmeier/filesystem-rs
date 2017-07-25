use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

use super::{Dir, FakeFile, File};

#[derive(Debug, Clone, Default)]
pub struct Registry {
    cwd: PathBuf,
    files: HashMap<PathBuf, FakeFile>,
}

impl Registry {
    pub fn new() -> Self {
        let cwd = PathBuf::from("/");
        let mut files = HashMap::new();

        files.insert(cwd.clone(), FakeFile::Dir(Dir::new()));

        Registry {
            cwd: cwd,
            files: files,
        }
    }

    pub fn current_dir(&self) -> Result<PathBuf> {
        self.get_dir(&self.cwd)
            .map(|_| self.cwd.clone())
    }

    pub fn set_current_dir(&mut self, cwd: PathBuf) -> Result<()> {
        match self.get_dir(&cwd) {
            Ok(_) => {
                self.cwd = cwd;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn is_dir(&self, path: &Path) -> bool {
        self.files
            .get(path)
            .map(FakeFile::is_dir)
            .unwrap_or(false)
    }

    pub fn is_file(&self, path: &Path) -> bool {
        self.files
            .get(path)
            .map(FakeFile::is_file)
            .unwrap_or(false)
    }

    pub fn create_dir(&mut self, path: &Path) -> Result<()> {
        self.insert(path.to_path_buf(), FakeFile::Dir(Dir::new()))
    }

    pub fn create_dir_all(&mut self, path: &Path) -> Result<()> {
        // Based on std::fs::DirBuilder::create_dir_all
        if path == Path::new("") {
            return Ok(());
        }

        match self.create_dir(path) {
            Ok(_) => return Ok(()),
            Err(ref e) if e.kind() == ErrorKind::NotFound => {}
            Err(_) if self.is_dir(path) => return Ok(()),
            Err(e) => return Err(e),
        }

        match path.parent() {
            Some(p) => self.create_dir_all(p)?,
            None => return Err(create_error(ErrorKind::Other)),
        }

        self.create_dir_all(path)
    }

    pub fn remove_dir(&mut self, path: &Path) -> Result<()> {
        match self.get_dir(path) {
            Ok(dir) if dir.children.is_empty() => {}
            Ok(_) => return Err(create_error(ErrorKind::Other)),
            Err(e) => return Err(e),
        };

        self.files.remove(path);

        Ok(())
    }

    pub fn remove_dir_all(&mut self, path: &Path) -> Result<()> {
        let dir = self.get_dir(path)?.clone();

        for child in dir.children {
            if self.is_dir(&child) {
                self.remove_dir_all(&child)?;
            } else {
                self.files.remove(&child);
            }
        }

        self.files.remove(path);

        Ok(())
    }

    pub fn create_file(&mut self, path: &Path, buf: &[u8]) -> Result<()> {
        let file = File::new(buf.to_vec());

        self.insert(path.to_path_buf(), FakeFile::File(file))
    }

    pub fn write_file(&mut self, path: &Path, buf: &[u8]) -> Result<()> {
        self.get_file_mut(path)
            .map(|ref mut f| f.contents = buf.to_vec())
            .or_else(|e| if e.kind() == ErrorKind::NotFound {
                self.create_file(path, buf)
            } else {
                Err(e)
            })
    }

    pub fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        self.get_file(path)
            .map(|f| f.contents.clone())
    }

    pub fn readonly(&self, path: &Path) -> Result<bool> {
        match self.files.get(path) {
            Some(&FakeFile::File(ref f)) => Ok(f.readonly),
            Some(&FakeFile::Dir(ref d)) => Ok(d.readonly),
            None => Err(create_error(ErrorKind::NotFound)),
        }
    }

    pub fn set_readonly(&mut self, path: &Path, readonly: bool) -> Result<()> {
        match self.files.get_mut(path) {
            Some(&mut FakeFile::File(ref mut f)) => {
                f.readonly = readonly;
                Ok(())
            }
            Some(&mut FakeFile::Dir(ref mut d)) => {
                d.readonly = readonly;
                Ok(())
            }
            None => Err(create_error(ErrorKind::NotFound)),
        }
    }

    fn get_dir(&self, path: &Path) -> Result<&Dir> {
        match self.files.get(path) {
            Some(&FakeFile::Dir(ref dir)) => Ok(dir),
            Some(_) => Err(create_error(ErrorKind::Other)),
            None => Err(create_error(ErrorKind::NotFound)),
        }
    }

    fn get_dir_mut(&mut self, path: &Path) -> Result<&mut Dir> {
        match self.files.get_mut(path) {
            Some(&mut FakeFile::Dir(ref mut dir)) => {
                if dir.readonly {
                    Err(create_error(ErrorKind::PermissionDenied))
                } else {
                    Ok(dir)
                }
            }
            Some(_) => Err(create_error(ErrorKind::Other)),
            None => Err(create_error(ErrorKind::NotFound)),
        }
    }

    fn get_file(&self, path: &Path) -> Result<&File> {
        match self.files.get(path) {
            Some(&FakeFile::File(ref file)) => Ok(file),
            Some(_) => Err(create_error(ErrorKind::Other)),
            None => Err(create_error(ErrorKind::NotFound)),
        }
    }

    fn get_file_mut(&mut self, path: &Path) -> Result<&mut File> {
        match self.files.get_mut(path) {
            Some(&mut FakeFile::File(ref mut file)) => {
                if file.readonly {
                    Err(create_error(ErrorKind::PermissionDenied))
                } else {
                    Ok(file)
                }
            }
            Some(_) => Err(create_error(ErrorKind::Other)),
            None => Err(create_error(ErrorKind::NotFound)),
        }
    }

    fn insert(&mut self, path: PathBuf, file: FakeFile) -> Result<()> {
        if self.files.contains_key(&path) {
            return Err(create_error(ErrorKind::AlreadyExists));
        } else if let Some(p) = path.parent() {
            let mut parent = self.get_dir_mut(p)?;
            parent.children.insert(path.clone());
        }
        self.files.insert(path, file);
        Ok(())
    }
}

fn create_error(kind: ErrorKind) -> Error {
    // Based on private std::io::ErrorKind::as_str()
    let description = match kind {
        ErrorKind::NotFound => "entity not found",
        ErrorKind::PermissionDenied => "permission denied",
        ErrorKind::ConnectionRefused => "connection refused",
        ErrorKind::ConnectionReset => "connection reset",
        ErrorKind::ConnectionAborted => "connection aborted",
        ErrorKind::NotConnected => "not connected",
        ErrorKind::AddrInUse => "address in use",
        ErrorKind::AddrNotAvailable => "address not available",
        ErrorKind::BrokenPipe => "broken pipe",
        ErrorKind::AlreadyExists => "entity already exists",
        ErrorKind::WouldBlock => "operation would block",
        ErrorKind::InvalidInput => "invalid input parameter",
        ErrorKind::InvalidData => "invalid data",
        ErrorKind::TimedOut => "timed out",
        ErrorKind::WriteZero => "write zero",
        ErrorKind::Interrupted => "operation interrupted",
        ErrorKind::Other => "other os error",
        ErrorKind::UnexpectedEof => "unexpected end of file",
        _ => "other",
    };

    Error::new(kind, description)
}
