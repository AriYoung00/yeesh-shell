#[cfg(test)]
mod cmd_input_tests {
    use std::io;

    use derive_more::Display;
    use filesystem::{FakeFileSystem, FileSystem};
    use termion::event::Key;

    use crate::cmd_input::DetectCursorPosAlias;
    use crate::fixture::raw_tty_emulator::RawTTYEmulator;
    use crate::CmdInput;

    #[derive(Display, Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[display(fmt = "{}", string)]
    struct StringWriter {
        pub string:     String,
        pub cursor_pos: usize,
    }

    impl StringWriter {
        pub fn new() -> StringWriter {
            StringWriter {
                string:     String::new(),
                cursor_pos: 0,
            }
        }
    }

    impl io::Write for StringWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.string.push_str(std::str::from_utf8(buf).unwrap());
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[cfg(test)]
    fn setup() -> (CmdInput, RawTTYEmulator) {
        (CmdInput::new(FakeFileSystem::new()), RawTTYEmulator::new())
    }

    #[cfg(test)]
    fn setup_with_fs() -> (CmdInput, RawTTYEmulator, FakeFileSystem) {
        let fs = FakeFileSystem::new();
        (CmdInput::new(fs.clone()), RawTTYEmulator::new(), fs)
    }

    #[cfg(test)]
    fn insert_word(cmd: &mut CmdInput, out: &mut RawTTYEmulator, word: &str) {
        for c in word.chars() {
            cmd.insert(Key::Char(c));
            cmd.render_line(out, 0).expect("Unable to render word");
        }
    }

    #[test]
    fn test_render_line_single_word() {
        let (mut cmd, mut out) = setup();

        // Check that a single word renders correctly
        insert_word(&mut cmd, &mut out, "hello");
        cmd.render_line(&mut out, 0).expect("This is a problem");
        assert_eq!(out.get_line_str(), "hello ");
    }

    #[test]
    fn test_render_empty_after_clear_line_and_render() {
        let (mut cmd, mut out) = setup();
        let mut garbage_out = RawTTYEmulator::new();

        insert_word(&mut cmd, &mut garbage_out, "hello");
        cmd.render_line(&mut garbage_out, 0).expect("This is a problem");
        cmd.clear();
        cmd.render_line(&mut out, 0).expect("This is a problem");
        assert_eq!(out.get_line_str(), " ");
    }

    #[test]
    fn test_move_left() {
        let (mut cmd, mut out) = setup();
        let input = "hello";

        insert_word(&mut cmd, &mut out, input);
        cmd.insert(Key::Left);
        cmd.render_line(&mut out, 0).expect("This is a problem");
        assert_eq!(out.get_cursor_pos(), (input.len(), 1));
    }

    #[test]
    fn test_cmd_input_empty_after_no_input() {
        let (cmd, _out) = setup();

        assert!(cmd.get_cmd().is_empty());
        assert!(cmd.get_input().is_empty());
    }

    #[test]
    fn test_cmd_input_tab_valid_prefix() {
        let (mut cmd, mut out, fs) = setup_with_fs();

        fs.create_dir_all("/test/dir").unwrap();
        insert_word(&mut cmd, &mut out, "te");
        cmd.insert(Key::Char('\t'));
        cmd.render_line(&mut out, 0).expect("Unable to render line");
        assert_eq!(out.get_line_str(), "test/ ");
        println!("out is string '{}'", out.get_line_str());
        assert_eq!(out.get_cursor_pos().0, 6_usize);
    }

    #[test]
    fn test_cmd_input_tab_empty_line_empty_dir() {
        let (mut cmd, mut out, _fs) = setup_with_fs();
        cmd.insert(Key::Char('\t'));
        cmd.render_line(&mut out, 0).expect("Unable to render line");
        assert_eq!(out.get_line_str(), " ");
        assert_eq!(out.get_cursor_pos().0, 1_usize);
    }

    #[test]
    fn test_valid_prefix_cycle() {
        let (mut cmd, mut out, fs) = setup_with_fs();
        fs.create_dir_all("/test/dir").unwrap();
        fs.create_dir_all("/telephone").unwrap();
        insert_word(&mut cmd, &mut out, "te");

        cmd.insert(Key::Char('\t'));
        cmd.render_line(&mut out, 0).expect("Unable to render line");
        assert_eq!(out.get_line_str(), "telephone/ ");

        cmd.insert(Key::Char('\t'));
        cmd.render_line(&mut out, 0).expect("Unable to render line");
        assert_eq!(out.get_line_str(), "test/ ");

        cmd.insert(Key::Char('\t'));
        cmd.render_line(&mut out, 0).expect("Unable to render line");
        assert_eq!(out.get_line_str(), "telephone/ ");
    }

    #[test]
    fn test_input_tab_empty_line_matches_all() {
        let (mut cmd, mut out, fs) = setup_with_fs();
        fs.create_dir_all("/test/dir").unwrap();
        fs.create_dir_all("/telephone").unwrap();

        cmd.insert(Key::Char('\t'));
        cmd.render_line(&mut out, 0).expect("Unable to render line");
        assert_eq!(out.get_line_str(), "telephone/ ");
        assert_eq!(out.get_cursor_pos().0, 11_usize);

        cmd.insert(Key::Char('\t'));
        cmd.render_line(&mut out, 0).expect("Unable to render line");
        assert_eq!(out.get_line_str(), "test/ ");
        assert_eq!(out.get_cursor_pos().0, 6_usize);
    }
}
