use regex::Regex;
use std::cmp::min;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Lexer {
    terminal_symbols: Vec<Rc<TerminalSymbol>>,
    eof_symbol: Rc<TerminalSymbol>,
}

impl Lexer {
    pub fn new(mut terminal_symbols: Vec<TerminalSymbol>) -> Self {
        terminal_symbols.push(TerminalSymbol::new_from_regex(r"[ 　/t]+", u32::MAX));

        let mut symbols = Vec::new();
        for symbol in terminal_symbols {
            symbols.push(Rc::new(symbol));
        }

        return Self {
            terminal_symbols: symbols,
            eof_symbol: Rc::new(TerminalSymbol::new_from_string("EOF", 0)),
        };
    }

    pub fn scan<'input>(
        &self,
        source: &'input str,
    ) -> Result<Vec<Token<'input>>, UnexpectedCharacter> {
        let source_length = source.len();

        if source_length == 0 {
            let position = TokenPosition::new(0, 0, 1, 1);
            return Ok(vec![Token::new_eof(position, self.eof_symbol.clone())]);
        }

        let mut tokens = Vec::<Token>::new();
        let mut current_byte_position = 0;

        loop {
            let token = match self.read_until_token_found(
                &source[current_byte_position..],
                &mut current_byte_position,
            ) {
                Ok(token) => token,
                Err(mut unexpected) => {
                    let mut position_map = HashMap::new();
                    position_map
                        .insert(unexpected.position.start_position, &mut unexpected.position);
                    Self::set_line_and_column_info(source, &mut position_map);

                    return Err(unexpected);
                }
            };
            if token.symbol_id != u32::MAX {
                tokens.push(token);
            }

            if current_byte_position == source.len() {
                break;
            }
        }

        Self::set_line_and_column_info_for_tokens(&source, &mut tokens);

        let eof_position = TokenPosition::new(source_length - 1, 0, 0, 0);
        let mut eof_token = Token::new_eof(eof_position, self.eof_symbol.clone());
        Self::set_line_and_column_info_for_eof_token(&source, &mut eof_token);
        tokens.push(eof_token);

        return Ok(tokens);
    }

    fn read_until_token_found<'input>(
        &self,
        current_input: &'input str,
        current_byte_position: &mut usize,
    ) -> Result<Token<'input>, UnexpectedCharacter> {
        let mut terminal_symbol = self.eof_symbol.clone();
        let mut text_length = 0;

        let start_position = *current_byte_position;

        for symbol in self.terminal_symbols.iter() {
            let length = symbol.tokenize(current_input);
            if length > text_length {
                terminal_symbol = symbol.clone();
                text_length = length;
            }
        }

        let token_text = &current_input[..text_length];

        *current_byte_position += text_length;

        if text_length == 0 {
            Err(UnexpectedCharacter {
                position: TokenPosition {
                    start_position,
                    text_length: 1,
                    line: 0,
                    column: 0,
                },
                character: current_input.chars().next().unwrap(),
            })
        } else {
            let symbol_id = terminal_symbol.symbol_id;
            Ok(Token {
                position: TokenPosition {
                    start_position,
                    text_length,
                    line: 0,
                    column: 0,
                },
                text: token_text,
                terminal_symbol,
                is_eof: false,
                symbol_id,
            })
        }
    }

    fn set_line_and_column_info_for_tokens(input: &str, tokens: &mut Vec<Token>) {
        let mut position_map = HashMap::<usize, &mut TokenPosition>::new();
        for token in tokens.iter_mut() {
            position_map.insert(token.position.start_position, &mut token.position);
        }
        Self::set_line_and_column_info(input, &mut position_map);
    }

    fn set_line_and_column_info_for_eof_token(input: &str, token: &mut Token) {
        let mut position_map = HashMap::<usize, &mut TokenPosition>::new();
        position_map.insert(token.position.start_position, &mut token.position);
        Self::set_line_and_column_info(input, &mut position_map);
    }

    fn set_line_and_column_info(
        input: &str,
        position_map: &mut HashMap<usize, &mut TokenPosition>,
    ) {
        let mut line = 1;
        let mut column = 1;
        let mut byte_position = 0;

        let mut previous_column = column;
        let mut previous_char: Option<char> = None;

        let mut input_chars = input.chars();

        loop {
            let char = match input_chars.next() {
                Some(char) => char,
                None => break,
            };

            match position_map.get_mut(&byte_position) {
                Some(position) => {
                    position.line = line;
                    position.column = column;
                }
                _ => {}
            }

            let line_feed = if char == '\n' {
                match previous_char {
                    Some(previous_char) => {
                        if previous_char == '\r' {
                            //for CRLF
                            column = previous_column;
                            false
                        } else {
                            //for LF
                            true
                        }
                    }
                    _ => {
                        //for LF
                        true
                    }
                }
            } else if char == '\r' {
                //for CR
                true
            } else {
                false
            };

            if line_feed {
                previous_column = column;
                line += 1;
                column = 0;
            }

            previous_char = Some(char);

            byte_position += char.len_utf8();

            column += 1;
        }
    }
}

#[derive(Debug)]
pub struct UnexpectedCharacter {
    pub position: TokenPosition,
    pub character: char,
}

type TokenizerFn = fn(input: &str) -> usize;

#[derive(Debug)]
pub struct TerminalSymbol {
    tokenizer: Tokenizer,
    symbol_id: u32,
}

#[derive(Debug)]
pub enum Tokenizer {
    Keyword(&'static str),
    Regex(Regex),
    Functional(TokenizerFn),
}

impl TerminalSymbol {
    pub fn new_from_tokenizer_fn(tokenizer: TokenizerFn, symbol_id: u32) -> Self {
        Self {
            tokenizer: Tokenizer::Functional(tokenizer),
            symbol_id,
        }
    }

    pub fn new_from_string(keyword: &'static str, symbol_id: u32) -> Self {
        Self {
            tokenizer: Tokenizer::Keyword(keyword),
            symbol_id,
        }
    }

    pub fn new_from_regex(regex: &str, symbol_id: u32) -> Self {
        Self {
            tokenizer: Tokenizer::Regex(Regex::new(format!("^{}", regex).as_str()).unwrap()),
            symbol_id,
        }
    }

    pub fn tokenize(&self, input: &str) -> usize {
        return match &self.tokenizer {
            Tokenizer::Functional(tokenizer_fn) => tokenizer_fn(input),
            Tokenizer::Keyword(keyword) => {
                let mut input_chars = input.chars();
                let mut keyword_chars = keyword.chars();
                let mut current_byte_length = 0;

                loop {
                    let keyword_char = match keyword_chars.next() {
                        Some(c) => c,
                        _ => break current_byte_length,
                    };
                    let current_char = match input_chars.next() {
                        Some(c) => c,
                        _ => return 0, // reject
                    };
                    if current_char != keyword_char {
                        return 0; // reject
                    }

                    current_byte_length += current_char.len_utf8();
                }
            }
            Tokenizer::Regex(regex) => {
                match regex.find(input) {
                    Some(matched) => matched.end(),
                    None => 0, // reject
                }
            }
        };
    }
}

#[derive(Debug, Clone)]
pub struct Token<'input> {
    pub position: TokenPosition,
    pub text: &'input str,
    pub terminal_symbol: Rc<TerminalSymbol>,
    pub is_eof: bool,
    pub symbol_id: u32,
}

impl<'input> Token<'input> {
    pub fn new(
        position: TokenPosition,
        text: &'input str,
        terminal_symbol: Rc<TerminalSymbol>,
        symbol_id: u32,
    ) -> Self {
        return Self {
            position,
            text,
            terminal_symbol,
            is_eof: false,
            symbol_id,
        };
    }

    pub fn new_eof(position: TokenPosition, terminal_symbol: Rc<TerminalSymbol>) -> Self {
        return Self {
            position,
            text: "",
            terminal_symbol,
            is_eof: true,
            symbol_id: 0,
        };
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TokenPosition {
    pub start_position: usize,
    pub text_length: usize,
    pub line: usize,
    pub column: usize,
}

impl TokenPosition {
    pub fn new(start_position: usize, text_length: usize, line: usize, column: usize) -> Self {
        return Self {
            start_position,
            text_length,
            line,
            column,
        };
    }

    pub fn marge_start_position() -> Self {
        return Self {
            start_position: usize::MAX,
            text_length: 0,
            line: usize::MAX,
            column: usize::MAX,
        };
    }

    pub fn marge(&mut self, other_token_position: &TokenPosition) {
        if other_token_position.text_length != 0 {
            self.text_length = if self.start_position == usize::MAX {
                other_token_position.text_length
            } else if other_token_position.start_position == usize::MAX {
                self.text_length
            } else {
                if self.start_position < other_token_position.start_position {
                    (other_token_position.start_position - self.start_position)
                        + other_token_position.text_length
                } else {
                    (self.start_position - other_token_position.start_position) + self.text_length
                }
            };
        }

        self.start_position = min(self.start_position, other_token_position.start_position);
        self.line = min(self.line, other_token_position.line);
        self.column = min(self.column, other_token_position.column);
    }
}
