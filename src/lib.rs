#[cfg(any(feature = "mock", test))]
extern crate pseudo;
#[cfg(feature = "temp")]
extern crate rand;
#[cfg(feature = "temp")]
extern crate tempdir;

use std::io::Result;
use std::path::{Path, PathBuf};

#[cfg(any(feature = "mock", test))]
pub use mock::{FakeError, MockFileSystem};
#[cfg(feature = "fake")]
pub use fake::{FakeFileSystem, FakeTempDir};
pub use os::OsFileSystem;
#[cfg(feature = "temp")]
pub use os::OsTempDir;

#[cfg(feature = "fake")]
mod fake;
#[cfg(any(feature = "mock", test))]
mod mock;
mod os;

pub trait FileSystem {
    fn current_dir(&self) -> Result<PathBuf>;
    fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool;
    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool;

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    fn create_file<P, B>(&self, path: P, buf: B) -> Result<()>
        where P: AsRef<Path>,
              B: AsRef<[u8]>;
    fn write_file<P, B>(&self, path: P, buf: B) -> Result<()>
        where P: AsRef<Path>,
              B: AsRef<[u8]>;
    fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>>;

    fn readonly<P: AsRef<Path>>(&self, path: P) -> Result<bool>;
    fn set_readonly<P: AsRef<Path>>(&self, path: P, readonly: bool) -> Result<()>;
}

#[cfg(feature = "temp")]
pub trait TempDir {
    fn path(&self) -> &Path;
}

#[cfg(feature = "temp")]
pub trait TempFileSystem {
    type TempDir: TempDir;

    fn temp_dir<S: AsRef<str>>(&self, prefix: S) -> Result<Self::TempDir>;
}
