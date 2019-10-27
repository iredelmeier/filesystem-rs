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
