use std::cmp::min;
use std::collections::HashMap;
use std::rc::Rc;
use either::Either;
use regex::{Error, Regex};



pub struct Lexer {
    terminal_symbols: Vec<Rc<TerminalSymbol>>,
    eof_symbol: Rc<TerminalSymbol>
}



impl Lexer {

    pub fn new(mut terminal_symbols: Vec<TerminalSymbol>) -> Self {
        terminal_symbols.push(TerminalSymbol::new_from_regex(r"(\t|[ 　])+", usize::MAX).unwrap());

        let mut symbols = Vec::new();
        for symbol in terminal_symbols {
            symbols.push(Rc::new(symbol));
        }

        return Self {
            terminal_symbols: symbols,
            eof_symbol: Rc::new(TerminalSymbol::new_from_string("EOF", 0))
        };
    }

    pub fn scan(&self, source: &str) -> Result<Vec<Token>, UnexpectedToken> {

        let source = source.chars().collect::<Vec<char>>();
        let source_length = source.len();

        if source_length == 0 {
            let position = TokenPosition::new(0, 0, 1, 1);
            return Ok(vec![Token::new_eof(position, self.eof_symbol.clone())]);
        }

        let mut tokens = Vec::<Token>::new();
        let mut source_index = 0;
        let mut current_line = 1;
        let mut current_column = 1;

        loop {
            let token = self.read_until_token_found(&source, &mut source_index)?;
            if token.symbol_id != usize::MAX {
                tokens.push(token);
            }

            if source_index == source.len() {
                break;
            }
        }

        let eof_position = TokenPosition::new(source_length - 1, 0, 0, 0);
        tokens.push(Token::new_eof(eof_position, self.eof_symbol.clone()));

        Lexer::set_line_and_column_info(&source, &mut tokens);

        return Ok(tokens);
    }

    pub fn read_until_token_found(&self, source: &Vec<char>, source_index: &mut usize) -> Result<Token, UnexpectedToken> {

        let start_position = *source_index;

        let mut string_symbol_token = Option::<Token>::None;
        let mut regex_symbol_token = Option::<Token>::None;
        let mut i = start_position;
        let mut word_buffer = String::new();
        let mut phase = 0;

        'all : loop {
            let char = source[i];
            word_buffer.push(char);

            let word_length = i - start_position + 1;

            let mut is_matched = false;
            for terminal_symbol in self.terminal_symbols.iter() {
                if phase == 0 {
                    if terminal_symbol.judgement.is_left() {
                        continue;
                    }
                } else {
                    if terminal_symbol.judgement.is_right() {
                        continue;
                    }
                }

                if terminal_symbol.is_match(word_buffer.as_str()) {
                    is_matched = true;
                    break;
                }
            }

            if !is_matched || i + 1 == source.len() {
                if word_length == 1 && !is_matched {

                    if phase == 0 {
                        i = start_position;
                        word_buffer = String::new();
                        phase = 1;
                        continue;
                    } else {
                        i = start_position;
                        word_buffer = String::new();

                        loop {
                            let char = source[i];
                            word_buffer.push(char);

                            let word_length = i - start_position + 1;

                            let mut matched_symbol = Option::<Rc<TerminalSymbol>>::None;

                            for terminal_symbol in self.terminal_symbols.iter() {
                                if terminal_symbol.judgement.is_right() {
                                    continue;
                                }

                                if terminal_symbol.is_match_strict(word_buffer.as_str()) {
                                    matched_symbol = Some(terminal_symbol.clone());
                                }
                            }

                            match matched_symbol {
                                Some(symbol) => {
                                    let position = TokenPosition::new(start_position, word_length, 0, 0);
                                    regex_symbol_token = Some(Token::new(position, word_buffer.clone(), symbol.clone(), symbol.symbol_id));
                                    break 'all;
                                },
                                _ => {}
                            }

                            if i + 1 == source.len() {
                                break 'all;
                            }

                            i += 1;
                        }

                    }

                }

                let mut word_length = word_length;

                if !is_matched {
                    word_buffer.pop();
                    word_length -= 1;
                }

                let mut matched_symbol = Option::<Rc<TerminalSymbol>>::None;

                for terminal_symbol in self.terminal_symbols.iter() {
                    if phase == 0 {
                        if terminal_symbol.judgement.is_left() {
                            continue;
                        }
                    } else {
                        if terminal_symbol.judgement.is_right() {
                            continue;
                        }
                    }

                    if terminal_symbol.is_match_strict(word_buffer.as_str()) {
                        matched_symbol = Some(terminal_symbol.clone());
                    }
                }

                match matched_symbol {
                    Some(symbol) => {
                        let position = TokenPosition::new(start_position, word_length, 0, 0);
                        if phase == 0 {
                            string_symbol_token = Some(Token::new(position, word_buffer.clone(), symbol.clone(), symbol.symbol_id));
                        } else {
                            regex_symbol_token = Some(Token::new(position, word_buffer.clone(), symbol.clone(), symbol.symbol_id));
                        }
                    },
                    _ => {}
                }

                if phase == 0 {
                    if string_symbol_token.is_some() {
                        break;
                    }
                    i = start_position;
                    word_buffer = String::new();
                    phase = 1;
                    continue;
                } else {
                    break;
                }
            }

            i += 1;
        }


        return match string_symbol_token {
            Some(string_token) => {
                match regex_symbol_token {
                    Some(regex_token) => {
                        if string_token.position.text_length >= regex_token.position.text_length {
                            *source_index = start_position + string_token.position.text_length;
                            Ok(string_token)
                        } else {
                            *source_index = start_position + regex_token.position.text_length;
                            Ok(regex_token)
                        }
                    },
                    _ => {
                        *source_index = start_position + string_token.position.text_length;
                        Ok(string_token)
                    }
                }
            },
            _ => {
                match regex_symbol_token {
                    Some(regex_token) => {
                        *source_index = start_position + regex_token.position.text_length;
                        Ok(regex_token)
                    },
                    _ => {
                        let position = TokenPosition::new(start_position, 1, 0, 0);
                        Err(UnexpectedToken { position, text: source[start_position].to_string() })
                    }
                }
            }
        };
    }


    pub fn set_line_and_column_info(source: &Vec<char>, tokens: &mut Vec<Token>) {
        let mut i = 0;
        let mut line = 1;
        let mut column = 1;

        let mut position_map = HashMap::<usize, Vec<&mut TokenPosition>>::new();
        for token in tokens.iter_mut() {
            let positions = position_map.entry(token.position.start_position).or_insert_with(|| vec![]);
            positions.push(&mut token.position);
        }

        let mut previous_column = column;

        loop {
            let char = source[i];

            match position_map.get_mut(&i) {
                Some(positions) => {
                    for position in positions.iter_mut() {
                        position.line = line;
                        position.column = column;
                    }
                },
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
                    },
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
                break
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
pub struct UnexpectedToken {
    pub position: TokenPosition,
    pub text: String
}

#[derive(Debug)]
pub struct TerminalSymbol {
    judgement: Either<Regex, String>,
    symbol_id: usize
}

impl TerminalSymbol {

    pub fn new_from_regex(regex_str: &str, symbol_id: usize) -> Result<Self, Error> {
        let regex = Regex::new(format!(r"^{}$", regex_str).as_str())?;
        return Ok(Self {
            judgement: Either::Left(regex),
            symbol_id
        });
    }

    pub fn new_from_string(judge_str: &str, symbol_id: usize) -> Self {
        return Self {
            judgement: Either::Right(judge_str.to_string()),
            symbol_id
        };
    }


    pub fn is_match(&self, target_str: &str) -> bool {
        return match &self.judgement {
            Either::Left(regex) => regex.is_match(target_str),
            Either::Right(judge_string) => {
                if target_str.len() <= judge_string.len() {
                    judge_string.starts_with(target_str)
                } else {
                    false
                }
            }
        };
    }

    pub fn is_match_strict(&self, target_str: &str) -> bool {
        return match &self.judgement {
            Either::Left(regex) => regex.is_match(target_str),
            Either::Right(judge_string) => target_str == judge_string
        };
    }

}




#[derive(Debug, Clone)]
pub struct Token {
    pub position: TokenPosition,
    pub text: String,
    pub terminal_symbol: Rc<TerminalSymbol>,
    pub is_eof: bool,
    pub symbol_id: usize
}


impl Token {

    pub fn new(position: TokenPosition, text: String, terminal_symbol: Rc<TerminalSymbol>, symbol_id: usize) -> Self {
        return Self {
            position,
            text,
            terminal_symbol,
            is_eof: false,
            symbol_id
        }
    }

    pub fn new_eof(position: TokenPosition, terminal_symbol: Rc<TerminalSymbol>) -> Self {
        return Self {
            position,
            text: String::new(),
            terminal_symbol,
            is_eof: true,
            symbol_id: 0
        }
    }

}


#[derive(Debug, Clone)]
pub struct TokenPosition {
    pub start_position: usize,
    pub text_length: usize,
    pub line: usize,
    pub column: usize
}


impl TokenPosition {

    pub fn new(start_position: usize, text_length: usize, line: usize, column: usize) -> Self {
        return Self {
            start_position,
            text_length,
            line,
            column
        }
    }

    pub fn marge_start_position() -> Self {
        return Self {
            start_position: usize::MAX,
            text_length: 0,
            line: usize::MAX,
            column: usize::MAX
        }
    }

    pub fn marge(&mut self, other_token_position: &TokenPosition) {
        if other_token_position.text_length != 0 {
            self.text_length = if self.start_position == usize::MAX {
                other_token_position.text_length
            } else if other_token_position.start_position == usize::MAX {
                self.text_length
            } else {
                if self.start_position < other_token_position.start_position {
                    (other_token_position.start_position - self.start_position) + other_token_position.text_length
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


