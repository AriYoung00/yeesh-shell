#[cfg(test)]
mod token_tests {
    use crate::cmd_input::token::Token;

    fn setup() {}

    #[test]
    fn test_parse_non_quoted_words() {
        let input1: Vec<char> = "one two".chars().collect();
        let tokens1 = Token::parse_input(&input1);
        assert_eq!(tokens1.len(), 2);
        assert_eq!(tokens1[0].get_contents(), "one");
        assert_eq!(tokens1[1].get_contents(), "two");

        let input2 = "hello there how are you today THERE ARE CAPS, this is a very long string!??)(*#)!@(*#!";
        let tokens2 = Token::parse_input(&input2.chars().collect::<Vec<char>>());
        let words: Vec<&str> = input2.split(' ').collect();
        assert_eq!(words.len(), tokens2.len());
        for (idx, word) in words.iter().enumerate() {
            assert_eq!(*word, tokens2[idx].get_contents());
        }
    }

    #[test]
    fn test_parse_quoted_words() {
        let input1 = "\"hello there\" buddy".chars().collect();
        let tokens1 = Token::parse_input(&input1);
        assert_eq!(tokens1.len(), 2);
        assert_eq!(tokens1[0].get_contents(), "hello there");
        assert!(tokens1[0].get_is_quoted());
        assert_eq!(tokens1[0].get_quote_char(), '"');
        assert_eq!(tokens1[1].get_contents(), "buddy");
        assert!(!tokens1[1].get_is_quoted());

        let input2 = "'single quotes' now".chars().collect();
        let tokens2 = Token::parse_input(&input2);
        assert_eq!(tokens2.len(), 2);
        assert_eq!(tokens2[0].get_contents(), "single quotes");
        assert!(tokens2[0].get_is_quoted());
        assert_eq!(tokens2[0].get_quote_char(), '\'');
        assert_eq!(tokens2[1].get_contents(), "now");
        assert!(!tokens2[1].get_is_quoted());
    }

    #[test]
    fn test_non_quoted_start_end_pos() {
        let input1 = "three four".chars().collect();
        let tokens1 = Token::parse_input(&input1);
        assert_eq!(tokens1.len(), 2);
        assert_eq!(tokens1[0].get_start_pos(), 0);
        assert_eq!(tokens1[0].get_end_pos(), 5);
        assert_eq!(tokens1[1].get_start_pos(), 6);
        assert_eq!(tokens1[1].get_end_pos(), 9);

        let input2 = "this is a very long string today";
        let mut pos = 0_usize;
        let tokens2 = Token::parse_input(&input2.chars().collect());
        let words: Vec<&str> = input2.split(' ').collect();
        for (idx, word) in words.iter().enumerate() {
            assert_eq!(tokens2[idx].get_start_pos(), pos);
            if idx == words.len() - 1 {
                pos += word.len() - 1;
            }
            else {
                pos += word.len();
            }
            assert_eq!(tokens2[idx].get_end_pos(), pos);
            pos += 1;
        }
    }

    #[test]
    fn test_quoted_start_end_pos() {
        let input1: Vec<char> = "\"hello there\" buddy".chars().collect();
        let tokens1 = Token::parse_input(&input1);
        assert_eq!(tokens1.len(), 2);
        assert_eq!(tokens1[0].get_start_pos(), 0);
        assert_eq!(tokens1[0].get_end_pos(), 13);
        assert_eq!(tokens1[1].get_start_pos(), 14);
        assert_eq!(tokens1[1].get_end_pos(), 18);

        let input2: Vec<char> = "'hello there' buddy".chars().collect();
        let tokens2 = Token::parse_input(&input2);
        assert_eq!(tokens2.len(), 2);
        assert_eq!(tokens2[0].get_start_pos(), 0);
        assert_eq!(tokens2[0].get_end_pos(), 13);
        assert_eq!(tokens2[1].get_start_pos(), 14);
        assert_eq!(tokens2[1].get_end_pos(), 18);
    }

    #[test]
    fn test_parse_empty_buffer() {
        let input1: Vec<char> = vec![];
        assert_eq!(Token::parse_input(&input1).len(), 0);
    }

    #[test]
    fn test_simple_getters() {
        let token = Token::new("hello".to_string(), false, '"', 0, 4);
        assert_eq!(token.get_contents(), "hello");
        assert!(!token.get_is_quoted());
        assert_eq!(token.get_quote_char(), '"');
        assert_eq!(token.get_start_pos(), 0);
        assert_eq!(token.get_end_pos(), 4);
    }

    #[test]
    fn test_get_assembled() {
        let token1 = Token::new("werd".to_string(), false, '"', 0, 3);
        assert_eq!(token1.get_assembled(), "werd");

        let token2 = Token::new("two werds".to_string(), true, '"', 0, 9);
        assert_eq!(token2.get_assembled(), "\"two werds\"");

        let token3 = Token::new("now three werds".to_string(), true, '\'', 0, 15);
        assert_eq!(token3.get_assembled(), "'now three werds'");
    }
}
