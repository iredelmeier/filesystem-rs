use std::path::{Path, PathBuf};
use std::sync::{Mutex, Arc};

use rand;
use rand::Rng;

use TempDir;

use super::Registry;

const SUFFIX_LENGTH: usize = 10;

#[derive(Debug, Clone)]
pub struct FakeTempDir {
    registry: Arc<Mutex<Registry>>,
    path: PathBuf,
}

impl FakeTempDir {
    pub fn new(registry: Arc<Mutex<Registry>>, base: &Path, prefix: &str) -> Self {
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
        let _ = self.registry.lock().unwrap().remove_dir_all(&self.path);
    }
}
