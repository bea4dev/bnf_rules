use either::Either;
use std::cmp::min;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Lexer {
    terminal_symbols: Vec<Rc<TerminalSymbol>>,
    eof_symbol: Rc<TerminalSymbol>,
}

impl Lexer {
    pub fn new(mut terminal_symbols: Vec<TerminalSymbol>) -> Self {
        fn blank_tokenizer(source: &Vec<char>, mut current_position: usize) -> usize {
            let mut iteration_count = 0;
            loop {
                let current_char = match source.get(current_position) {
                    Some(ch) => ch,
                    _ => break,
                };
                let chars = ['\t', ' ', '　'];
                if !chars.contains(&current_char) {
                    break;
                }
                iteration_count += 1;
                current_position += 1;
            }
            return iteration_count;
        }
        terminal_symbols.push(TerminalSymbol::new_from_tokenizer_fn(
            blank_tokenizer,
            u32::MAX,
        ));

        let mut symbols = Vec::new();
        for symbol in terminal_symbols {
            symbols.push(Rc::new(symbol));
        }

        return Self {
            terminal_symbols: symbols,
            eof_symbol: Rc::new(TerminalSymbol::new_from_string("EOF", 0)),
        };
    }

    pub fn scan(&self, source: &str) -> Result<Vec<Token>, UnexpectedCharacter> {
        let source = source.chars().collect::<Vec<char>>();
        let source_length = source.len();

        if source_length == 0 {
            let position = TokenPosition::new(0, 0, 1, 1);
            return Ok(vec![Token::new_eof(position, self.eof_symbol.clone())]);
        }

        let mut tokens = Vec::<Token>::new();
        let mut source_index = 0;

        loop {
            let token = self.read_until_token_found(&source, &mut source_index)?;
            if token.symbol_id != u32::MAX {
                tokens.push(token);
            }

            if source_index == source.len() {
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

    fn read_until_token_found(
        &self,
        source: &Vec<char>,
        source_index: &mut usize,
    ) -> Result<Token, UnexpectedCharacter> {
        let mut terminal_symbol = self.eof_symbol.clone();
        let mut text_length = 0;

        let start_position = *source_index;
        for symbol in self.terminal_symbols.iter() {
            let length = symbol.tokenize(source, start_position);
            if length > text_length {
                terminal_symbol = symbol.clone();
                text_length = length;
            }
        }

        let end_position = start_position + text_length;
        let token_text = source[start_position..end_position]
            .iter()
            .collect::<String>();

        *source_index = end_position;

        return if text_length == 0 {
            let mut unexpected = UnexpectedCharacter {
                position: TokenPosition {
                    start_position,
                    text_length: 1,
                    line: 0,
                    column: 0,
                },
                character: source[start_position],
            };

            let mut position_temp_map = HashMap::<usize, &mut TokenPosition>::new();
            position_temp_map.insert(start_position, &mut unexpected.position);
            Self::set_line_and_column_info(source, &mut position_temp_map);

            Err(unexpected)
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
        };
    }

    fn set_line_and_column_info_for_tokens(source: &Vec<char>, tokens: &mut Vec<Token>) {
        let mut position_map = HashMap::<usize, &mut TokenPosition>::new();
        for token in tokens.iter_mut() {
            position_map.insert(token.position.start_position, &mut token.position);
        }
        Self::set_line_and_column_info(source, &mut position_map);
    }

    fn set_line_and_column_info_for_eof_token(source: &Vec<char>, token: &mut Token) {
        let mut position_map = HashMap::<usize, &mut TokenPosition>::new();
        position_map.insert(token.position.start_position, &mut token.position);
        Self::set_line_and_column_info(source, &mut position_map);
    }

    fn set_line_and_column_info(
        source: &Vec<char>,
        position_map: &mut HashMap<usize, &mut TokenPosition>,
    ) {
        let mut i = 0;
        let mut line = 1;
        let mut column = 1;

        let mut previous_column = column;

        loop {
            let char = source[i];

            match position_map.get_mut(&i) {
                Some(position) => {
                    position.line = line;
                    position.column = column;
                }
                _ => {}
            }

            let line_feed = if char == '\n' {
                match Lexer::read_back(source, i, 1) {
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

            i += 1;
            column += 1;

            if i == source.len() {
                break;
            }
        }
    }

    fn read_back(source: &Vec<char>, position: usize, back: usize) -> Option<char> {
        if position < back {
            return None;
        }
        return source.get(position - back).copied();
    }
}

#[derive(Debug)]
pub struct UnexpectedCharacter {
    pub position: TokenPosition,
    pub character: char,
}

type TokenizerFn = fn(source: &Vec<char>, current_position: usize) -> usize;

#[derive(Debug)]
pub struct TerminalSymbol {
    judgement: Either<TokenizerFn, Vec<char>>,
    symbol_id: u32,
}

impl TerminalSymbol {
    pub fn new_from_tokenizer_fn(tokenizer: TokenizerFn, symbol_id: u32) -> Self {
        return Self {
            judgement: Either::Left(tokenizer),
            symbol_id,
        };
    }

    pub fn new_from_string(judge_str: &str, symbol_id: u32) -> Self {
        return Self {
            judgement: Either::Right(judge_str.chars().collect::<Vec<char>>()),
            symbol_id,
        };
    }

    pub fn tokenize(&self, source: &Vec<char>, current_position: usize) -> usize {
        return match &self.judgement {
            Either::Left(tokenizer_fn) => tokenizer_fn(source, current_position),
            Either::Right(string) => {
                for i in 0..string.len() {
                    let current_char = match source.get(current_position + i) {
                        Some(ch) => ch.clone(),
                        _ => return 0, // reject
                    };
                    if current_char != string[i] {
                        return 0; // reject
                    }
                }
                string.len() // accept
            }
        };
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub position: TokenPosition,
    pub text: String,
    pub terminal_symbol: Rc<TerminalSymbol>,
    pub is_eof: bool,
    pub symbol_id: u32,
}

impl Token {
    pub fn new(
        position: TokenPosition,
        text: String,
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
            text: String::new(),
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
