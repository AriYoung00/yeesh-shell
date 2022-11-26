#[cfg(test)]
mod suggester_tests {
    use std::cmp::Ordering::{Equal, Greater, Less};
    use std::io;
    use std::path::Path;

    use filesystem::{FakeFileSystem, FileSystem};

    use crate::cmd_input::suggester::SuggestionType::{Directory, File};
    use crate::cmd_input::suggester::{FileSystemSuggester, Suggester, Suggestion};

    fn setup_filesystem() -> (FileSystemSuggester<FakeFileSystem>, FakeFileSystem) {
        let filesystem = FakeFileSystem::new();
        let suggester = FileSystemSuggester::new(filesystem.clone());

        (suggester, filesystem)
    }

    fn create_files(fs: &FakeFileSystem, files: Vec<&'static str>) -> io::Result<()> {
        for file in files {
            fs.create_file(file, vec![])?;
        }
        Ok(())
    }

    fn create_directories(fs: &FakeFileSystem, directories: Vec<&'static str>) -> io::Result<()> {
        for dir in directories {
            fs.create_dir_all(dir)?;
        }
        Ok(())
    }

    #[test]
    fn test_suggestion_cmp() {
        let mut s1 = Suggestion {
            replacement: "a".to_string(),
            is_prefix:   true,
            s_type:      File,
        };
        let mut s2 = Suggestion {
            replacement: "b".to_string(),
            is_prefix:   true,
            s_type:      File,
        };

        assert_eq!(s1.cmp(&s2), Less);

        s1.is_prefix = false;
        assert_eq!(s1.cmp(&s2), Greater);

        s1.is_prefix = true;
        s2.replacement = "a".to_string();
        assert_eq!(s1.cmp(&s2), Equal);
    }

    #[test]
    fn test_get_search_params() {
        let (suggester, fs) = setup_filesystem();
        create_directories(&fs, vec!["hello"]).unwrap();

        let (mut path, mut search_str) = suggester.get_search_params("");
        assert_eq!(path, Path::new("").into());
        assert_eq!(search_str, "");

        (path, search_str) = suggester.get_search_params("he");
        assert_eq!(path, Path::new("").into());
        assert_eq!(search_str, "he");

        (path, search_str) = suggester.get_search_params("hello/");
        assert_eq!(path, Path::new("hello").into());
        assert_eq!(search_str, "");
    }

    #[test]
    fn test_exact_match() {
        let (mut suggester, fs) = setup_filesystem();
        create_directories(&fs, vec!["a", "aa", "b"]).unwrap();

        let mut suggestions = suggester.get_suggestions("b");
        assert_eq!(suggestions, vec![("b/", true, Directory).into()]);

        suggestions = suggester.get_suggestions("a");
        assert_eq!(
            suggestions,
            vec![("a/", true, Directory).into(), ("aa/", true, Directory).into()]
        );
    }

    #[test]
    fn test_no_match() {
        let (mut suggester, fs) = setup_filesystem();
        create_directories(&fs, vec!["ello"]).unwrap();
        create_files(&fs, vec!["hello", "ello/there"]).unwrap();

        let mut suggestions = suggester.get_suggestions("a");
        assert_eq!(suggestions, vec![]);

        suggestions = suggester.get_suggestions("a/");
        assert_eq!(suggestions, vec![]);
    }

    #[test]
    fn test_current_dir() {
        let (mut suggester, fs) = setup_filesystem();
        create_files(&fs, vec!["hello", "there", "world"]).unwrap();
        create_directories(&fs, vec!["ello", "here", "orld"]).unwrap();

        let mut suggestions = suggester.get_suggestions("");
        assert_eq!(
            suggestions,
            vec![
                ("ello/", true, Directory).into(),
                ("hello", true, File).into(),
                ("here/", true, Directory).into(),
                ("orld/", true, Directory).into(),
                ("there", true, File).into(),
                ("world", true, File).into(),
            ]
        );

        suggestions = suggester.get_suggestions("./");
        assert_eq!(
            suggestions,
            vec![
                ("./ello/", true, Directory).into(),
                ("./hello", true, File).into(),
                ("./here/", true, Directory).into(),
                ("./orld/", true, Directory).into(),
                ("./there", true, File).into(),
                ("./world", true, File).into(),
            ]
        );

        suggestions = suggester.get_suggestions("he");
        assert_eq!(
            suggestions,
            vec![
                ("hello", true, File).into(),
                ("here/", true, Directory).into(),
                ("there", false, File).into(),
            ]
        );

        suggestions = suggester.get_suggestions("./he");
        assert_eq!(
            suggestions,
            vec![
                ("./hello", true, File).into(),
                ("./here/", true, Directory).into(),
                ("./there", false, File).into(),
            ]
        );
    }

    #[test]
    fn test_subdir() {
        let (mut suggester, fs) = setup_filesystem();
        create_directories(&fs, vec!["test", "test/ello"]).unwrap();
        create_files(&fs, vec!["test/hello", "test/there", "test/world"]).unwrap();

        let mut suggestions = suggester.get_suggestions("test/");
        assert_eq!(
            suggestions,
            vec![
                ("test/ello/", true, Directory).into(),
                ("test/hello", true, File).into(),
                ("test/there", true, File).into(),
                ("test/world", true, File).into()
            ]
        );

        suggestions = suggester.get_suggestions("./test/");
        assert_eq!(
            suggestions,
            vec![
                ("./test/ello/", true, Directory).into(),
                ("./test/hello", true, File).into(),
                ("./test/there", true, File).into(),
                ("./test/world", true, File).into()
            ]
        );

        suggestions = suggester.get_suggestions("test/he");
        assert_eq!(
            suggestions,
            vec![("test/hello", true, File).into(), ("test/there", false, File).into(),]
        );

        suggestions = suggester.get_suggestions("./test/he");
        assert_eq!(
            suggestions,
            vec![
                ("./test/hello", true, File).into(),
                ("./test/there", false, File).into(),
            ]
        );
    }

    #[test]
    fn test_absolute_path() {
        let (mut suggester, fs) = setup_filesystem();
        create_directories(&fs, vec!["ello", "here", "orld"]).unwrap();
        create_files(&fs, vec!["hello", "there", "world"]).unwrap();
        fs.set_current_dir(Path::new("ello")).unwrap();

        let mut suggestions = suggester.get_suggestions("/");
        assert_eq!(
            suggestions,
            vec![
                ("/ello/", true, Directory).into(),
                ("/hello", true, File).into(),
                ("/here/", true, Directory).into(),
                ("/orld/", true, Directory).into(),
                ("/there", true, File).into(),
                ("/world", true, File).into(),
            ]
        );

        suggestions = suggester.get_suggestions("/he");
        assert_eq!(
            suggestions,
            vec![
                ("/hello", true, File).into(),
                ("/here/", true, Directory).into(),
                ("/there", false, File).into(),
            ]
        );
    }
}
