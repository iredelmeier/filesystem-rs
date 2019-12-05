use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{Read, Result};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::vec::IntoIter;
use std::cmp::min;

use FileSystem;
#[cfg(unix)]
use UnixFileSystem;
#[cfg(feature = "temp")]
use {TempDir, TempFileSystem};

#[cfg(feature = "temp")]
pub use self::tempdir::FakeTempDir;

use self::registry::Registry;

mod node;
mod registry;
#[cfg(feature = "temp")]
mod tempdir;

/// An in-memory file system.
#[derive(Clone, Debug, Default)]
pub struct FakeFileSystem {
    registry: Arc<Mutex<Registry>>,
}

fn apply<F, T>(registry: &Arc<Mutex<Registry>>, path: &Path, f: F) -> T
where
    F: FnOnce(&MutexGuard<Registry>, &Path) -> T,
{
    let registry = registry.lock().unwrap();
    let storage;
    let path = if path.is_relative() {
        storage = registry
            .current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path);
        &storage
    } else {
        path
    };

    f(&registry, path)
}

impl FakeFileSystem {
    pub fn new() -> Self {
        let registry = Registry::new();

        FakeFileSystem {
            registry: Arc::new(Mutex::new(registry)),
        }
    }

    fn apply_mut<F, T>(&self, path: &Path, mut f: F) -> T
    where
        F: FnMut(&mut MutexGuard<Registry>, &Path) -> T,
    {
        let mut registry = self.registry.lock().unwrap();
        let storage;
        let path = if path.is_relative() {
            storage = registry
                .current_dir()
                .unwrap_or_else(|_| PathBuf::from("/"))
                .join(path);
            &storage
        } else {
            path
        };

        f(&mut registry, path)
    }

    fn apply_mut_from_to<F, T>(&self, from: &Path, to: &Path, mut f: F) -> T
    where
        F: FnMut(&mut MutexGuard<Registry>, &Path, &Path) -> T,
    {
        let mut registry = self.registry.lock().unwrap();
        let from_storage;
        let from = if from.is_relative() {
            from_storage = registry
                .current_dir()
                .unwrap_or_else(|_| PathBuf::from("/"))
                .join(from);
            &from_storage
        } else {
            from
        };
        let to_storage;
        let to = if to.is_relative() {
            to_storage = registry
                .current_dir()
                .unwrap_or_else(|_| PathBuf::from("/"))
                .join(to);
            &to_storage
        } else {
            to
        };

        f(&mut registry, from, to)
    }
}

impl FileSystem for FakeFileSystem {
    type DirEntry = DirEntry;
    type ReadDir = ReadDir;
    type OpenFile = FakeOpenFile;

    fn current_dir(&self) -> Result<PathBuf> {
        let registry = self.registry.lock().unwrap();
        registry.current_dir()
    }

    fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.set_current_dir(p.to_path_buf()))
    }

    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        apply(&self.registry, path.as_ref(), |r, p| r.is_dir(p))
    }

    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        apply(&self.registry, path.as_ref(), |r, p| r.is_file(p))
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.create_dir(p))
    }

    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.create_dir_all(p))
    }

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.remove_dir(p))
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.remove_dir_all(p))
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadDir> {
        let path = path.as_ref();

        apply(&self.registry, path, |r, p| r.read_dir(p)).map(|entries| {
            let entries = entries
                .iter()
                .map(|e| {
                    let file_name = e.file_name().unwrap_or_else(|| e.as_os_str());

                    Ok(DirEntry::new(path, &file_name))
                })
                .collect();

            ReadDir::new(entries)
        })
    }

    fn create_file<P, B>(&self, path: P, buf: B) -> Result<()>
    where
        P: AsRef<Path>,
        B: AsRef<[u8]>,
    {
        self.apply_mut(path.as_ref(), |r, p| r.create_file(p, buf.as_ref()))
    }

    fn write_file<P, B>(&self, path: P, buf: B) -> Result<()>
    where
        P: AsRef<Path>,
        B: AsRef<[u8]>,
    {
        self.apply_mut(path.as_ref(), |r, p| r.write_file(p, buf.as_ref()))
    }

    fn overwrite_file<P, B>(&self, path: P, buf: B) -> Result<()>
    where
        P: AsRef<Path>,
        B: AsRef<[u8]>,
    {
        self.apply_mut(path.as_ref(), |r, p| r.overwrite_file(p, buf.as_ref()))
    }

    fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        apply(&self.registry, path.as_ref(), |r, p| r.read_file(p))
    }

    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Self::OpenFile> {
        FakeOpenFile::try_new(
            &self.registry,
            path.as_ref()
        )
    }

    fn read_file_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        apply(&self.registry, path.as_ref(), |r, p| r.read_file_to_string(p))
    }

    fn read_file_into<P, B>(&self, path: P, mut buf: B) -> Result<usize>
    where
        P: AsRef<Path>,
        B: AsMut<Vec<u8>>,
    {
        apply(&self.registry, path.as_ref(), |r, p| r.read_file_into(p, buf.as_mut()))
    }

    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.remove_file(p))
    }

    fn copy_file<P, Q>(&self, from: P, to: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        self.apply_mut_from_to(from.as_ref(), to.as_ref(), |r, from, to| {
            r.copy_file(from, to)
        })
    }

    fn rename<P, Q>(&self, from: P, to: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        self.apply_mut_from_to(from.as_ref(), to.as_ref(), |r, from, to| r.rename(from, to))
    }

    fn readonly<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        apply(&self.registry, path.as_ref(), |r, p| r.readonly(p))
    }

    fn set_readonly<P: AsRef<Path>>(&self, path: P, readonly: bool) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.set_readonly(p, readonly))
    }

    fn len<P: AsRef<Path>>(&self, path: P) -> u64 {
        apply(&self.registry, path.as_ref(), |r, p| r.len(p))
    }
}

#[derive(Debug)]
pub struct FakeOpenFile {
    registry: Arc<Mutex<Registry>>,
    path: PathBuf,
    offset: usize,
}

impl FakeOpenFile {
    fn try_new(registry: &Arc<Mutex<Registry>>, path: &Path) -> Result<Self> {
        apply(registry, path, |r, p| {
            r.access(p)
        })
        .map(|()| FakeOpenFile {
            registry: registry.clone(),
            path: path.to_owned(),
            offset: 0,
        })
    }
}

impl Read for FakeOpenFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        apply(&self.registry, self.path.as_ref(), |r, p| {
            let contents = r.read_file_ref(p)?;
            let ofs = self.offset;
            // If the underlying file has shrunk, the offset could
            // point to beyond eof.
            let len = if ofs < contents.len() {
                min(contents.len() - ofs, buf.len())
            } else {
                0
            };
            if len > 0 {
                buf[..len].copy_from_slice(&contents[ofs..ofs+len]);
            }
            Ok(len)
        })
        .map(|len| {
            self.offset += len;
            len
        })
    }
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    parent: PathBuf,
    file_name: OsString,
}

impl DirEntry {
    fn new<P, S>(parent: P, file_name: S) -> Self
    where
        P: AsRef<Path>,
        S: AsRef<OsStr>,
    {
        DirEntry {
            parent: parent.as_ref().to_path_buf(),
            file_name: file_name.as_ref().to_os_string(),
        }
    }
}

impl crate::DirEntry for DirEntry {
    fn file_name(&self) -> OsString {
        self.file_name.clone()
    }

    fn path(&self) -> PathBuf {
        self.parent.join(&self.file_name)
    }
}

#[derive(Debug)]
pub struct ReadDir(IntoIter<Result<DirEntry>>);

impl ReadDir {
    fn new(entries: Vec<Result<DirEntry>>) -> Self {
        ReadDir(entries.into_iter())
    }
}

impl Iterator for ReadDir {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl crate::ReadDir<DirEntry> for ReadDir {}

#[cfg(unix)]
impl UnixFileSystem for FakeFileSystem {
    fn mode<P: AsRef<Path>>(&self, path: P) -> Result<u32> {
        apply(&self.registry, path.as_ref(), |r, p| r.mode(p))
    }

    fn set_mode<P: AsRef<Path>>(&self, path: P, mode: u32) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.set_mode(p, mode))
    }
}

#[cfg(feature = "temp")]
impl TempFileSystem for FakeFileSystem {
    type TempDir = FakeTempDir;

    fn temp_dir<S: AsRef<str>>(&self, prefix: S) -> Result<Self::TempDir> {
        let base = env::temp_dir();
        let dir = FakeTempDir::new(Arc::downgrade(&self.registry), &base, prefix.as_ref());

        self.create_dir_all(&dir.path()).and(Ok(dir))
    }
}
