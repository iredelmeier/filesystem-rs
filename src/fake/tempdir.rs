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

use std::path::{Path, PathBuf};
use std::sync::{Mutex, Weak};

use rand;
use rand::Rng;

use TempDir;

use super::Registry;

const SUFFIX_LENGTH: usize = 10;

#[derive(Debug, Clone)]
pub struct FakeTempDir {
    registry: Weak<Mutex<Registry>>,
    path: PathBuf,
}

impl FakeTempDir {
    pub fn new(registry: Weak<Mutex<Registry>>, base: &Path, prefix: &str) -> Self {
        let mut rng = rand::thread_rng();
        let suffix: String = rng.gen_ascii_chars().take(SUFFIX_LENGTH).collect();
        let name = format!("{}_{}", prefix, suffix);
        let path = base.join(prefix).join(name);

        FakeTempDir { registry, path }
    }
}

impl TempDir for FakeTempDir {
    fn path(&self) -> &Path {
        self.path.as_ref()
    }
}

impl Drop for FakeTempDir {
    fn drop(&mut self) {
        if let Some(registry) = self.registry.upgrade() {
            let _ = registry.lock().unwrap().remove_dir_all(&self.path);
        }
    }
}
