use crate::lexer::{Token, TokenPosition, UnexpectedCharacter};
use crate::{OPERATION_ACCEPT, OPERATION_GOTO, OPERATION_NONE, OPERATION_REDUCE, OPERATION_SHIFT};
use either::Either;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ASTNode {
    NonTerminal {
        internal_symbol_id: Option<u32>,
        symbol_name: String,
        children: Vec<ASTNode>,
        position: TokenPosition,
    },
    Terminal {
        internal_symbol_id: Option<u32>,
        text: String,
        position: TokenPosition,
    },
}

impl ASTNode {
    fn get_symbol_id(&self) -> u32 {
        return match self {
            ASTNode::NonTerminal {
                internal_symbol_id: symbol_id,
                symbol_name: _,
                children: _,
                position: _,
            } => symbol_id.unwrap().clone(),
            ASTNode::Terminal {
                internal_symbol_id: symbol_id,
                text: _,
                position: _,
            } => symbol_id.unwrap().clone(),
        };
    }

    pub fn get_position(&self) -> TokenPosition {
        return match self {
            ASTNode::NonTerminal {
                internal_symbol_id: _,
                symbol_name: _,
                children: _,
                position,
            } => position.clone(),
            ASTNode::Terminal {
                internal_symbol_id: _,
                text: _,
                position,
            } => position.clone(),
        };
    }
}

#[derive(Debug, Eq, Clone, Hash, PartialEq)]
pub enum Symbol {
    String(String),
    Regex(String),
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub position: Option<TokenPosition>,
    pub message: String,
    pub error_type: ParseErrorType,
}

impl ParseError {
    pub fn new(token: Option<Token>, message: String, error_type: ParseErrorType) -> Self {
        let position = match token {
            Some(token) => Some(token.position),
            _ => None,
        };
        return Self {
            position,
            message,
            error_type,
        };
    }

    pub fn new_from_position(
        position: Option<TokenPosition>,
        message: String,
        error_type: ParseErrorType,
    ) -> Self {
        return Self {
            position,
            message,
            error_type,
        };
    }
}

#[derive(Debug, Clone)]
pub enum ParseErrorType {
    InvalidSyntax,
    UnexpectedToken,
    InternalError,
}

pub fn __parse(
    tokens: Result<Vec<Token>, UnexpectedCharacter>,
    rule_pattern_name: &[&str],
    lr_table: &[&[(usize, usize)]],
    bnf_rules: &[(u32, &[u32])],
) -> Result<ASTNode, ParseError> {
    let mut tokens = match tokens {
        Ok(tokens) => tokens,
        Err(err) => {
            return Err(ParseError::new_from_position(
                Some(err.position),
                "Unexpected token(Lexer).".to_string(),
                ParseErrorType::UnexpectedToken,
            ))
        }
    };

    let last_token_position = match tokens.last() {
        Some(token) => token.position.clone(),
        _ => TokenPosition::new(0, 0, 0, 0),
    };

    tokens.reverse();

    let mut stack = Vec::<usize>::new();
    let mut reduced_buffer = Vec::<Either<Token, ASTNode>>::new();

    stack.push(0);

    loop {
        let stack_last = get_stack_last(&stack, &tokens)?;
        let symbol_id = get_tokens_last_symbol_id(&tokens)?;
        let operation = &lr_table[stack_last][symbol_id as usize];

        if operation.0 == OPERATION_NONE {
            let position = match tokens.last() {
                Some(token) => Some(token.position.clone()),
                _ => None,
            };
            return Err(ParseError::new_from_position(
                position,
                "Invalid operation.".to_string(),
                ParseErrorType::InvalidSyntax,
            ));
        }

        let operation_argument = operation.1;

        match operation.0 {
            OPERATION_SHIFT => {
                let popped_token = pop_token(&mut tokens)?;
                reduced_buffer.push(Either::Left(popped_token));

                stack.push(operation_argument);
            }
            OPERATION_REDUCE => {
                let reduce_rule_id = operation_argument;
                let rule = &bnf_rules[reduce_rule_id];
                let rule_pattern = rule.1;
                let right_side_count = rule_pattern.len();

                if reduced_buffer.len() < right_side_count {
                    return Err(ParseError::new_from_position(
                        get_buffer_position(&reduced_buffer),
                        "Invalid syntax.".to_string(),
                        ParseErrorType::InvalidSyntax,
                    ));
                }

                let mut buffer = Vec::<Either<Token, ASTNode>>::new();
                for _ in 0..right_side_count {
                    buffer.push(reduced_buffer.pop().unwrap());

                    if stack.pop().is_none() {
                        return Err(ParseError::new_from_position(
                            get_buffer_position(&buffer),
                            "Invalid syntax.".to_string(),
                            ParseErrorType::InvalidSyntax,
                        ));
                    }
                }
                buffer.reverse();

                let mut reduce_children = Vec::<ASTNode>::new();

                let mut position = if buffer.len() == 0 {
                    match tokens.last() {
                        Some(token) => {
                            let mut position = token.position.clone();
                            position.text_length = 0;
                            position
                        }
                        _ => last_token_position.clone(),
                    }
                } else {
                    TokenPosition::marge_start_position()
                };

                for i in 0..buffer.len() {
                    let token_or_node = &buffer[i];
                    let symbol_id = get_token_or_node_symbol_id(token_or_node);
                    let pattern_symbol_id = rule_pattern[i];

                    if symbol_id != pattern_symbol_id {
                        return Err(ParseError::new_from_position(
                            get_token_or_node_position(Some(&token_or_node)),
                            "Invalid syntax.".to_string(),
                            ParseErrorType::InvalidSyntax,
                        ));
                    }

                    match token_or_node {
                        Either::Left(token) => {
                            let node = ASTNode::Terminal {
                                internal_symbol_id: Some(symbol_id),
                                text: token.text.clone(),
                                position: token.position.clone(),
                            };
                            position.marge(&node.get_position());
                            reduce_children.push(node);
                        }
                        Either::Right(node) => {
                            position.marge(&node.get_position());

                            match node {
                                ASTNode::NonTerminal {
                                    internal_symbol_id: _,
                                    symbol_name,
                                    children,
                                    position: _,
                                } => {
                                    if symbol_name.starts_with(" ") {
                                        for child in children.iter() {
                                            reduce_children.push(child.clone());
                                        }
                                    } else {
                                        reduce_children.push(node.clone());
                                    }
                                }
                                ASTNode::Terminal {
                                    internal_symbol_id: _,
                                    text: _,
                                    position: _,
                                } => {
                                    reduce_children.push(node.clone());
                                }
                            }
                        }
                    };
                }

                let rule_root_symbol_id = rule.0;
                let rule_name = rule_pattern_name[reduce_rule_id].to_string();

                let node = ASTNode::NonTerminal {
                    internal_symbol_id: Some(rule_root_symbol_id),
                    symbol_name: rule_name,
                    children: reduce_children,
                    position,
                };

                reduced_buffer.push(Either::Right(node));

                let stack_last = get_stack_last(&stack, &tokens)?;

                let operation = &lr_table[stack_last][rule_root_symbol_id as usize];

                if operation.0 != OPERATION_GOTO {
                    return Err(ParseError::new_from_position(
                        get_buffer_position(&reduced_buffer),
                        "Invalid operation.".to_string(),
                        ParseErrorType::InvalidSyntax,
                    ));
                }

                stack.push(operation.1);
            }
            OPERATION_ACCEPT => break,
            _ => {}
        }
    }

    if reduced_buffer.len() != 1 {
        return Err(ParseError::new_from_position(
            None,
            "May be internal error. reduce_buffer.len() is not 1.".to_string(),
            ParseErrorType::InternalError,
        ));
    }

    let mut node = reduced_buffer.remove(0).right().unwrap();
    unset_internal_symbol_id(&mut node);

    return Ok(node);
}

fn unset_internal_symbol_id(node: &mut ASTNode) {
    match node {
        ASTNode::NonTerminal {
            internal_symbol_id,
            symbol_name: _,
            children,
            position: _,
        } => {
            *internal_symbol_id = None;
            for child in children.iter_mut() {
                unset_internal_symbol_id(child);
            }
        }
        ASTNode::Terminal {
            internal_symbol_id,
            text: _,
            position: _,
        } => {
            *internal_symbol_id = None;
        }
    }
}

fn get_token_or_node_symbol_id(token_or_node: &Either<Token, ASTNode>) -> u32 {
    return match token_or_node {
        Either::Left(token) => token.symbol_id,
        Either::Right(node) => node.get_symbol_id(),
    };
}

fn get_token_or_node_position(
    token_or_node: Option<&Either<Token, ASTNode>>,
) -> Option<TokenPosition> {
    let token_or_node = token_or_node?;
    return match token_or_node {
        Either::Left(token) => Some(token.position.clone()),
        Either::Right(node) => Some(node.get_position()),
    };
}

fn get_buffer_position(buffer: &Vec<Either<Token, ASTNode>>) -> Option<TokenPosition> {
    if buffer.is_empty() {
        return None;
    }

    let mut merged_position = TokenPosition::marge_start_position();
    for token_or_node in buffer.iter() {
        let position = get_token_or_node_position(Some(token_or_node))?;
        merged_position.marge(&position);
    }

    return Some(merged_position);
}

fn get_stack_last(stack: &Vec<usize>, tokens: &Vec<Token>) -> Result<usize, ParseError> {
    return match stack.last() {
        Some(last) => Ok(*last),
        _ => {
            return Err(ParseError::new(
                tokens.first().cloned(),
                "Elements of the parser stack are missing.".to_string(),
                ParseErrorType::InvalidSyntax,
            ))
        }
    };
}

fn get_tokens_last(tokens: &Vec<Token>) -> Result<&Token, ParseError> {
    return match tokens.last() {
        Some(last) => Ok(last),
        _ => {
            return Err(ParseError::new(
                None,
                "Elements of the tokens are missing.".to_string(),
                ParseErrorType::InvalidSyntax,
            ))
        }
    };
}

fn pop_token(tokens: &mut Vec<Token>) -> Result<Token, ParseError> {
    return match tokens.pop() {
        Some(last) => Ok(last),
        _ => {
            return Err(ParseError::new(
                None,
                "Elements of the tokens are missing.".to_string(),
                ParseErrorType::InvalidSyntax,
            ))
        }
    };
}

fn get_tokens_last_symbol_id(tokens: &Vec<Token>) -> Result<u32, ParseError> {
    let token = get_tokens_last(tokens)?;
    return Ok(token.symbol_id);
}

