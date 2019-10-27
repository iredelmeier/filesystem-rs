# Changelog

## [Unreleased](https://github.com/iredelmeier/filesystem-rs/compare/v0.4.4...HEAD)

### Fixed

* `FakeFileSystem::copy_file` uses `ErrorKind::NotFound` on attempts to copy a file that doesn't exist
* `FakeFileSystem::remove_dir_all` requires all descendants to be readable, corresponding to the behaviour of `OsFileSystem::remove_dir_all`

## [v0.4.4](https://github.com/olivierlacan/keep-a-changelog/compare/v0.4.3...v0.4.4)

### Added

* `FileSystem::read_file_into` method (thanks @jean-airoldie)

### Fixed

* `FakeFileSystem::read_dir` now returns only children, not all descendants (thanks @jean-airoldie)
