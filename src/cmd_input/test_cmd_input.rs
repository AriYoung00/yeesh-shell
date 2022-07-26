#[cfg(test)]
mod cmd_input_tests {
    use std::io;

    use derive_more::Display;
    use termion::event::Key;

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
    fn insert_word(cmd: &mut CmdInput, word: &str) {
        for c in word.chars() {
            cmd.insert(Key::Char(c));
        }
    }

    #[test]
    fn test_render_line_single_word() {
        let mut cmd_input = CmdInput::new();
        let mut out = RawTTYEmulator::new();

        // Check that a single word renders correctly
        for c in "hello".chars() {
            cmd_input.insert(Key::Char(c));
            cmd_input.render_line(&mut out, 0).unwrap();
        }
        assert_eq!(String::from_iter(out.get_line()), "hello ");
    }
}
