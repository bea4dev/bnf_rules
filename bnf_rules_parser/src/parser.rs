use either::Either;
use crate::{OPERATION_NONE, OPERATION_SHIFT, OPERATION_GOTO, OPERATION_REDUCE, OPERATION_ACCEPT};
use crate::lexer::{TokenPosition, Token};


#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ASTNode {
    NonTerminal {
        internal_symbol_id: u32,
        symbol_name: String,
        children: Vec<Result<ASTNode, Token>>,
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


#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct IncompleteAST {
    pub node: Result<ASTNode, Vec<Either<Token, ASTNode>>>,
    pub failed_to_recover: bool
}



pub fn __parse(
    tokens: Result<Vec<Token>, Vec<Token>>,
    rule_pattern_name: &[&str],
    lr_table: &[&[(usize, usize)]],
    bnf_rules: &[(u32, &[u32])],
    first_sets: &[(u32, &[u32])],
    non_terminal_symbols: &[&[usize]]
) -> Result<ASTNode, IncompleteAST> {

    let mut has_error = false;
    let mut success_to_recover = true;

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
    let mut reduce_buffer = Vec::<Either<Token, ASTNode>>::new();

    stack.push(0);

    loop {
        let stack_last = stack.last().unwrap().clone();
        let symbol_id = tokens.last().unwrap().symbol_id;
        if symbol_id == u32::MAX - 1 {
            // unexpected character
            success_to_recover &= recover_error(&mut stack, &mut tokens, &mut reduce_buffer, rule_pattern_name, first_sets, non_terminal_symbols);
            continue;
        }

        let operation = &lr_table[stack_last][symbol_id as usize];

        if operation.0 == OPERATION_NONE {
            success_to_recover &= recover_error(&mut stack, &mut tokens, &mut reduce_buffer, rule_pattern_name, first_sets, non_terminal_symbols);
            continue;
        }

        let operation_argument = operation.1;

        match operation.0 {
            OPERATION_SHIFT => {
                if tokens.last().is_none() {
                    // fatal error
                    has_error = true;
                    success_to_recover = false;
                    break;
                }

                let popped_token = tokens.pop().unwrap();
                reduce_buffer.push(Either::Left(popped_token));

                stack.push(operation_argument);
            },
            OPERATION_REDUCE => {
                let reduce_rule_id = operation_argument;
                let rule = &bnf_rules[reduce_rule_id];
                let rule_pattern = rule.1;
                let right_side_count = rule_pattern.len();

                if reduce_buffer.len() < right_side_count {
                    // fatal error
                    has_error = true;
                    success_to_recover = false;
                    break;
                }

                let mut buffer = Vec::<Either<Token, ASTNode>>::new();
                for _ in 0..right_side_count {
                    buffer.push(reduce_buffer.pop().unwrap());

                    if stack.pop().is_none() {
                        // fatal error
                        has_error = true;
                        success_to_recover = false;
                        break;
                    }
                }
                buffer.reverse();


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

                let mut reduce_children = Vec::<Result<ASTNode, Token>>::new();

                let mut i = 0;
                for token_or_node in buffer {
                    let symbol_id = get_token_or_node_symbol_id(&token_or_node);
                    let pattern_symbol_id = rule_pattern[i];

                    if symbol_id != pattern_symbol_id {
                        // fatal error
                        has_error = true;
                        success_to_recover = false;
                        break;
                    }

                    match token_or_node {
                        Either::Left(token) => {
                            let node = ASTNode::Terminal { internal_symbol_id: symbol_id, text: token.text.clone(), position: token.position.clone() };
                            position.marge(&node.get_position());
                            reduce_children.push(Ok(node));
                        },
                        Either::Right(node) => {
                            position.marge(&node.get_position());

                            match node {
                                ASTNode::NonTerminal { internal_symbol_id, symbol_name, children, position } => {
                                    if symbol_name.starts_with(" ") {
                                        for child in children.iter() {
                                            reduce_children.push(child.clone());
                                        }
                                    } else {
                                        reduce_children.push(Ok(ASTNode::NonTerminal {
                                            internal_symbol_id,
                                            symbol_name,
                                            children,
                                            position,
                                        }));
                                    }
                                },
                                ASTNode::Terminal { internal_symbol_id, text, position } => {
                                    reduce_children.push(Ok(ASTNode::Terminal {
                                        internal_symbol_id,
                                        text,
                                        position,
                                    }));
                                }
                            }
                        },
                    };

                    i += 1;
                }

                let rule_root_symbol_id = rule.0;
                let rule_name = rule_pattern_name[reduce_rule_id].to_string();

                let node = ASTNode::NonTerminal { internal_symbol_id: rule_root_symbol_id, symbol_name: rule_name, children: reduce_children, position };

                reduce_buffer.push(Either::Right(node));

                let stack_last = match stack.last() {
                    None => {
                        // fatal error
                        has_error = true;
                        success_to_recover = false;
                        break;
                    }
                    Some(last) => *last,
                };

                let operation = &lr_table[stack_last][rule_root_symbol_id as usize];

                if operation.0 != OPERATION_GOTO {
                    // fatal error
                    has_error = true;
                    success_to_recover = false;
                    break;
                }

                stack.push(operation.1);
            },
            OPERATION_ACCEPT => break,
            _ => {}
        }
    }

    if reduce_buffer.len() != 1 {
        has_error = true;
        success_to_recover = false;
    }
    let node = if success_to_recover {
        Ok(reduce_buffer.remove(0).right().unwrap())
    } else {
        Err(reduce_buffer)
    };

    return if has_error {
        Err(IncompleteAST {
            node,
            failed_to_recover: !success_to_recover,
        })
    } else {
        Ok(node.unwrap())
    };
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

        let mut popped_tokens = Vec::<Result<ASTNode, Token>>::new();
        loop {
            match tokens.last() {
                Some(token) => {
                    println!("token {} {}", &token.text, token.symbol_id);
                    if first_symbols.contains(&token.symbol_id) {
                        break;
                    }
                },
                _ => {
                    println!("{}", rule_pattern_name[rule_id]);
                    println!("{:?}", first_symbols);
                    unreachable!()
                }
            };

            let popped_token = tokens.pop().unwrap();
            popped_tokens.push(Err(popped_token));
        }

        let position = if popped_tokens.is_empty() {
            tokens.last().unwrap().position.clone()
        } else {
            let mut position = TokenPosition::marge_start_position();
            for token in tokens.iter() {
                position.marge(&token.position);
            }
            position
        };

        let node = ASTNode::NonTerminal {
            internal_symbol_id: non_terminal_symbol_id as u32,
            symbol_name: rule_pattern_name[rule_id].to_string(),
            children: popped_tokens,
            position,
        };

        reduce_buffer.push(Either::Right(node));
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