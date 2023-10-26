
pub extern crate bnf_rules_macro;
pub extern crate bnf_rules_parser;


#[cfg(test)]
mod tests {
    use bnf_rules_macro::bnf_rules;
    use bnf_rules_parser::lexer::{*};
    use bnf_rules_parser::parser::{*};
    use bnf_rules_parser::parser::ASTNode::{NonTerminal, Terminal};

    bnf_rules!(
        source   ::= [ expr ]
        expr     ::= factor { "+" factor }
        factor   ::= "-" primary | primary
        primary  ::= "(" expr ")" | number
        number   ::= fn (number_tokenizer) // custom tokenizer with function
    );

    fn number_tokenizer(source: &Vec<char>, mut current_position: usize) -> usize {
        let mut iteration_count = 0;
        loop {
            let current_char = match source.get(current_position) {
                Some(ch) => ch.clone(),
                _ => break
            };
            if !current_char.is_numeric() {
                break;
            }
            iteration_count += 1;
            current_position += 1;
        }
        return iteration_count; // 0 means 'rejected', other means 'accepted' and 'length of token'.
    }


    #[test]
    pub fn test() {
        // A function named "parse_source" is automatically generated.
        let node: ASTNode = parse_source("10 + (200 + -10)").unwrap();
        dbg!(&node);

        assert_eq!(node, NonTerminal {
            internal_symbol_id: None,
            symbol_name: "source".to_string(),
            children: [
                NonTerminal {
                    internal_symbol_id: None,
                    symbol_name: "expr".to_string(),
                    children: [
                        NonTerminal {
                            internal_symbol_id: None,
                            symbol_name: "factor".to_string(),
                            children: [
                                NonTerminal {
                                    internal_symbol_id: None,
                                    symbol_name: "primary".to_string(),
                                    children: [
                                        NonTerminal {
                                            internal_symbol_id: None,
                                            symbol_name: "number".to_string(),
                                            children: [
                                                Terminal {
                                                    internal_symbol_id: None,
                                                    text: "10".to_string(),
                                                    position: TokenPosition {
                                                        start_position: 0,
                                                        text_length: 2,
                                                        line: 1,
                                                        column: 1,
                                                    },
                                                },
                                            ].to_vec(),
                                            position: TokenPosition {
                                                start_position: 0,
                                                text_length: 2,
                                                line: 1,
                                                column: 1,
                                            },
                                        },
                                    ].to_vec(),
                                    position: TokenPosition {
                                        start_position: 0,
                                        text_length: 2,
                                        line: 1,
                                        column: 1,
                                    },
                                },
                            ].to_vec(),
                            position: TokenPosition {
                                start_position: 0,
                                text_length: 2,
                                line: 1,
                                column: 1,
                            },
                        },
                        Terminal {
                            internal_symbol_id: None,
                            text: "+".to_string(),
                            position: TokenPosition {
                                start_position: 3,
                                text_length: 1,
                                line: 1,
                                column: 4,
                            },
                        },
                        NonTerminal {
                            internal_symbol_id: None,
                            symbol_name: "factor".to_string(),
                            children: [
                                NonTerminal {
                                    internal_symbol_id: None,
                                    symbol_name: "primary".to_string(),
                                    children: [
                                        Terminal {
                                            internal_symbol_id: None,
                                            text: "(".to_string(),
                                            position: TokenPosition {
                                                start_position: 5,
                                                text_length: 1,
                                                line: 1,
                                                column: 6,
                                            },
                                        },
                                        NonTerminal {
                                            internal_symbol_id: None,
                                            symbol_name: "expr".to_string(),
                                            children: [
                                                NonTerminal {
                                                    internal_symbol_id: None,
                                                    symbol_name: "factor".to_string(),
                                                    children: [
                                                        NonTerminal {
                                                            internal_symbol_id: None,
                                                            symbol_name: "primary".to_string(),
                                                            children: [
                                                                NonTerminal {
                                                                    internal_symbol_id: None,
                                                                    symbol_name: "number".to_string(),
                                                                    children: [
                                                                        Terminal {
                                                                            internal_symbol_id: None,
                                                                            text: "200".to_string(),
                                                                            position: TokenPosition {
                                                                                start_position: 6,
                                                                                text_length: 3,
                                                                                line: 1,
                                                                                column: 7,
                                                                            },
                                                                        },
                                                                    ].to_vec(),
                                                                    position: TokenPosition {
                                                                        start_position: 6,
                                                                        text_length: 3,
                                                                        line: 1,
                                                                        column: 7,
                                                                    },
                                                                },
                                                            ].to_vec(),
                                                            position: TokenPosition {
                                                                start_position: 6,
                                                                text_length: 3,
                                                                line: 1,
                                                                column: 7,
                                                            },
                                                        },
                                                    ].to_vec(),
                                                    position: TokenPosition {
                                                        start_position: 6,
                                                        text_length: 3,
                                                        line: 1,
                                                        column: 7,
                                                    },
                                                },
                                                Terminal {
                                                    internal_symbol_id: None,
                                                    text: "+".to_string(),
                                                    position: TokenPosition {
                                                        start_position: 10,
                                                        text_length: 1,
                                                        line: 1,
                                                        column: 11,
                                                    },
                                                },
                                                NonTerminal {
                                                    internal_symbol_id: None,
                                                    symbol_name: "factor".to_string(),
                                                    children: [
                                                        Terminal {
                                                            internal_symbol_id: None,
                                                            text: "-".to_string(),
                                                            position: TokenPosition {
                                                                start_position: 12,
                                                                text_length: 1,
                                                                line: 1,
                                                                column: 13,
                                                            },
                                                        },
                                                        NonTerminal {
                                                            internal_symbol_id: None,
                                                            symbol_name: "primary".to_string(),
                                                            children: [
                                                                NonTerminal {
                                                                    internal_symbol_id: None,
                                                                    symbol_name: "number".to_string(),
                                                                    children: [
                                                                        Terminal {
                                                                            internal_symbol_id: None,
                                                                            text: "10".to_string(),
                                                                            position: TokenPosition {
                                                                                start_position: 13,
                                                                                text_length: 2,
                                                                                line: 1,
                                                                                column: 14,
                                                                            },
                                                                        },
                                                                    ].to_vec(),
                                                                    position: TokenPosition {
                                                                        start_position: 13,
                                                                        text_length: 2,
                                                                        line: 1,
                                                                        column: 14,
                                                                    },
                                                                },
                                                            ].to_vec(),
                                                            position: TokenPosition {
                                                                start_position: 13,
                                                                text_length: 2,
                                                                line: 1,
                                                                column: 14,
                                                            },
                                                        },
                                                    ].to_vec(),
                                                    position: TokenPosition {
                                                        start_position: 12,
                                                        text_length: 3,
                                                        line: 1,
                                                        column: 13,
                                                    },
                                                },
                                            ].to_vec(),
                                            position: TokenPosition {
                                                start_position: 6,
                                                text_length: 9,
                                                line: 1,
                                                column: 7,
                                            },
                                        },
                                        Terminal {
                                            internal_symbol_id: None,
                                            text: ")".to_string(),
                                            position: TokenPosition {
                                                start_position: 15,
                                                text_length: 1,
                                                line: 1,
                                                column: 16,
                                            },
                                        },
                                    ].to_vec(),
                                    position: TokenPosition {
                                        start_position: 5,
                                        text_length: 11,
                                        line: 1,
                                        column: 6,
                                    },
                                },
                            ].to_vec(),
                            position: TokenPosition {
                                start_position: 5,
                                text_length: 11,
                                line: 1,
                                column: 6,
                            },
                        },
                    ].to_vec(),
                    position: TokenPosition {
                        start_position: 0,
                        text_length: 16,
                        line: 1,
                        column: 1,
                    },
                },
            ].to_vec(),
            position: TokenPosition {
                start_position: 0,
                text_length: 16,
                line: 1,
                column: 1,
            },
        });
    }

}