use std::env;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use FileSystem;
#[cfg(feature = "temp")]
use {TempDir, TempFileSystem};

#[cfg(feature = "temp")]
pub use self::tempdir::FakeTempDir;

use self::file::{Dir, FakeFile, File};
use self::registry::Registry;

mod file;
mod registry;
#[cfg(feature = "temp")]
mod tempdir;

#[derive(Clone, Debug, Default)]
pub struct FakeFileSystem {
    registry: Arc<Mutex<Registry>>,
}

impl FakeFileSystem {
    pub fn new() -> Self {
        let registry = Registry::new();

        FakeFileSystem { registry: Arc::new(Mutex::new(registry)) }
    }
}

impl FileSystem for FakeFileSystem {
    fn current_dir(&self) -> Result<PathBuf> {
        self.registry.lock().unwrap().current_dir()
    }

    fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.set_current_dir(path)
    }

    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        let registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.is_dir(&path)
    }

    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        let registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.is_file(&path)
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.create_dir(&path)
    }

    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.create_dir_all(&path)
    }

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.remove_dir(&path)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.remove_dir_all(&path)
    }

    fn create_file<P, B>(&self, path: P, buf: B) -> Result<()>
        where P: AsRef<Path>,
              B: AsRef<[u8]>
    {
        let mut registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.create_file(&path, buf.as_ref())
    }

    fn write_file<P, B>(&self, path: P, buf: B) -> Result<()>
        where P: AsRef<Path>,
              B: AsRef<[u8]>
    {
        let mut registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.write_file(&path, buf.as_ref())
    }

    fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.read_file(&path)
    }

    fn readonly<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.readonly(&path)
    }

    fn set_readonly<P: AsRef<Path>>(&self, path: P, readonly: bool) -> Result<()> {
        let mut registry = self.registry.lock().unwrap();
        let path = expand_path(path, registry.current_dir());
        registry.set_readonly(&path, readonly)
    }
}

#[cfg(feature = "temp")]
impl TempFileSystem for FakeFileSystem {
    type TempDir = FakeTempDir;

    fn temp_dir<S: AsRef<str>>(&self, prefix: S) -> Result<Self::TempDir> {
        let base = env::temp_dir();
        let dir = FakeTempDir::new(Arc::downgrade(&self.registry), &base, prefix.as_ref());

        self.create_dir_all(&dir.path())
            .and(Ok(dir))
    }
}

fn expand_path<P: AsRef<Path>>(path: P, cwd: Result<PathBuf>) -> PathBuf {
    let path = path.as_ref();

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    }
}
