#[cfg(test)]
mod cmd_input_tests {
    use std::io;

    use derive_more::Display;
    use lazy_static::lazy_static;
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
    lazy_static! {
        static ref THROWAWAY_FS: MemoryFS = MemoryFS::new();
    }
    #[cfg(test)]
    fn setup() -> (CmdInput<'static, MemoryFS>, RawTTYEmulator) {
        (CmdInput::new(&THROWAWAY_FS), RawTTYEmulator::new())
    }

    #[cfg(test)]
    fn insert_word(cmd: &mut CmdInput<MemoryFS>, out: &mut RawTTYEmulator, word: &str) {
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
}
