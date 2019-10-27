use std::collections::HashMap;
use std::io::{ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use super::node::{Dir, File, Node};

#[derive(Debug, Clone, Default)]
pub struct Registry {
    cwd: PathBuf,
    files: HashMap<PathBuf, Node>,
}

impl Registry {
    pub fn new() -> Self {
        let cwd = PathBuf::from("/");
        let mut files = HashMap::new();

        files.insert(cwd.clone(), Node::Dir(Dir::new()));

        Registry { cwd, files }
    }

    pub fn current_dir(&self) -> Result<PathBuf> {
        self.get(&self.cwd)
            .and_then(Node::as_dir)
            .and(Ok(self.cwd.clone()))
    }

    pub fn set_current_dir(&mut self, cwd: PathBuf) -> Result<()> {
        self.get(&cwd).and_then(Node::as_dir)?;

        self.cwd = cwd;

        Ok(())
    }

    pub fn is_dir(&self, path: &Path) -> bool {
        self.get(path).map(Node::is_dir).unwrap_or(false)
    }

    pub fn is_file(&self, path: &Path) -> bool {
        self.get(path).map(Node::is_file).unwrap_or(false)
    }

    pub fn create_dir(&mut self, path: &Path) -> Result<()> {
        self.insert(path.to_path_buf(), Node::Dir(Dir::new()))
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
            None => return Err(super::create_error(ErrorKind::Other)),
        }

        self.create_dir_all(path)
    }

    pub fn remove_dir(&mut self, path: &Path) -> Result<()> {
        self.get(path).and_then(Node::as_dir).and_then(|_| {
            if self.descendants(path).is_empty() {
                Ok(())
            } else {
                Err(super::create_error(ErrorKind::Other))
            }
        })?;

        self.remove(path).and(Ok(()))
    }

    pub fn remove_dir_all(&mut self, path: &Path) -> Result<()> {
        self.get(path).and_then(Node::as_dir)?;

        let descendants = self.descendants(path);
        let all_readable = descendants
            .iter()
            .all(|(_, mode)| super::is_readable(*mode));

        if !all_readable {
            return Err(super::create_error(ErrorKind::PermissionDenied));
        }

        for (child, _) in descendants {
            self.remove(&child)?;
        }

        self.remove(path).and(Ok(()))
    }

    pub fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        self.get(path).and_then(Node::as_dir)?;

        Ok(self.children(path))
    }

    pub fn create_file(&mut self, path: &Path, buf: &[u8]) -> Result<()> {
        let mut file = File::new();

        file.write_all(buf)?;
        file.seek(SeekFrom::Start(0))?;

        self.insert(path.to_path_buf(), Node::File(file))
    }

    pub fn write_file(&mut self, path: &Path, buf: &[u8]) -> Result<()> {
        self.overwrite_file(path, buf).or_else(|e| {
            if e.kind() == ErrorKind::NotFound {
                self.create_file(path, buf)
            } else {
                Err(e)
            }
        })
    }

    pub fn overwrite_file(&mut self, path: &Path, buf: &[u8]) -> Result<()> {
        self.get_mut(path)
            .and_then(Node::as_writable_file)
            .and_then(|ref mut file| {
                file.truncate();
                file.write_all(buf)?;

                Ok(())
            })
            .and(Ok(()))
    }

    pub fn read_file(&mut self, path: &Path) -> Result<Vec<u8>> {
        let mut buf = Vec::<u8>::new();

        self.read_file_into(path, &mut buf)?;

        Ok(buf)
    }

    pub fn read_file_to_string(&mut self, path: &Path) -> Result<String> {
        let mut buf = String::new();

        self.get_mut(path)
            .and_then(Node::as_readable_file)
            .and_then(|file| file.read_to_string(&mut buf))?;

        Ok(buf)
    }

    pub fn read_file_into(&mut self, path: &Path, buf: &mut Vec<u8>) -> Result<usize> {
        self.get_mut(path)
            .and_then(Node::as_readable_file)
            .and_then(|file| {
                file.seek(SeekFrom::Start(0))?;
                file.read_to_end(buf)
            })
    }

    pub fn remove_file(&mut self, path: &Path) -> Result<()> {
        let node = self.get(path)?;

        if node.is_file() {
            self.remove(path).and(Ok(()))
        } else {
            Err(super::create_error(ErrorKind::Other))
        }
    }

    pub fn copy_file(&mut self, from: &Path, to: &Path) -> Result<()> {
        match self.read_file(from) {
            Ok(ref buf) => self.write_file(to, buf),
            Err(ref err) if err.kind() == ErrorKind::Other => {
                Err(super::create_error(ErrorKind::InvalidInput))
            }
            Err(err) => Err(err),
        }
    }

    pub fn rename(&mut self, from: &Path, to: &Path) -> Result<()> {
        match (self.get(from), self.get(to)) {
            (Ok(&Node::File(_)), Ok(&Node::File(_))) => {
                self.remove_file(to)?;
                self.rename_path(from, to.to_path_buf())
            }
            (Ok(&Node::File(_)), Err(ref err)) if err.kind() == ErrorKind::NotFound => {
                self.rename_path(from, to.to_path_buf())
            }
            (Ok(&Node::Dir(_)), Ok(&Node::Dir(_))) if self.descendants(to).is_empty() => {
                self.remove(to)?;
                self.move_dir(from, to)
            }
            (Ok(&Node::File(_)), Ok(&Node::Dir(_)))
            | (Ok(&Node::Dir(_)), Ok(&Node::File(_)))
            | (Ok(&Node::Dir(_)), Ok(&Node::Dir(_))) => Err(super::create_error(ErrorKind::Other)),
            (Ok(&Node::Dir(_)), Err(ref err)) if err.kind() == ErrorKind::NotFound => {
                self.move_dir(from, to)
            }
            (Err(err), _) => Err(err),
            (_, Err(err)) => Err(err),
        }
    }

    pub fn readonly(&self, path: &Path) -> Result<bool> {
        let node = self.get(path)?;
        let readonly = node.mode() & 0o222 == 0;

        Ok(readonly)
    }

    pub fn set_readonly(&mut self, path: &Path, readonly: bool) -> Result<()> {
        let node = self.get_mut(path)?;
        let mode = node.mode();

        if readonly {
            node.set_mode(mode & !0o222);
        } else {
            node.set_mode(mode | 0o222);
        }

        Ok(())
    }

    pub fn mode(&self, path: &Path) -> Result<u32> {
        self.get(path).map(Node::mode)
    }

    pub fn set_mode(&mut self, path: &Path, mode: u32) -> Result<()> {
        self.get_mut(path).map(|node| node.set_mode(mode))
    }

    pub fn len(&self, path: &Path) -> u64 {
        self.get(path).map(Node::len).unwrap_or(0)
    }

    fn get(&self, path: &Path) -> Result<&Node> {
        self.files
            .get(path)
            .ok_or_else(|| super::create_error(ErrorKind::NotFound))
    }

    fn get_mut(&mut self, path: &Path) -> Result<&mut Node> {
        self.files
            .get_mut(path)
            .ok_or_else(|| super::create_error(ErrorKind::NotFound))
    }

    fn insert(&mut self, path: PathBuf, node: Node) -> Result<()> {
        if self.files.contains_key(&path) {
            return Err(super::create_error(ErrorKind::AlreadyExists));
        } else if let Some(p) = path.parent() {
            self.get(p).and_then(Node::as_writable_dir)?;
        }

        self.files.insert(path, node);

        Ok(())
    }

    fn remove(&mut self, path: &Path) -> Result<Node> {
        match self.files.remove(path) {
            Some(f) => Ok(f),
            None => Err(super::create_error(ErrorKind::NotFound)),
        }
    }

    fn descendants(&self, path: &Path) -> Vec<(PathBuf, u32)> {
        self.files
            .iter()
            .filter(|(p, _)| p.starts_with(path) && *p != path)
            .map(|(p, n)| (p.to_path_buf(), n.mode()))
            .collect()
    }

    fn children(&self, path: &Path) -> Vec<PathBuf> {
        self.files
            .keys()
            .filter(|p| p.parent().map(|parent| parent == path).unwrap_or(false))
            .map(|p| p.to_path_buf())
            .collect()
    }

    fn rename_path(&mut self, from: &Path, to: PathBuf) -> Result<()> {
        let file = self.remove(from)?;
        self.insert(to, file)
    }

    fn move_dir(&mut self, from: &Path, to: &Path) -> Result<()> {
        self.rename_path(from, to.to_path_buf())?;

        for child in self.children(from) {
            let stem = child.strip_prefix(from).unwrap_or(&child);
            let new_path = to.join(stem);

            self.rename(&child, &new_path)?;
        }

        Ok(())
    }
}
