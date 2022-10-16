// Copyright (c) 2017 Isobel Redelmeier
// Copyright (c) 2021 Miguel Barreto
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::collections::{HashMap, HashSet};
use std::io::{Error, ErrorKind, Result};
use std::path::{Component, Path, PathBuf};

use super::node::{Dir, File, Node, Symlink};

#[derive(Debug, Clone)]
pub struct Registry {
    cwd: PathBuf,
    files: HashMap<PathBuf, Node>,
}

impl Default for Registry {
    fn default() -> Self {
        Registry::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        let cwd = PathBuf::from("/");
        let mut files = HashMap::new();

        files.insert(cwd.clone(), Node::Dir(Dir::new()));

        Registry { cwd, files }
    }

    pub fn current_dir(&self) -> Result<PathBuf> {
        self.get_dir(&self.cwd).map(|_| self.cwd.clone())
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
        match self.resolve_path(path, true) {
            Ok(resolved_path) => self
                .get(&resolved_path)
                .map(|node| node.is_dir(&self))
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    pub fn is_file(&self, path: &Path) -> bool {
        match self.resolve_path(path, true) {
            Ok(resolved_path) => self
                .get(&resolved_path)
                .map(|node| node.is_file(&self))
                .unwrap_or(false),
            Err(_) => false,
        }
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
            None => return Err(create_error(ErrorKind::Other)),
        }

        self.create_dir_all(path)
    }

    pub fn remove_dir(&mut self, path: &Path) -> Result<()> {
        let path = &self.resolve_path(path, false)?;
        match self.get(path) {
            Ok(Node::Dir(_)) if self.descendants(path).is_empty() => {}
            Ok(Node::Dir(_)) => return Err(create_error(ErrorKind::DirectoryNotEmpty)),
            Ok(_) => return Err(create_error(ErrorKind::NotADirectory)),
            Err(e) => return Err(e),
        };

        self.remove(path).and(Ok(()))
    }

    pub fn remove_dir_all(&mut self, path: &Path) -> Result<()> {
        let path = &self.resolve_path(path, false)?;
        self.get_dir_mut(path)?;

        let descendants = self.descendants(path);
        let all_readable = descendants.iter().all(|(_, mode)| mode & 0o444 != 0);

        if !all_readable {
            return Err(create_error(ErrorKind::PermissionDenied));
        }

        for (child, _) in descendants {
            self.remove(&child)?;
        }

        self.remove(path).and(Ok(()))
    }

    pub fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let path = &self.resolve_path(path, true)?;
        self.get_dir(path)?;

        Ok(self.children(path))
    }

    pub fn create_file(&mut self, path: &Path, buf: &[u8]) -> Result<()> {
        let path = &self.resolve_path(path, true)?;
        let file = File::new(buf.to_vec());
        self.insert(path.to_path_buf(), Node::File(file))
    }

    pub fn write_file(&mut self, path: &Path, buf: &[u8]) -> Result<()> {
        let path = &self.resolve_path(path, true)?;
        self.get_file_mut(path)
            .map(|ref mut f| f.contents = buf.to_vec())
            .or_else(|e| {
                if e.kind() == ErrorKind::NotFound {
                    self.create_file(path, buf)
                } else {
                    Err(e)
                }
            })
    }

    pub fn overwrite_file(&mut self, path: &Path, buf: &[u8]) -> Result<()> {
        let path = &self.resolve_path(path, true)?;
        self.get_file_mut(path)
            .map(|ref mut f| f.contents = buf.to_vec())
    }

    pub fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        let path = &self.resolve_path(path, true)?;
        match self.get_file(path) {
            Ok(f) if f.mode & 0o444 != 0 => Ok(f.contents.clone()),
            Ok(_) => Err(create_error(ErrorKind::PermissionDenied)),
            Err(err) => Err(err),
        }
    }

    pub fn read_file_to_string(&self, path: &Path) -> Result<String> {
        let path = &self.resolve_path(path, true)?;
        match self.read_file(path) {
            Ok(vec) => String::from_utf8(vec).map_err(|_| create_error(ErrorKind::InvalidData)),
            Err(err) => Err(err),
        }
    }

    pub fn read_file_into(&self, path: &Path, buf: &mut Vec<u8>) -> Result<usize> {
        let path = &self.resolve_path(path, true)?;
        match self.get_file(path) {
            Ok(f) if f.mode & 0o444 != 0 => {
                buf.extend(&f.contents);
                Ok(f.contents.len())
            }
            Ok(_) => Err(create_error(ErrorKind::PermissionDenied)),
            Err(err) => Err(err),
        }
    }

    pub fn remove_file(&mut self, path: &Path) -> Result<()> {
        let path = &self.resolve_path(path, false)?;
        match self.get(path)? {
            Node::File(_) | Node::Symlink(_) => self.remove(path).and(Ok(())),
            Node::Dir(_) => Err(create_error(ErrorKind::Other)),
        }
    }

    pub fn copy_file(&mut self, from: &Path, to: &Path) -> Result<()> {
        let from = &self.resolve_path(from, true)?;
        let to = &self.resolve_path(to, true)?;
        match (self.read_file(from), self.get(to)) {
            (Ok(ref buf), Err(e)) if e.kind() == ErrorKind::NotFound => self.write_file(to, buf),
            (Ok(ref buf), Ok(Node::File(f))) if f.mode != 644 => self.write_file(to, buf),
            (Ok(ref buf), Ok(Node::Symlink(l))) if l.mode != 644 => self.write_file(to, buf),
            (Ok(_), Ok(Node::Symlink(_)) | Ok(Node::File(_))) => {
                Err(create_error(ErrorKind::PermissionDenied))
            }
            (Ok(_), _) => Err(create_error(ErrorKind::IsADirectory)),
            (Err(e), _) if e.kind() == ErrorKind::IsADirectory => {
                Err(create_error(ErrorKind::InvalidInput))
            }
            (Err(e), _) => Err(e),
        }
    }

    pub fn read_link<P: AsRef<Path>>(&'_ self, dst: P) -> Result<PathBuf> {
        let path = self.resolve_path(dst.as_ref(), false)?;
        match self.files.get(&path) {
            Some(Node::Symlink(link)) => Ok(link.source.to_path_buf()),
            Some(_) => Err(create_error(ErrorKind::InvalidInput)),
            None => Err(create_error(ErrorKind::NotFound)),
        }
    }

    fn resolve_path(&'_ self, path: &Path, follow_last_component: bool) -> Result<PathBuf> {
        match self.files.get(path) {
            Some(Node::File(_)) | Some(Node::Dir(_)) => return Ok(path.to_path_buf()),
            Some(Node::Symlink(_)) if follow_last_component => {
                return Ok(self.recurse_symlink(path).map(|(_, p)| p)?)
            }
            Some(Node::Symlink(_)) => return Ok(path.to_path_buf()),
            None => (),
        }
        let mut pathbuf = PathBuf::new();
        let count = path.components().count();
        for (i, component) in path.components().enumerate() {
            match component {
                Component::Prefix(prefix) => {
                    pathbuf.push(Component::Prefix(prefix));
                    continue;
                }
                Component::RootDir => {
                    pathbuf.push(Component::RootDir);
                    continue;
                }
                Component::Normal(comp) => pathbuf.push(Component::Normal(comp)),
                Component::CurDir | Component::ParentDir => {
                    return Err(create_error(ErrorKind::InvalidInput))
                }
            }

            match self.files.get(&pathbuf) {
                Some(Node::File(_)) | Some(Node::Dir(_)) => continue,
                Some(Node::Symlink(_)) => {
                    if !follow_last_component && i == count - 1 {
                        return Ok(pathbuf);
                    } else {
                        pathbuf = self.recurse_symlink(&pathbuf).map(|(_, p)| p)?;
                    }
                }
                None => {
                    if i == count - 1 {
                        return Ok(pathbuf);
                    } else {
                        return Err(create_error(ErrorKind::NotFound));
                    }
                }
            }
        }
        Ok(pathbuf)
    }
    fn recurse_symlink<'a>(&'a self, path: &Path) -> Result<(&'a Node, PathBuf)> {
        let mut traversed_items = HashSet::new();
        let mut path = path;
        let mut current = self.files.get(path);
        while let Some(&Node::Symlink(_)) = current {
            if traversed_items.contains(path) {
                return Err(create_error(ErrorKind::Other));
            }
            traversed_items.insert(path.to_path_buf());
            path = if let Node::Symlink(ref link) = current.unwrap() {
                &link.source
            } else {
                path
            };
            current = self.files.get(path);
        }
        match current {
            None => Err(create_error(ErrorKind::NotFound)),
            Some(node) => Ok((&node, path.to_path_buf())),
        }
    }

    pub fn rename(&mut self, from: &Path, to: &Path) -> Result<()> {
        let mut from = from.to_path_buf();
        match self.resolve_path(&from, false) {
            Ok(path) => from = path,
            Err(_) => return Err(create_error(ErrorKind::NotFound)),
        }
        let mut to = to.to_path_buf();
        match self.resolve_path(&to, false) {
            Ok(path) => to = path,
            Err(_) => return Err(create_error(ErrorKind::NotFound)),
        }
        match (self.get(&from), self.get(&to)) {
            (Ok(&Node::File(_)), Ok(&Node::File(_))) => {
                self.remove_file(&to)?;
                self.rename_path(&from, to)
            }
            (Ok(&Node::File(_)), Err(ref err)) if err.kind() == ErrorKind::NotFound => {
                self.rename_path(&from, to)
            }
            (Ok(&Node::Dir(_)), Err(ref err)) if err.kind() == ErrorKind::NotFound => {
                self.move_dir(&from, &to)
            }
            (Ok(&Node::Dir(_)), Ok(&Node::Dir(_))) if self.descendants(&to).is_empty() => {
                self.remove(&to)?;
                self.move_dir(&from, &to)
            }
            (Ok(&Node::File(_)), Ok(&Node::Symlink(_)))
                if self.recurse_symlink(&to)?.0.is_file(&self) =>
            {
                self.remove(&to)?;
                self.rename_path(&from, to)
            }
            (Ok(&Node::Dir(_)), Ok(&Node::Symlink(_))) => match self.recurse_symlink(&to)? {
                (Node::Dir(_), path) if self.descendants(&path).is_empty() => {
                    self.remove(&to)?;
                    self.move_dir(&from, &to)
                }
                _ => Err(create_error(ErrorKind::Other)),
            },
            (Ok(&Node::Symlink(_)), Err(ref err)) if err.kind() == ErrorKind::NotFound => {
                self.rename_path(&from, to)
            }
            (Ok(&Node::Symlink(_)), Ok(&Node::File(_)))
                if self.recurse_symlink(&from)?.0.is_file(&self) =>
            {
                self.remove(&to)?;
                self.rename_path(&from, to)
            }
            (Ok(&Node::Symlink(_)), Ok(&Node::Dir(_))) => match self.recurse_symlink(&from)? {
                (Node::Dir(_), _) if self.descendants(&to).is_empty() => {
                    self.remove(&to)?;
                    self.move_dir(&from, &to)
                }
                _ => Err(create_error(ErrorKind::Other)),
            },
            (Ok(&Node::Symlink(_)), Ok(&Node::Symlink(_))) => {
                match (self.recurse_symlink(&from), self.recurse_symlink(&to)) {
                    (Ok(_), Err(e)) if e.kind() == ErrorKind::NotFound => {
                        self.rename_path(&from, to.to_path_buf())
                    }
                    (Err(e), Ok((Node::File(_), _))) if e.kind() == ErrorKind::NotFound => {
                        self.remove_file(&to)?;
                        self.rename_path(&from, to)
                    }
                    (Err(e), _) => Err(e),
                    (Ok((Node::File(_), _)), Ok((Node::File(_), _))) => {
                        self.remove(&to)?;
                        self.rename_path(&from, to)
                    }
                    (Ok((Node::Dir(_), _)), Ok((Node::Dir(_), path))) => {
                        if self.descendants(&path).is_empty() {
                            self.remove(&to)?;
                            self.rename_path(&from, to)
                        } else {
                            Err(create_error(ErrorKind::Other))
                        }
                    }
                    (_, Err(_))
                    | (Ok((Node::File(_), _)), _)
                    | (Ok((Node::Dir(_), _)), _)
                    | (Ok((Node::Symlink(_), _)), _) => Err(create_error(ErrorKind::Other)),
                }
            }
            (Ok(&Node::File(_)), Ok(&Node::Dir(_)))
            | (Ok(&Node::File(_)), Ok(&Node::Symlink(_))) => {
                Err(create_error(ErrorKind::IsADirectory))
            }
            (Ok(&Node::Dir(_)), Ok(&Node::File(_)))
            | (Ok(&Node::Symlink(_)), Ok(&Node::File(_))) => {
                Err(create_error(ErrorKind::NotADirectory))
            }
            (Ok(&Node::Dir(_)), Ok(&Node::Dir(_))) => Err(create_error(ErrorKind::Other)),
            (Ok(&Node::Dir(_)), Err(ref err)) if err.kind() == ErrorKind::NotFound => {
                self.move_dir(&from, &to)
            }
            (Err(err), _) => Err(err),
            (_, Err(err)) => Err(err),
        }
    }

    pub fn readonly(&self, path: &Path) -> Result<bool> {
        self.get(path).map(|node| match node {
            Node::File(ref file) => file.mode & 0o222 == 0,
            Node::Dir(ref dir) => dir.mode & 0o222 == 0,
            Node::Symlink(ref symlink) => symlink.mode & 0o222 == 0,
        })
    }

    pub fn set_readonly(&mut self, path: &Path, readonly: bool) -> Result<()> {
        fn set_readonly_mode(mode: &mut u32, readonly: bool) {
            if readonly {
                *mode &= !0o222
            } else {
                *mode |= 0o222
            }
        }
        self.get_mut(path).map(|node| match node {
            Node::File(ref mut file) => {
                set_readonly_mode(&mut file.mode, readonly);
            }
            Node::Dir(ref mut dir) => {
                set_readonly_mode(&mut dir.mode, readonly);
            }
            Node::Symlink(ref mut link) => {
                set_readonly_mode(&mut link.mode, readonly);
            }
        })
    }

    pub fn mode(&self, path: &Path) -> Result<u32> {
        self.get(path).map(|node| match node {
            Node::File(ref file) => file.mode,
            Node::Dir(ref dir) => dir.mode,
            Node::Symlink(ref link) => link.mode,
        })
    }

    pub fn set_mode(&mut self, path: &Path, mode: u32) -> Result<()> {
        self.get_mut(path).map(|node| match node {
            Node::File(ref mut file) => file.mode = mode,
            Node::Dir(ref mut dir) => dir.mode = mode,
            Node::Symlink(ref mut link) => link.mode = mode,
        })
    }

    pub fn len(&self, path: &Path) -> u64 {
        self.get(path)
            .map(|node| match node {
                Node::File(ref file) => file.contents.len() as u64,
                Node::Dir(_) => 4096,
                Node::Symlink(_) => 34, // This is what it actually is on macOS
            })
            .unwrap_or(0)
    }

    fn get(&self, path: &Path) -> Result<&Node> {
        self.files
            .get(path)
            .ok_or_else(|| create_error(ErrorKind::NotFound))
    }

    fn get_mut(&mut self, path: &Path) -> Result<&mut Node> {
        self.files
            .get_mut(path)
            .ok_or_else(|| create_error(ErrorKind::NotFound))
    }

    fn get_dir(&self, path: &Path) -> Result<&Dir> {
        self.get(path).and_then(|node| match node {
            Node::Dir(ref dir) => Ok(dir),
            Node::File(_) => Err(create_error(ErrorKind::NotADirectory)),
            Node::Symlink(_) => match self.recurse_symlink(path) {
                Ok((Node::Dir(dir), _)) => Ok(&dir),
                Ok((Node::File(_), _)) | Ok((Node::Symlink(_), _)) => {
                    Err(create_error(ErrorKind::NotADirectory))
                }
                Err(e) => Err(e),
            },
        })
    }

    fn get_dir_mut(&mut self, path: &Path) -> Result<&mut Dir> {
        let mut path = path.to_path_buf();
        match self.get(&path)? {
            Node::Dir(dir) if dir.mode & 0o222 != 0 => (), // still get the original path
            Node::Dir(_) => return Err(create_error(ErrorKind::PermissionDenied)),
            Node::File(_) => return Err(create_error(ErrorKind::NotADirectory)),
            Node::Symlink(_) => match self.recurse_symlink(&path) {
                Ok((Node::Dir(_), new_path)) => path = new_path,
                Ok((Node::File(_), _)) | Ok((Node::Symlink(_), _)) => {
                    return Err(create_error(ErrorKind::NotADirectory))
                }
                Err(e) => return Err(e),
            },
        };
        if let Ok(Node::Dir(dir)) = self.get_mut(&path) {
            Ok(dir)
        } else {
            Err(create_error(ErrorKind::Other))
        }
    }

    fn get_file(&self, path: &Path) -> Result<&File> {
        self.get(path).and_then(|node| match node {
            Node::File(ref file) => Ok(file),
            Node::Dir(_) => Err(create_error(ErrorKind::IsADirectory)),
            Node::Symlink(_) => match self.recurse_symlink(path) {
                Ok((Node::File(file), _)) => Ok(&file),
                Ok((Node::Dir(_), _)) | Ok((Node::Symlink(_), _)) => {
                    Err(create_error(ErrorKind::IsADirectory))
                }
                Err(e) => Err(e),
            },
        })
    }

    fn get_file_mut(&mut self, path: &Path) -> Result<&mut File> {
        let mut path = path.to_path_buf();
        match self.get(&path)? {
            Node::File(file) if file.mode & 0o222 != 0 => (), // still get the original path
            Node::File(_) => return Err(create_error(ErrorKind::PermissionDenied)),
            Node::Dir(_) => return Err(create_error(ErrorKind::IsADirectory)),
            Node::Symlink(_) => match self.recurse_symlink(&path) {
                Ok((Node::File(_), new_path)) => path = new_path,
                Ok((Node::Dir(_), _)) | Ok((Node::Symlink(_), _)) => {
                    return Err(create_error(ErrorKind::IsADirectory))
                }
                Err(e) => return Err(e),
            },
        };
        match self.get_mut(&path) {
            Ok(Node::File(file)) => Ok(file),
            Ok(Node::Dir(_)) => Err(create_error(ErrorKind::IsADirectory)),
            Ok(Node::Symlink(_)) => Err(create_error(ErrorKind::Other)),
            Err(e) => Err(e),
        }
    }

    fn insert(&mut self, path: PathBuf, file: Node) -> Result<()> {
        let path = self.resolve_path(&path, false)?;
        if self.files.get(&path).is_some() {
            return Err(create_error(ErrorKind::AlreadyExists));
        }
        let parent: &Path = &path
            .parent()
            .ok_or_else(|| create_error(ErrorKind::NotADirectory))?;
        match self.files.get(parent) {
            Some(Node::Dir(_)) => self.get_dir_mut(parent)?,
            None | Some(_) => return Err(create_error(ErrorKind::NotADirectory)),
        };
        self.files.insert(path, file);

        Ok(())
    }

    fn remove(&mut self, path: &Path) -> Result<Node> {
        match self.files.remove(path) {
            Some(f) => Ok(f),
            None => Err(create_error(ErrorKind::NotFound)),
        }
    }

    fn descendants(&self, path: &Path) -> Vec<(PathBuf, u32)> {
        let mut pathbuf = path.to_path_buf();
        if let Ok(Node::Symlink(_)) = self.get(&path) {
            if let Ok((_, new_path)) = self.recurse_symlink(&path) {
                pathbuf = new_path;
            }
        }
        let path = &pathbuf;
        let mut descendants: Vec<(PathBuf, u32)> = self
            .files
            .iter()
            .filter(|(p, _)| p.starts_with(path) && *p != path)
            .map(|(p, n)| {
                (
                    p.to_path_buf(),
                    match n {
                        Node::File(ref file) => file.mode,
                        Node::Dir(ref dir) => dir.mode,
                        Node::Symlink(ref link) => link.mode,
                    },
                )
            })
            .collect();
        let mut found_symlink = true;
        let mut list = descendants.clone();
        while found_symlink {
            found_symlink = false;
            let mut new_list = Vec::new();
            for (p, _) in list {
                if let Some(Node::Symlink(_)) = self.files.get(&p) {
                    found_symlink = true;
                    new_list.extend(self.descendants(&p));
                }
            }
            descendants.extend(new_list.iter().cloned());
            list = new_list;
        }
        descendants
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

    pub fn symlink(&mut self, src: &Path, dst: &Path) -> Result<()> {
        if self.get(dst).is_ok() {
            return Err(create_error(ErrorKind::AlreadyExists));
        }
        let parent = if let Some(parent) = dst.parent() {
            parent
        } else {
            return Err(create_error(ErrorKind::NotFound));
        };
        match self.readonly(parent) {
            Ok(true) => Err(create_error(ErrorKind::PermissionDenied)),
            Ok(false) => {
                self.files.insert(
                    PathBuf::from(dst),
                    Node::Symlink(Symlink::new(PathBuf::from(src))),
                );
                Ok(())
            }
            Err(_) => Err(create_error(ErrorKind::NotFound)),
        }
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
