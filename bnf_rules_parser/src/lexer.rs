use std::cmp::min;
use std::rc::Rc;
use either::Either;
use fixedbitset::FixedBitSet;
use regex::{Error, Regex};



pub struct Lexer {
    terminal_symbols: Vec<Rc<TerminalSymbol>>,
    eof_symbol: Rc<TerminalSymbol>
}



impl Lexer {

    pub fn new(mut terminal_symbols: Vec<TerminalSymbol>) -> Self {
        terminal_symbols.push(TerminalSymbol::new_from_regex(r"\s+", usize::MAX).unwrap());

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

        let mut tokens = Vec::<Token>::new();
        let mut word_buffer = String::new();
        let mut word_buffer_start_position = 0;

        let terminal_symbol_length = self.terminal_symbols.len();
        let mut enabled_symbol_flags = FixedBitSet::with_capacity(terminal_symbol_length);
        for i in 0..terminal_symbol_length {
            enabled_symbol_flags.set(i, true);
        }

        for source_index in 0..source_length {
            let char = source[source_index];
            word_buffer.push(char);

            for i in 0..terminal_symbol_length {

                if enabled_symbol_flags[i] {

                    let terminal_symbol = &self.terminal_symbols[i];

                    if !terminal_symbol.is_match(word_buffer.as_str()) {
                        enabled_symbol_flags.set(i, false);

                        if enabled_symbol_flags.is_clear() {
                            let last_char = word_buffer.pop();

                            let word_buffer_str = word_buffer.as_str();

                            let mut terminal_symbol = terminal_symbol;
                            if !terminal_symbol.is_match_strict(word_buffer_str) {
                                for symbol in self.terminal_symbols.iter() {
                                    if symbol.is_match_strict(word_buffer_str) {
                                        terminal_symbol = symbol;
                                        break;
                                    }
                                }
                            }

                            let word_length = source_index - word_buffer_start_position;

                            if word_length == 0 {
                                let position = TokenPosition::new(word_buffer_start_position, 1, 0, 0);
                                return Err(UnexpectedToken {
                                    position,
                                    text: last_char.unwrap().to_string()
                                });
                            } else if !terminal_symbol.is_match(word_buffer_str) {
                                let position = TokenPosition::new(word_buffer_start_position, word_length, 0, 0);
                                return Err(UnexpectedToken {
                                    position,
                                    text: word_buffer.clone()
                                });
                            }

                            if terminal_symbol.symbol_id != usize::MAX {
                                let position = TokenPosition::new(word_buffer_start_position, word_length, 0, 0);
                                let token = Token::new(
                                    position,
                                    word_buffer.clone(),
                                    terminal_symbol.clone(), terminal_symbol.symbol_id);

                                tokens.push(token);
                            }


                            word_buffer = String::new();
                            match last_char { Some(char) => word_buffer.push(char), _ => {} }

                            word_buffer_start_position = source_index;



                            let mut found = false;
                            for i in 0..terminal_symbol_length {

                                let symbol = &self.terminal_symbols[i];

                                if symbol.is_match(word_buffer.as_str()) {
                                    enabled_symbol_flags.set(i, true);
                                    found = true;
                                }

                            }

                            if !found {
                                let position = TokenPosition::new(word_buffer_start_position, word_length, 0, 0);
                                return Err(UnexpectedToken {
                                    position,
                                    text: word_buffer.clone()
                                });
                            }
                        }
                    }

                }

            }


            if source_index + 1 == source_length {

                let word_length = word_buffer.chars().count();
                let eof_symbol = &self.eof_symbol;

                if word_length == 0 {
                    let position = TokenPosition::new(source_index, 0, 0, 0);
                    tokens.push(Token::new_eof(position, eof_symbol.clone()));
                } else {
                    let mut last_matched_symbol = eof_symbol;
                    let mut found = false;

                    for i in 0..terminal_symbol_length {
                        if enabled_symbol_flags[i] {

                            let terminal_symbol = &self.terminal_symbols[i];

                            if terminal_symbol.is_match(word_buffer.as_str()) {
                                last_matched_symbol = &self.terminal_symbols[i];
                                found = true;
                            }
                        }
                    }

                    if found {
                        if last_matched_symbol.symbol_id != usize::MAX {
                            let position = TokenPosition::new(word_buffer_start_position, word_length, 0, 0);
                            tokens.push(Token::new(position, word_buffer.clone(), last_matched_symbol.clone(), last_matched_symbol.symbol_id));
                        }
                    } else {
                        let position = TokenPosition::new(word_buffer_start_position, word_length, 0, 0);
                        return Err(UnexpectedToken {
                            position,
                            text: word_buffer.clone()
                        });
                    }

                    let position = TokenPosition::new(source_index, 0, 0, 0);
                    tokens.push(Token::new_eof(position, eof_symbol.clone()));
                }

            }

        }

        return Ok(tokens);
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



#[derive(Debug)]
pub struct NonterminalSymbol {
    name: String
}


impl NonterminalSymbol {

    pub fn new(name: String) -> Self {
        return Self {
            name
        }
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


