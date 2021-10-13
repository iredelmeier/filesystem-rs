# miguelandres/filesystem-rs

This has been forked from [iredelmeier/filesystem-rs](http://github.com/iredelmeier/filesystem-rs). I needed some more functionality than the original package offered, namely symlinks, so I wrote it myself. I wrote a PR for Isobel to review but she hasn't got back to me, therefore I'll be treating this as an isolated fork.

### Real, fake, and mock implementations of file system operations.
[![Build Status](https://github.com/miguelandres/filesystem-rs/actions/workflows/rust_build_and_test.yml/badge.svg?branch=main)](https://github.com/miguelandres/filesystem-rs/actions/workflows/rust_build_and_test.yml)
[![Documentation](https://github.com/miguelandres/filesystem-rs/actions/workflows/rust_doc_generator.yml/badge.svg?branch=main)](https://miguelandres.github.io/filesystem-rs/filesystem/)


[Documentation](https://miguelandres.github.io/filesystem-rs/filesystem/)

filesystem-rs provides real, fake, and mock implementations of file system-related functionality. It abstracts away details of certain common but complex operations (e.g., setting permissions) and makes it easier to test any file system-related logic without having to wait for slow I/O operations or coerce the file system into particular states.
