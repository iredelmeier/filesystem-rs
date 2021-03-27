#![cfg(unix)]
///! This file contains tests for the symlink functionality. Since it's only supported on 
///! objects that implement the `UnixFileSystem` trait, this whole file is restricted to
///! the Unix configuration.
extern crate filesystem;

#[macro_use]
mod utils;

use std::io::ErrorKind;
use std::path::{PathBuf,Path};

use filesystem::UnixFileSystem;
use filesystem::{DirEntry, FakeFileSystem, FileSystem, OsFileSystem, TempDir, TempFileSystem};

macro_rules! test_fs {
    ($name:ident, $fs:expr) => {
        mod $name {
            use super::*;

            make_test!(set_current_dir_fails_if_node_is_broken_symlink, $fs);
            make_test!(set_current_dir_fails_if_node_is_file_symlink, $fs);

            make_test!(is_dir_returns_true_if_node_is_dir_symlink, $fs);
            make_test!(is_dir_returns_false_if_node_is_file_symlink, $fs);
            make_test!(is_dir_returns_false_if_node_is_broken_symlink, $fs);

            make_test!(is_file_returns_true_if_node_is_file_symlink, $fs);
            make_test!(is_file_returns_false_if_node_is_dir, $fs);
            make_test!(is_file_returns_false_if_node_is_broken_symlink, $fs);

            make_test!(symlink_fails_if_something_already_exists, $fs);
            make_test!(create_dir_fails_if_parent_is_broken_symlink, $fs);

            make_test!(create_dir_and_create_file_succeed_inside_symlink_source, $fs);
            make_test!(create_dir_and_create_file_fail_in_file_symlink, $fs);

            make_test!(remove_file_deletes_only_dir_symlink, $fs);
            make_test!(remove_file_deletes_only_file_symlink, $fs);

            make_test!(remove_dir_fails_if_node_is_file_symlink, $fs);
            make_test!(remove_dir_fails_if_node_is_dir_symlink, $fs);
            
            make_test!(remove_dir_inside_symlink_works, $fs);
            make_test!(remove_dir_all_inside_symlink_works, $fs);
            make_test!(remove_file_inside_symlink_works, $fs);

            make_test!(read_dir_fails_if_node_is_broken_symlink, $fs);

            make_test!(write_file_writes_to_new_file_inside_symlink, $fs);
            make_test!(write_file_overwrites_contents_of_existing_file_inside_symlink, $fs);
            make_test!(write_file_overwrites_contents_of_symlink_source_file, $fs);

            make_test!(read_file_returns_symlink_source_contents, $fs);
            make_test!(read_file_works_inside_symlink, $fs);
            make_test!(read_file_fails_if_node_is_broken_symlink, $fs);
            
            make_test!(create_file_writes_to_new_file_inside_symlink, $fs);
            
            make_test!(copy_file_copies_a_file_from_symlink, $fs);
            make_test!(copy_file_copies_a_file_from_inside_symlink, $fs);
            make_test!(copy_file_copies_a_file_to_inside_symlink, $fs);
            make_test!(copy_file_fails_if_original_file_is_broken_symlink, $fs);

            make_test!(rename_renames_a_symlink, $fs);        }
    };
}

#[cfg(unix)]
test_fs!(os, OsFileSystem::new);
#[cfg(unix)]
test_fs!(fake, FakeFileSystem::new);

fn set_current_dir_fails_if_node_is_broken_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
  let path = parent.join("file");
  let link_path = parent.join("file_link");

  fs.symlink(&path, &link_path).unwrap();

  let result = fs.set_current_dir(&link_path);

  assert!(result.is_err());
  assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn set_current_dir_fails_if_node_is_file_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
  let path = parent.join("file");
  fs.create_file(&path, "").unwrap();
  
  let link_path = parent.join("file_link");
  fs.symlink(&path, &link_path).unwrap();

  let result = fs.set_current_dir(&link_path);

  assert!(result.is_err());
  assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn is_dir_returns_true_if_node_is_dir_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");
    fs.create_dir(&path).unwrap();
    
    let link_path = parent.join("link");
    fs.symlink(&path, &link_path).unwrap();

    assert!(fs.is_dir(&link_path));
}

fn is_dir_returns_false_if_node_is_file_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_file");
    fs.create_file(&path, "").unwrap();
    
    let link_path = parent.join("link");
    fs.symlink(&path, &link_path).unwrap();

    assert!(!fs.is_dir(&link_path));
}

fn is_dir_returns_false_if_node_is_broken_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");
    let link_path = parent.join("link");

    fs.symlink(&path, &link_path).unwrap();

    assert!(!fs.is_dir(parent.join("link")));
}

fn is_file_returns_true_if_node_is_file_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_file");

    fs.create_file(&path, "").unwrap();
    
    let link_path = parent.join("link");
    fs.symlink(&path, &link_path).unwrap();

    assert!(fs.is_file(&link_path));
}

fn is_file_returns_false_if_node_is_dir<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    fs.create_dir(&path).unwrap();

    let link_path = parent.join("link");
    fs.symlink(&path, &link_path).unwrap();

    assert!(!fs.is_file(&link_path));
}

fn is_file_returns_false_if_node_is_broken_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let link_path = parent.join("link");
    fs.symlink(parent.join("404"), &link_path).unwrap();

    assert!(!fs.is_file(&link_path));
}

fn symlink_fails_if_something_already_exists<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let file_path = parent.join("file");
    let dir_path = parent.join("dir");
    let symlink_file_path = parent.join("symlink_file");
    let symlink_dir_path = parent.join("symlink_dir");
    let symlink_broken_path = parent.join("symlink_broken");
    let dummy_path = parent.join("dummy");

    fs.create_dir(&dir_path).unwrap();
    fs.create_file(&file_path, "").unwrap();
    fs.symlink(&dir_path, &symlink_dir_path).unwrap();
    fs.symlink(&file_path, &symlink_file_path).unwrap();
    fs.symlink(&parent.join("404"), &symlink_broken_path).unwrap();

    let used_paths = [&file_path, &dir_path, &symlink_dir_path, &symlink_broken_path, &symlink_file_path];
    for path in used_paths.iter() {
        let result = fs.symlink(&dummy_path, path);
        assert!(result.is_err(), "Could create symlink {:?}, that contained another dir/file/symlink", path);
        assert_eq!(result.unwrap_err().kind(), ErrorKind::AlreadyExists);
    }
}

fn create_dir_fails_if_parent_is_broken_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("parent/new_dir");
    let link_path = parent.join("parent");
    fs.symlink(parent.join("404"), &link_path).unwrap();

    let result = fs.create_dir(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn create_dir_and_create_file_succeed_inside_symlink_source<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir_path = parent.join("link/new_dir");
    let file_path = parent.join("link/file_dir");
    let link_path = parent.join("link");
    let source_path = parent.join("real_dir");
    let real_dir_path = parent.join("real_dir/new_dir");
    let real_file_path = parent.join("real_dir/new_dir");
    
    fs.create_dir(&source_path).unwrap();

    fs.symlink(&source_path, &link_path).unwrap();

    fs.create_dir(&dir_path).unwrap();
    fs.create_file(&file_path, "").unwrap();

    assert!(fs.is_dir(real_dir_path));
    assert!(fs.is_dir(real_file_path));
}

fn create_dir_and_create_file_fail_in_file_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir_path = parent.join("link/new_dir");
    let file_path = parent.join("link/file");
    let link_path = parent.join("link");
    let source_path = parent.join("file");
    fs.create_file(&source_path, "").unwrap();
    fs.symlink(&source_path, &link_path).unwrap();

    let result = fs.create_dir(&dir_path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
    
    let result = fs.create_file(&file_path, "");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn remove_file_deletes_only_dir_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("dir");
    let link = parent.join("link");

    fs.create_dir(&path).unwrap();
    fs.symlink(&path, &link).unwrap();

    assert!(fs.is_dir(&path));
    assert!(fs.is_dir(&link));

    fs.remove_file(&link).unwrap();

    assert!(fs.is_dir(&path));
    assert!(!fs.is_dir(&link));
}

fn remove_file_deletes_only_file_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");
    let link = parent.join("link");
    fs.create_file(&path, "").unwrap();
    fs.symlink(&path, &link).unwrap();

    assert!(fs.is_file(&path));
    assert!(fs.is_file(&link));

    fs.remove_file(&link).unwrap();

    assert!(fs.is_file(&path));
    assert!(!fs.is_file(&link));
}

fn remove_dir_fails_if_node_is_file_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");
    let symlink = parent.join("symlink");

    fs.create_file(&path, "").unwrap();
    fs.symlink(&path, &symlink).unwrap();

    let result = fs.remove_dir(&symlink);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
    assert!(fs.is_file(&symlink));
}

fn remove_dir_fails_if_node_is_dir_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    // Symlinks are only deleted with std::remove_file, std::remove_dir fails.
    let path = parent.join("dir");
    let symlink = parent.join("symlink");

    fs.create_dir(&path).unwrap();
    fs.symlink(&path, &symlink).unwrap();

    let result = fs.remove_dir(&symlink);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
    assert!(fs.is_dir(&symlink));
}

fn remove_dir_inside_symlink_works<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir1 = parent.join("dir");
    let dir2 = dir1.join("dir");
    let link = parent.join("link");

    fs.create_dir(&dir1).unwrap();
    fs.create_dir(&dir2).unwrap();
    fs.symlink(&dir1, &link).unwrap();

    let result = fs.remove_dir(link.join("dir"));

    assert!(result.is_ok());
    assert!(fs.is_dir(&dir1));
    assert!(fs.is_dir(&link));
    assert!(!fs.is_dir(&dir2));
}

fn remove_dir_all_inside_symlink_works<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir1 = parent.join("dir");
    let dir2 = dir1.join("dir");
    let file = dir2.join("file");
    let link = parent.join("link");

    fs.create_dir(&dir1).unwrap();
    fs.create_dir(&dir2).unwrap();
    fs.create_file(&file, "").unwrap();
    fs.symlink(&dir1, &link).unwrap();

    let result = fs.remove_dir_all(link.join("dir"));

    assert!(result.is_ok());
    assert!(fs.is_dir(&dir1));
    assert!(fs.is_dir(&link));
    assert!(!fs.is_dir(&dir2));
    assert!(!fs.is_file(&file));
}

fn remove_file_inside_symlink_works<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("dir");
    let file = dir.join("file");
    let link = parent.join("link");

    fs.create_dir(&dir).unwrap();
    fs.create_file(&file, "").unwrap();
    fs.symlink(&dir, &link).unwrap();

    let result = fs.remove_file(link.join("file"));

    assert!(result.is_ok());
    assert!(fs.is_dir(&dir));
    assert!(fs.is_dir(&link));
    assert!(!fs.is_file(&file));
}

fn read_dir_fails_if_node_is_broken_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("broken_symlink");
    let result = fs.read_dir(&path);

    assert!(result.is_err());

    match result {
        Ok(_) => panic!("should be an err"),
        Err(err) => assert_eq!(err.kind(), ErrorKind::NotFound),
    }
}

fn write_file_writes_to_new_file_inside_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("dir");
    let link = parent.join("link");
    let file = link.join("file");
    let real_file = dir.join("file");

    fs.create_dir(&dir).unwrap();
    fs.symlink(&dir, &link).unwrap();
    fs.write_file(&file, "file").unwrap();

    assert!(fs.is_file(&file));
    assert!(fs.is_file(&real_file));

    assert_eq!(fs.read_file_to_string(&file).unwrap(), fs.read_file_to_string(&real_file).unwrap());
}

fn write_file_overwrites_contents_of_existing_file_inside_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("dir");
    let link = parent.join("link");
    let file = link.join("file");
    let real_file = dir.join("file");
    let contents = "some random content";

    fs.create_dir(&dir).unwrap();
    fs.symlink(&dir, &link).unwrap();
    fs.create_file(&file, "").unwrap();
    fs.write_file(&file, contents).unwrap();

    assert!(fs.is_file(&file));
    assert!(fs.is_file(&real_file));

    assert_eq!(contents, fs.read_file_to_string(&real_file).unwrap());
    assert_eq!(contents, fs.read_file_to_string(&file).unwrap());
}

fn write_file_overwrites_contents_of_symlink_source_file<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let link = parent.join("link");
    let file = parent.join("file");
    let contents = "some random content";

    fs.create_file(&file, "").unwrap();
    fs.symlink(&file, &link).unwrap();
    fs.write_file(&link, contents).unwrap();

    assert!(fs.is_file(&file));
    assert!(fs.is_file(&link));

    assert_eq!(contents, fs.read_file_to_string(&file).unwrap());
    assert_eq!(contents, fs.read_file_to_string(&link).unwrap());
}

fn read_file_returns_symlink_source_contents<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let file = parent.join("test.txt");
    let link = parent.join("link");

    let contents = "some random content";

    fs.write_file(&file, contents).unwrap();
    fs.symlink(&file, &link).unwrap();

    assert!(fs.is_file(&file));
    assert!(fs.is_file(&link));

    assert_eq!(contents, fs.read_file_to_string(&file).unwrap());
    assert_eq!(contents, fs.read_file_to_string(&link).unwrap());
}

fn read_file_works_inside_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("dir");
    let file = dir.join("test.txt");
    let link = parent.join("link");
    let linked_file = link.join("test.txt");

    let contents = "some random content";

    fs.create_dir(&dir).unwrap();
    fs.write_file(&file, contents).unwrap();
    fs.symlink(&dir, &link).unwrap();

    assert!(fs.is_file(&file));
    assert!(fs.is_file(&linked_file));

    assert_eq!(contents, fs.read_file_to_string(&file).unwrap());
    assert_eq!(contents, fs.read_file_to_string(&linked_file).unwrap());
}

fn read_file_fails_if_node_is_broken_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let link = parent.join("test.txt");

    fs.symlink(parent.join("file"), &link).unwrap();
    
    let result = fs.read_file(&link);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn create_file_writes_to_new_file_inside_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("dir");
    let link = parent.join("link");
    let file = link.join("file");
    let real_file = dir.join("file");

    fs.create_dir(&dir).unwrap();
    fs.symlink(&dir, &link).unwrap();
    fs.create_file(&file, "file").unwrap();

    assert!(fs.is_file(&file));
    assert!(fs.is_file(&real_file));

    assert_eq!(fs.read_file_to_string(&file).unwrap(), fs.read_file_to_string(&real_file).unwrap());
}

fn copy_file_copies_a_file_from_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let file = parent.join("file");
    let link = parent.join("link");
    let to = parent.join("to");

    let contents = "some random content";

    fs.create_file(&file, &contents).unwrap();
    fs.symlink(&file, &link).unwrap();

    fs.copy_file(&link, &to).unwrap();

    assert_eq!(contents, fs.read_file_to_string(&to).unwrap());
}

fn copy_file_copies_a_file_from_inside_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("dir");
    let link = parent.join("link");
    let file = dir.join("file");
    let from = link.join("file");
    let to = parent.join("to");

    let contents = "some random content";

    fs.create_dir(&dir).unwrap();
    fs.create_file(&file, &contents).unwrap();
    fs.symlink(&dir, &link).unwrap();

    fs.copy_file(&from, &to).unwrap();

    assert_eq!(contents, fs.read_file_to_string(&to).unwrap());
}

fn copy_file_copies_a_file_to_inside_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("dir");
    let link = parent.join("link");
    let from = parent.join("file");
    let to = link.join("file");

    let contents = "some random content";

    fs.create_dir(&dir).unwrap();
    fs.create_file(&from, &contents).unwrap();
    fs.symlink(&dir, &link).unwrap();

    fs.copy_file(&from, &to).unwrap();

    assert_eq!(contents, fs.read_file_to_string(&from).unwrap());
    assert_eq!(contents, fs.read_file_to_string(&to).unwrap());
    assert_eq!(contents, fs.read_file_to_string(&dir.join("file")).unwrap());
}

fn copy_file_fails_if_original_file_is_broken_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    let result = fs.copy_file(&from, &to);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
    assert!(!fs.is_file(&to));
}

fn rename_renames_a_symlink<T: UnixFileSystem + FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    fs.symlink(parent.join("some_file"), &from).unwrap();

    fs.rename(&from, &to).unwrap();
    
    let entries: Vec<PathBuf> = fs.read_dir(&parent).unwrap().map(|e| e.unwrap().path()).collect();
    assert_eq!(1, entries.len());
    assert_eq!(to, entries[0]);
}
