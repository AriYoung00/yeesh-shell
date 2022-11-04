#[derive(Clone, Debug)]
pub struct Token {
    pub contents: String,
    is_quoted:    bool,
    quote_char:   char,

    start_pos: usize,
    end_pos:   usize,
}

impl Token {
    pub fn new(contents: String, is_quoted: bool, quote_char: char, start_pos: usize, end_pos: usize) -> Self {
        Token {
            contents,
            is_quoted,
            quote_char,
            start_pos,
            end_pos,
        }
    }

    pub fn parse_input(input: &Vec<char>) -> Vec<Token> {
        let mut current_arg = vec![];
        let mut is_quoted = false;
        let mut was_quoted = false;
        let mut quote_char = '\'';
        let mut start_pos = 0_usize;

        let mut tokens = vec![];
        for (idx, c) in input.iter().enumerate() {
            match c {
                ' ' if !is_quoted => {
                    if input.len() > 1 {
                        tokens.push(Token {
                            contents: String::from_iter(current_arg.iter()),
                            is_quoted: was_quoted,
                            quote_char,
                            start_pos,
                            end_pos: idx,
                        });
                    }
                    start_pos = idx + 1;
                    current_arg.clear();
                    was_quoted = false;
                }
                '"' | '\'' if !is_quoted => {
                    is_quoted = true;
                    quote_char = *c;
                    start_pos = idx;
                }
                '"' | '\'' if is_quoted && *c == quote_char => {
                    is_quoted = false;
                    was_quoted = true;
                }
                _ => current_arg.push(*c),
            }
        }
        if input.len() > 1 {
            tokens.push(Token {
                contents: String::from_iter(current_arg.iter()),
                is_quoted: was_quoted,
                quote_char,
                start_pos,
                end_pos: input.len() - 1,
            });
        }
        // println!("Tokens: {:?}", tokens);

        tokens
    }

    pub fn assemble_tokens(tokens: &Vec<Token>) -> Vec<char> {
        tokens
            .iter()
            .map(|t| t.get_assembled())
            .intersperse(" ".to_string())
            .collect::<String>()
            .chars()
            .collect()
    }

    pub fn get_assembled(&self) -> String {
        if self.is_quoted {
            format!("{}{}{}", self.quote_char, self.contents, self.quote_char).to_string()
        }
        else {
            self.contents.clone()
        }
    }

    pub fn get_is_quoted(&self) -> bool {
        self.is_quoted
    }

    pub fn get_quote_char(&self) -> char {
        self.quote_char
    }

    pub fn get_start_pos(&self) -> usize {
        self.start_pos
    }

    pub fn get_end_pos(&self) -> usize {
        self.end_pos
    }
}
