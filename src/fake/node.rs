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
