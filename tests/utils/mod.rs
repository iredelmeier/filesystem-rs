
/// Macro to generate a test for a specific filesystem.
macro_rules! make_test {
    ($test:ident, $fs:expr) => {
        #[test]
        fn $test() {
            let fs = $fs();
            let temp_dir = fs.temp_dir("test").unwrap();

            super::$test(&fs, temp_dir.path());
        }
    };
  }