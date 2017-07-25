extern crate filesystem;

use std::io::ErrorKind;
use std::path::Path;

use filesystem::{FakeFileSystem, FileSystem, OsFileSystem, TempDir};

macro_rules! make_test {
    ($test:ident, $fs:expr) => {
        #[test]
        fn $test() {
            let fs = $fs();
            let temp_dir = fs.temp_dir("test").unwrap();

            super::$test(&fs, temp_dir.path());
        }
    }
}

macro_rules! test_fs {
    ($name:ident, $fs:expr) => {
        mod $name {
            use super::*;

            make_test!(set_current_dir_fails_if_path_does_not_exists, $fs);
            make_test!(set_current_dir_fails_if_path_is_a_file, $fs);

            make_test!(is_dir_returns_true_if_path_is_dir, $fs);
            make_test!(is_dir_returns_false_if_path_is_file, $fs);
            make_test!(is_dir_returns_false_if_path_does_not_exist, $fs);

            make_test!(is_file_returns_true_if_path_is_file, $fs);
            make_test!(is_file_returns_false_if_path_is_dir, $fs);
            make_test!(is_file_returns_false_if_path_does_not_exist, $fs);

            make_test!(create_dir_creates_new_dir, $fs);
            make_test!(create_dir_fails_if_dir_already_exists, $fs);
            make_test!(create_dir_fails_if_parent_does_not_exist, $fs);

            make_test!(create_dir_all_creates_dirs_in_path, $fs);
            make_test!(create_dir_all_still_succeeds_if_any_dir_already_exists, $fs);

            make_test!(remove_dir_deletes_dir, $fs);
            make_test!(remove_dir_only_deletes_child, $fs);
            make_test!(remove_dir_fails_if_path_does_not_exist, $fs);
            make_test!(remove_dir_fails_if_path_is_a_file, $fs);
            make_test!(remove_dir_fails_if_dir_is_not_empty, $fs);

            make_test!(remove_dir_all_removes_dir_and_contents, $fs);
            make_test!(remove_dir_all_fails_if_path_is_a_file, $fs);

            make_test!(write_file_writes_to_new_file, $fs);
            make_test!(write_file_overwrites_contents_of_existing_file, $fs);
            make_test!(write_file_fails_if_file_is_readonly, $fs);

            make_test!(read_file_returns_contents_as_bytes, $fs);
            make_test!(read_file_fails_if_file_does_not_exist, $fs);

            make_test!(create_file_writes_writes_to_new_file, $fs);
            make_test!(create_file_fails_if_file_already_exists, $fs);

            make_test!(readonly_returns_write_permission, $fs);
            make_test!(readonly_fails_if_path_does_not_exist, $fs);

            make_test!(set_readonly_toggles_write_permission_of_file, $fs);
            make_test!(set_readonly_toggles_write_permission_of_dir, $fs);
            make_test!(set_readonly_fails_if_path_does_not_exist, $fs);

            make_test!(temp_dir_creates_tempdir, $fs);
            make_test!(temp_dir_creates_unique_dir, $fs);
        }
    }
}

test_fs!(os, OsFileSystem::new);
test_fs!(fake, FakeFileSystem::new);

fn set_current_dir_fails_if_path_does_not_exists<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("does_not_exist");

    let result = fs.set_current_dir(path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn set_current_dir_fails_if_path_is_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");

    fs.create_file(&path, "").unwrap();

    let result = fs.set_current_dir(path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn is_dir_returns_true_if_path_is_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    fs.create_dir(&path).unwrap();

    assert!(fs.is_dir(&path));
}

fn is_dir_returns_false_if_path_is_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    fs.create_file(&path, "").unwrap();

    assert!(!fs.is_dir(&path));
}

fn is_dir_returns_false_if_path_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    assert!(!fs.is_dir(parent.join("does_not_exist")));
}

fn is_file_returns_true_if_path_is_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_file");

    fs.create_file(&path, "").unwrap();

    assert!(fs.is_file(&path));
}

fn is_file_returns_false_if_path_is_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    fs.create_dir(&path).unwrap();

    assert!(!fs.is_file(&path));
}

fn is_file_returns_false_if_path_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    assert!(!fs.is_file(parent.join("does_not_exist")));
}

fn create_dir_creates_new_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    let result = fs.create_dir(&path);

    assert!(result.is_ok());
    assert!(fs.is_dir(path));
}

fn create_dir_fails_if_dir_already_exists<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    fs.create_dir(&path).unwrap();

    let result = fs.create_dir(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::AlreadyExists);
}

fn create_dir_fails_if_parent_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("parent/new_dir");

    let result = fs.create_dir(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn create_dir_all_creates_dirs_in_path<T: FileSystem>(fs: &T, parent: &Path) {
    let result = fs.create_dir_all(parent.join("a/b/c"));

    assert!(result.is_ok());
    assert!(fs.is_dir(parent.join("a")));
    assert!(fs.is_dir(parent.join("a/b")));
    assert!(fs.is_dir(parent.join("a/b/c")));
}

fn create_dir_all_still_succeeds_if_any_dir_already_exists<T: FileSystem>(fs: &T, parent: &Path) {
    fs.create_dir_all(parent.join("a/b")).unwrap();

    let result = fs.create_dir_all(parent.join("a/b/c"));

    assert!(result.is_ok());
    assert!(fs.is_dir(parent.join("a")));
    assert!(fs.is_dir(parent.join("a/b")));
    assert!(fs.is_dir(parent.join("a/b/c")));
}

fn remove_dir_deletes_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("dir");

    fs.create_dir(&path).unwrap();

    let result = fs.remove_dir(&path);

    assert!(result.is_ok());
    assert!(!fs.is_dir(&path));
}

fn remove_dir_only_deletes_child<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("parent/child");

    fs.create_dir_all(&path).unwrap();

    let result = fs.remove_dir(&path);

    assert!(result.is_ok());
    assert!(fs.is_dir(parent.join("parent")));
    assert!(!fs.is_dir(parent.join("child")));
}

fn remove_dir_fails_if_path_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let result = fs.remove_dir(parent.join("does_not_exist"));

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn remove_dir_fails_if_path_is_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");

    fs.create_file(&path, "").unwrap();

    let result = fs.remove_dir(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
    assert!(fs.is_file(&path));
}

fn remove_dir_fails_if_dir_is_not_empty<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("dir");
    let child = path.join("file");

    fs.create_dir(&path).unwrap();
    fs.create_file(&child, "").unwrap();

    let result = fs.remove_dir(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
    assert!(fs.is_dir(&path));
    assert!(fs.is_file(&child));
}

fn remove_dir_all_removes_dir_and_contents<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("dir");
    let child = path.join("file");

    fs.create_dir(&path).unwrap();
    fs.create_file(&child, "").unwrap();

    let result = fs.remove_dir_all(&path);

    assert!(result.is_ok());
    assert!(!fs.is_dir(&path));
    assert!(!fs.is_file(&child));
    assert!(fs.is_dir(parent));
}

fn remove_dir_all_fails_if_path_is_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");

    fs.create_file(&path, "").unwrap();

    let result = fs.remove_dir_all(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
    assert!(fs.is_file(&path));
}

fn write_file_writes_to_new_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_file");
    let result = fs.write_file(&path, "new contents");

    assert!(result.is_ok());

    let contents = String::from_utf8(fs.read_file(path).unwrap()).unwrap();

    assert_eq!(&contents, "new contents");
}

fn write_file_overwrites_contents_of_existing_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    fs.write_file(&path, "old contents").unwrap();

    let result = fs.write_file(&path, "new contents");

    assert!(result.is_ok());

    let contents = String::from_utf8(fs.read_file(path).unwrap()).unwrap();

    assert_eq!(&contents, "new contents");
}

fn write_file_fails_if_file_is_readonly<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    fs.create_file(&path, "").unwrap();
    fs.set_readonly(&path, true).unwrap();

    let result = fs.write_file(&path, "test contents");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
}

fn read_file_returns_contents_as_bytes<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");

    fs.write_file(&path, "test text").unwrap();

    let result = fs.read_file(&path);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), br"test text");
}

fn read_file_fails_if_file_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let result = fs.read_file(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn create_file_writes_writes_to_new_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");
    let result = fs.create_file(&path, "new contents");

    assert!(result.is_ok());

    let contents = String::from_utf8(fs.read_file(path).unwrap()).unwrap();

    assert_eq!(&contents, "new contents");
}

fn create_file_fails_if_file_already_exists<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    fs.create_file(&path, "contents").unwrap();

    let result = fs.create_file(&path, "new contents");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::AlreadyExists);
}

fn readonly_returns_write_permission<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    fs.create_file(&path, "").unwrap();

    let result = fs.readonly(&path);

    assert!(result.is_ok());
    assert!(!result.unwrap());

    fs.set_readonly(&path, true).unwrap();

    let result = fs.readonly(&path);

    assert!(result.is_ok());
    assert!(result.unwrap());
}

fn readonly_fails_if_path_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let result = fs.readonly(parent.join("does_not_exist"));

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn set_readonly_toggles_write_permission_of_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    fs.create_file(&path, "").unwrap();

    let result = fs.set_readonly(&path, true);

    assert!(result.is_ok());
    assert!(fs.write_file(&path, "readonly").is_err());

    let result = fs.set_readonly(&path, false);

    assert!(result.is_ok());
    assert!(fs.write_file(&path, "no longer readonly").is_ok());
}

fn set_readonly_toggles_write_permission_of_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_dir");

    fs.create_dir(&path).unwrap();

    let result = fs.set_readonly(&path, true);

    assert!(result.is_ok());
    assert!(fs.write_file(&path.join("file"), "").is_err());

    let result = fs.set_readonly(&path, false);

    assert!(result.is_ok());
    assert!(fs.write_file(&path.join("file"), "").is_ok());
}

fn set_readonly_fails_if_path_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let result = fs.set_readonly(parent.join("does_not_exist"), true);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);

    let result = fs.set_readonly(parent.join("does_not_exist"), true);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn temp_dir_creates_tempdir<T: FileSystem>(fs: &T, _: &Path) {
    let path = {
        let result = fs.temp_dir("test");

        assert!(result.is_ok());

        let temp_dir = result.unwrap();

        assert!(fs.is_dir(temp_dir.path()));

        temp_dir.path().to_path_buf()
    };

    assert!(!fs.is_dir(&path));
    assert!(fs.is_dir(path.parent().unwrap()));
}

fn temp_dir_creates_unique_dir<T: FileSystem>(fs: &T, _: &Path) {
    let first = fs.temp_dir("test").unwrap();
    let second = fs.temp_dir("test").unwrap();

    assert_ne!(first.path(), second.path());
}
