#[cfg(test)]
use std::borrow::Borrow;

use crate::find_intrinsic;
use crate::intrinsics::INTRINSICS;

#[test]
fn test_find_intrinsic() {
    for intrinsic in INTRINSICS.iter() {
        let cmd_string = intrinsic.get_command().to_string();
        let found = find_intrinsic(&cmd_string);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), intrinsic.borrow());
    }
}

#[cfg(test)]
mod cd_tests {
    use filesystem::{FakeFileSystem, FileSystem};
    use path_absolutize::Absolutize;

    use crate::intrinsics::{ChangeDirectory, Intrinsic};

    macro_rules! get_path_str {
        ($variable:expr) => {
            $variable
                .current_dir()
                .unwrap()
                .absolutize()
                .unwrap()
                .to_str()
                .unwrap()
        };
    }

    fn setup() -> (ChangeDirectory<FakeFileSystem>, FakeFileSystem) {
        let fs = FakeFileSystem::new();
        (ChangeDirectory::new(fs.clone()), fs)
    }

    fn create_dirs(fs: &FakeFileSystem, dirs: &Vec<&'static str>) {
        for dir in dirs {
            fs.create_dir_all(dir).unwrap();
        }
    }

    fn get_ne_err(path: &'static str) -> String {
        format!("cd: The directory \"{}\" does not exist\r\n", path)
    }

    #[test]
    fn test_error_conditions() {
        let (cd, fs) = setup();
        fs.create_file("file.txt", "bleh").unwrap();
        create_dirs(&fs, &vec!["hello", "there"]);

        // test too many arguments
        let mut res = cd.handler(&["hello".to_string(), "there".to_string()]);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), "cd: too many arguments");

        // test cd file
        res = cd.handler(&["file.txt".to_string()]);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), "cd: 'file.txt' is not a directory");
    }

    /// Test ChangeDirectory with both absolute and relative paths that don't exist
    #[test]
    fn test_bad_path_handling() {
        let (cd, fs) = setup();

        // No directories exist, try CDing to a relative path, no ./
        let mut res = cd.handler(&["this_doesnt_exist".to_string()]);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), get_ne_err("this_doesnt_exist"));

        // relative path with ./
        res = cd.handler(&["./this_doesnt_exist".to_string()]);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), get_ne_err("./this_doesnt_exist"));

        // try an absolute path
        res = cd.handler(&["/absolutely_not_exist".to_string()]);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), get_ne_err("/absolutely_not_exist"));

        // try an absolute path that's a child of an existent path
        fs.create_dir("/exists").unwrap();
        res = cd.handler(&["/exists/doesnt".to_string()]);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), get_ne_err("/exists/doesnt"));
    }

    #[test]
    /// Test ChangeDirectory with absolute paths that exist
    fn test_good_absolute_path_handling() {
        let (cd, fs) = setup();
        let some_path = "/hello/I/am/a/path";
        let some_other_path = "/hello/I/hello/I/am/a";
        let another_path = "/totally/not/a/path";
        create_dirs(&fs, &vec![some_path, some_other_path, another_path]);

        // test absolute path
        let mut args = vec!["/hello/I".to_string()];
        let mut result = cd.handler(&args);
        assert!(result.is_ok());
        assert_eq!(get_path_str!(fs), "/hello/I");

        // test absolute path which can also be relative to the current dir (overlaps)
        args = vec!["/hello/I/am/a".to_string()];
        result = cd.handler(&args);
        assert!(result.is_ok());
        assert_eq!(get_path_str!(fs), "/hello/I/am/a");

        // test cd to root
        args = vec!["/".to_string()];
        result = cd.handler(&args);
        assert!(result.is_ok());
        assert_eq!(get_path_str!(fs), "/");

        // prepare for unrelated dir test
        args = vec![some_path.to_string()];
        result = cd.handler(&args);
        assert!(result.is_ok());

        // test changing to absolute path unrelated to current path
        args = vec![another_path.to_string()];
        result = cd.handler(&args);
        assert!(result.is_ok());
        assert_eq!(get_path_str!(fs), another_path);
    }

    #[test]
    fn test_good_relative_path_handling() {
        let (cd, fs) = setup();
        let path = "/this/is/a/path/that/exists";
        fs.create_dir_all(path).unwrap();

        // test no ./
        let mut res = cd.handler(&["this".to_string()]);
        assert!(res.is_ok());
        assert_eq!(get_path_str!(fs), "/this");

        // test with ./
        res = cd.handler(&["./is".to_string()]);
        assert!(res.is_ok());
        assert_eq!(get_path_str!(fs), "/this/is");

        // test relative compound path, no ./
        res = cd.handler(&["a/path".to_string()]);
        assert!(res.is_ok());
        assert_eq!(get_path_str!(fs), "/this/is/a/path");

        // test relative compount path with ./
        res = cd.handler(&["./that/exists".to_string()]);
        assert!(res.is_ok());
        assert_eq!(get_path_str!(fs), "/this/is/a/path/that/exists");
    }

    #[test]
    fn test_home_handling() {
        let (cd, fs) = setup();
        let home_dir_path = "/home/Person";
        let home_docs_path = "/home/Person/Documents/stuff";
        create_dirs(&fs, &vec![home_dir_path, home_docs_path]);

        // Test empty argument
        let mut args = vec![];
        let mut res = temp_env::with_var("HOME", Some(home_dir_path), || cd.handler(&args));
        assert!(res.is_ok());
        assert_eq!(get_path_str!(fs), home_dir_path);

        // return to root
        args = vec!["/".to_string()];
        res = cd.handler(&args);
        assert!(res.is_ok());

        // Test only tilde
        args = vec!["~".to_string()];
        res = temp_env::with_var("HOME", Some(home_dir_path), || cd.handler(&args));
        assert!(res.is_ok());
        assert_eq!(get_path_str!(fs), home_dir_path);

        // return to root
        args = vec!["/".to_string()];
        res = cd.handler(&args);
        assert!(res.is_ok());

        // Test tilde with subsequent relative path
        args = vec!["~/Documents/stuff".to_string()];
        res = temp_env::with_var("HOME", Some(home_dir_path), || cd.handler(&args));
        assert!(res.is_ok());
        assert_eq!(get_path_str!(fs), home_docs_path);
    }
}
