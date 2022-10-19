#[derive(Clone, Debug)]
pub struct Token {
    pub contents: String,
    is_quoted: bool,
    quote_char: char,
    is_active: bool
}

impl Token {
    pub fn parse_input(input: &Vec<char>, cursor_pos: usize) -> Vec<Token> {
        let mut current_arg = vec![];
        let mut is_quoted = false;
        let mut was_quoted = false;
        let mut quote_char = '\'';
        let mut start_pos = 0_usize;
        let mut found_active = false;

        let mut tokens = vec![];
        for (idx, c) in input.iter().enumerate() {
            match c {
                ' ' if !is_quoted => {
                    if input.len() > 1 {
                        tokens.push(Token {
                            contents: String::from_iter(current_arg.iter()),
                            is_quoted: was_quoted,
                            quote_char,
                            is_active: cursor_pos >= start_pos && !found_active
                        });
                        found_active = cursor_pos >= start_pos;
                    }
                    start_pos = idx + 1;
                    current_arg.clear()
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
                is_active: cursor_pos >= start_pos && !found_active
            });
        }
        // println!("Tokens: {:?}", tokens);

        tokens
    }

    pub fn assemble_tokens(tokens: &Vec<Token>) -> Vec<char> {
        let mut output_chars = vec![];
        tokens.iter()
            .map(|t| t.get_assembled())
            .for_each(|s| output_chars.extend(s.chars()));

        output_chars
    }

    pub fn get_assembled(&self) -> String {
        if self.is_quoted {
            format!("{}{}{}", self.quote_char, self.contents, self.quote_char).to_string()
        }
        else {
            self.contents.clone()
        }
    }

    pub fn get_is_active(&self) -> bool {
        self.is_active
    }
}