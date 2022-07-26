#[cfg(test)]
mod test_fixture_tests {
    use std::io;

    use crate::cmd_input::{DetectCursorPosAlias, IoWriteAlias};
    use crate::fixture::raw_tty_emulator::RawTTYEmulator;

    macro_rules! up {
        () => {
            "\x1B[1A"
        };

        ($a:expr) => {
            concat!("\x1B[", $a, 'A')
        };
    }

    macro_rules! down {
        () => {
            "\x1B[1B"
        };

        ($a:expr) => {
            concat!("\x1B[", $a, 'B')
        };
    }

    macro_rules! left {
        () => {
            "\x1B[1D"
        };

        ($a:expr) => {
            concat!("\x1B[", $a, 'D')
        };
    }

    macro_rules! right {
        () => {
            "\x1B[1C"
        };

        ($a:expr) => {
            concat!("\x1B[", $a, 'C')
        };
    }

    macro_rules! goto {
        ($x:expr, $y:expr) => {
            concat!("\x1B[", $y, ';', $x, 'H')
        };
    }

    macro_rules! home {
        () => {
            "\x1B[H"
        };
    }

    #[test]
    fn test_util_macros() {
        assert_eq!(up!(), "\x1B[1A");
        assert_eq!(up!(7), "\x1B[7A");
        assert_eq!(up!(18), "\x1B[18A");

        assert_eq!(down!(), "\x1B[1B");
        assert_eq!(down!(7), "\x1B[7B");
        assert_eq!(down!(18), "\x1B[18B");

        assert_eq!(down!(), "\x1B[1B");
        assert_eq!(down!(7), "\x1B[7B");
        assert_eq!(down!(18), "\x1B[18B");

        assert_eq!(right!(), "\x1B[1C");
        assert_eq!(right!(7), "\x1B[7C");
        assert_eq!(right!(18), "\x1B[18C");

        assert_eq!(goto!(7, 5), "\x1B[5;7H");
        assert_eq!(goto!(18, 11), "\x1B[11;18H");

        assert_eq!(home!(), "\x1B[H");
    }

    #[test]
    fn test_init() {
        let mut thing: RawTTYEmulator = RawTTYEmulator::new();

        assert!(thing.get_line().is_empty());
        assert_eq!(thing.get_text().len(), 1);
        assert_eq!(thing.get_line_str(), "");
        assert_eq!(thing.get_cursor_pos(), (1, 1));
    }

    #[test]
    fn test_move_vertical() -> io::Result<()> {
        let mut thing: RawTTYEmulator = RawTTYEmulator::new();

        // Make sure moving up at (0, 0) does nothing
        thing.write(up!(10).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 1));

        // Move down
        thing.write(down!(10).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 11));

        // Move up a bit and make sure we're in the right place
        thing.write(up!(5).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 6));

        // move up past origin again
        thing.write(up!(150).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 1));

        Ok(())
    }

    #[test]
    fn test_move_horizontal() -> io::Result<()> {
        let mut thing: RawTTYEmulator = RawTTYEmulator::new();

        // Make sure moving left at (0, 0) does nothing
        thing.write(left!(5).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 1));

        // Move right 10
        thing.write(right!(10).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (11, 1));

        // Move left 5
        thing.write(left!(5).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (6, 1));

        // Move left past origin
        thing.write(left!(100).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 1));

        Ok(())
    }

    #[test]
    fn test_goto() -> io::Result<()> {
        let mut thing: RawTTYEmulator = RawTTYEmulator::new();

        thing.write(goto!(10, 10).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (10, 10));

        thing.write(goto!(5, 1).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (5, 1));

        Ok(())
    }

    #[test]
    fn test_home() -> io::Result<()> {
        let mut thing: RawTTYEmulator = RawTTYEmulator::new();

        thing.write(goto!(10, 10).as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (10, 10));

        thing.write(home!().as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 1));

        Ok(())
    }

    #[test]
    fn test_newline() -> io::Result<()> {
        let mut thing: RawTTYEmulator = RawTTYEmulator::new();
        let init_len = thing.get_text().len();

        thing.write(&[b'\n']).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 2));
        assert_eq!(thing.get_text().len(), init_len + 1);

        thing.write(goto!(10, 10).as_bytes()).expect("This is a problem");
        thing.write(&[b'\n']).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (10, 11));
        assert_eq!(thing.get_text().len(), 11);

        Ok(())
    }

    #[test]
    fn test_carriage_return() -> io::Result<()> {
        let mut thing: RawTTYEmulator = RawTTYEmulator::new();

        thing.write(&[b'\r']).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 1));

        thing.write(goto!(10, 10).as_bytes()).expect("This is a problem");
        thing.write(&[b'\r']).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1, 10));

        Ok(())
    }

    #[test]
    fn test_input_at_correct_pos() -> io::Result<()> {
        let mut thing: RawTTYEmulator = RawTTYEmulator::new();
        let input = "hello";

        thing.write(input.as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (1 + input.len(), 1));
        assert_eq!(thing.get_line_str(), input.to_owned() + " ");

        thing.write(goto!(10, 10).as_bytes()).expect("This is a problem");
        thing.write(input.as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_cursor_pos(), (10 + input.len(), 10));
        assert_eq!(
            thing.get_line_str(),
            format!("{}{} ", String::from_utf8_lossy(vec![b' '; 9].as_slice()), input)
        );

        Ok(())
    }

    #[test]
    fn test_input_overwrite() -> io::Result<()> {
        let mut thing: RawTTYEmulator = RawTTYEmulator::new();
        let input1 = "hello";
        let input2 = "there";
        let input3 = "w";

        thing.write(input1.as_bytes()).expect("This is a problem");
        thing.write(home!().as_bytes()).expect("This is a problem");
        thing.write(input2.as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_line_str(), input2.to_owned() + " ");

        thing.write(home!().as_bytes()).expect("This is a problem");
        thing.write(input3.as_bytes()).expect("This is a problem");
        assert_eq!(thing.get_line_str(), "where ");

        Ok(())
    }
}
