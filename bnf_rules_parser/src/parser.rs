use either::Either;
use crate::{OPERATION_NONE, OPERATION_SHIFT, OPERATION_GOTO, OPERATION_REDUCE, OPERATION_ACCEPT};
use crate::lexer::{TokenPosition, Token};


#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ASTNode {
    NonTerminal {
        internal_symbol_id: u32,
        symbol_name: String,
        children: Result<Vec<ASTNode>, Vec<Token>>,
        position: TokenPosition
    },
    Terminal {
        internal_symbol_id: u32,
        text: String,
        position: TokenPosition
    }
}


impl ASTNode {

    fn get_symbol_id(&self) -> u32 {
        return match self {
            ASTNode::NonTerminal { internal_symbol_id: symbol_id, symbol_name: _, children: _, position: _ } => symbol_id.clone(),
            ASTNode::Terminal { internal_symbol_id: symbol_id, text: _, position: _ } => symbol_id.clone()
        }
    }

    pub fn get_position(&self) -> TokenPosition {
        return match self {
            ASTNode::NonTerminal { internal_symbol_id: _, symbol_name: _, children: _, position } => position.clone(),
            ASTNode::Terminal { internal_symbol_id: _, text: _, position } => position.clone()
        }
    }

}




#[derive(Debug, Eq, Clone, Hash, PartialEq)]
pub enum Symbol {
    String(String),
    Regex(String)
}


#[derive(Debug, Clone)]
pub struct ParseError {
    pub position: Option<TokenPosition>,
    pub message: String,
    pub error_type: ParseErrorType
}


impl ParseError {
    pub fn new(token: Option<Token>, message: String, error_type: ParseErrorType) -> Self {
        let position = match token {
            Some(token) => Some(token.position),
            _ => None
        };
        return Self {
            position,
            message,
            error_type
        }
    }

    pub fn new_from_position(position: Option<TokenPosition>, message: String, error_type: ParseErrorType) -> Self {
        return Self {
            position,
            message,
            error_type
        }
    }

}


#[derive(Debug, Clone)]
pub enum ParseErrorType {
    InvalidSyntax,
    UnexpectedToken,
    InternalError
}



pub fn __parse(
    tokens: Result<Vec<Token>, Vec<Token>>,
    rule_pattern_name: &[&str],
    lr_table: &[&[(usize, usize)]],
    bnf_rules: &[(u32, &[u32])],
    first_sets: &[(u32, &[u32])],
    non_terminal_symbols: &[&[usize]]
) -> Result<ASTNode, ASTNode> {

    let mut has_error = false;

    let mut tokens = match tokens {
        Ok(tokens) => tokens,
        Err(tokens) => {
            has_error = true;
            tokens
        }
    };

    let last_token_position = match tokens.last() {
        Some(token) => token.position.clone(),
        _ => TokenPosition::new(0, 0, 0, 0)
    };

    tokens.reverse();

    let mut stack = Vec::<usize>::new();
    let mut reduced_buffer = Vec::<Either<Token, ASTNode>>::new();

    stack.push(0);

    loop {
        let stack_last = stack.last().unwrap().clone();
        let symbol_id = tokens.last().unwrap().symbol_id;
        if symbol_id == u32::MAX - 1 {
            // unexpected character
            has_error |= recover_error(&mut stack, &mut tokens, bnf_rules, first_sets);
            continue;
        }

        let operation = &lr_table[stack_last][symbol_id as usize];

        if operation.0 == OPERATION_NONE {
            let position = match tokens.last() {
                Some(token) => Some(token.position.clone()),
                _ => None
            };
            return Err(ParseError::new_from_position(position, "Invalid operation.".to_string(), ParseErrorType::InvalidSyntax))
        }

        let operation_argument = operation.1;

        match operation.0 {
            OPERATION_SHIFT => {
                let popped_token = pop_token(&mut tokens)?;
                reduced_buffer.push(Either::Left(popped_token));

                stack.push(operation_argument);
            },
            OPERATION_REDUCE => {
                let reduce_rule_id = operation_argument;
                let rule = &bnf_rules[reduce_rule_id];
                let rule_pattern = rule.1;
                let right_side_count = rule_pattern.len();

                if reduced_buffer.len() < right_side_count {
                    return Err(ParseError::new_from_position(get_buffer_position(&reduced_buffer), "Invalid syntax.".to_string(), ParseErrorType::InvalidSyntax));
                }

                let mut buffer = Vec::<Either<Token, ASTNode>>::new();
                for _ in 0..right_side_count {
                    buffer.push(reduced_buffer.pop().unwrap());

                    if stack.pop().is_none() {
                        return Err(ParseError::new_from_position(get_buffer_position(&buffer), "Invalid syntax.".to_string(), ParseErrorType::InvalidSyntax));
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
                        },
                        _ => last_token_position.clone()
                    }
                } else {
                    TokenPosition::marge_start_position()
                };

                for i in 0..buffer.len() {
                    let token_or_node = &buffer[i];
                    let symbol_id = get_token_or_node_symbol_id(token_or_node);
                    let pattern_symbol_id = rule_pattern[i];

                    if symbol_id != pattern_symbol_id {
                        return Err(ParseError::new_from_position(get_token_or_node_position(Some(&token_or_node)), "Invalid syntax.".to_string(), ParseErrorType::InvalidSyntax))
                    }

                    match token_or_node {
                        Either::Left(token) => {
                            let node = ASTNode::Terminal { internal_symbol_id: Some(symbol_id), text: token.text.clone(), position: token.position.clone() };
                            position.marge(&node.get_position());
                            reduce_children.push(node);
                        },
                        Either::Right(node) => {
                            position.marge(&node.get_position());

                            match node {
                                ASTNode::NonTerminal { internal_symbol_id: _, symbol_name, children, position: _ } => {
                                    if symbol_name.starts_with(" ") {
                                        for child in children.iter() {
                                            reduce_children.push(child.clone());
                                        }
                                    } else {
                                        reduce_children.push(node.clone());
                                    }
                                },
                                ASTNode::Terminal { internal_symbol_id: _, text: _, position: _ } => {
                                    reduce_children.push(node.clone());
                                }
                            }
                        },
                    };
                }

                let rule_root_symbol_id = rule.0;
                let rule_name = rule_pattern_name[reduce_rule_id].to_string();

                let node = ASTNode::NonTerminal { internal_symbol_id: rule_root_symbol_id, symbol_name: rule_name, children: reduce_children, position };

                reduced_buffer.push(Either::Right(node));


                let stack_last = get_stack_last(&stack, &tokens)?;

                let operation = &lr_table[stack_last][rule_root_symbol_id as usize];

                if operation.0 != OPERATION_GOTO {
                    return Err(ParseError::new_from_position(get_buffer_position(&reduced_buffer), "Invalid operation.".to_string(), ParseErrorType::InvalidSyntax))
                }

                stack.push(operation.1);
            },
            OPERATION_ACCEPT => break,
            _ => {}
        }
    }

    if reduced_buffer.len() != 1 {
        return Err(ParseError::new_from_position(None, "May be internal error. reduce_buffer.len() is not 1.".to_string(), ParseErrorType::InternalError));
    }

    let node = reduced_buffer.remove(0).right().unwrap();

    return Ok(node);
}


fn recover_error(
    stack: &mut Vec<usize>,
    tokens: &mut Vec<Token>,
    reduce_buffer: &mut Vec<Either<Token, ASTNode>>,
    rule_pattern_name: &[&str],
    first_sets: &[(u32, &[u32])],
    non_terminal_symbols: &[&[usize]]
) -> bool {
    loop {
        let stack_top = match stack.last() {
            None => return false,
            Some(top) => *top
        };

        let non_terminal_symbols = non_terminal_symbols[stack_top];
        let non_terminal_symbol_id = match non_terminal_symbols.last() {
            Some(symbol_id) => *symbol_id,
            _ => {
                stack.pop();
                continue;
            }
        };

        let mut first_symbols = [].as_slice();
        let mut rule_id = 0;
        for first_sets_entry in first_sets {
            let root_symbol_id = first_sets_entry.0;
            if root_symbol_id == non_terminal_symbol_id as u32 {
                first_symbols = first_sets_entry.1;
                break;
            }
            rule_id += 1;
        }

        if first_symbols.is_empty() {
            unreachable!();
        }

        let mut popped_tokens = Vec::<Token>::new();
        loop {
            match tokens.last() {
                Some(token) => {
                    if first_symbols.contains(&token.symbol_id) {
                        break;
                    }
                },
                _ => unreachable!()
            };

            let popped_token = tokens.pop().unwrap();
            popped_tokens.push(popped_token);
        }

        let node = ASTNode::NonTerminal {
            internal_symbol_id: non_terminal_symbol_id as u32,
            symbol_name: rule_pattern_name[rule_id].to_string(),
            children: Err(popped_tokens),
            position: TokenPosition {},
        };
    }
}


fn get_token_or_node_symbol_id(token_or_node: &Either<Token, ASTNode>) -> u32 {
    return match token_or_node {
        Either::Left(token) => token.symbol_id,
        Either::Right(node) => node.get_symbol_id()
    };
}


fn get_token_or_node_position(token_or_node: Option<&Either<Token, ASTNode>>) -> Option<TokenPosition> {
    let token_or_node = token_or_node?;
    return match token_or_node {
        Either::Left(token) => Some(token.position.clone()),
        Either::Right(node) => Some(node.get_position())
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