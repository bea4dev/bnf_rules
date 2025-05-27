pub extern crate bnf_rules_macro;
pub extern crate bnf_rules_parser;

#[cfg(test)]
mod tests {
    mod test_expr {
        mod bnf_rules {
            pub extern crate bnf_rules_macro;
            pub extern crate bnf_rules_parser;
        }

        crate::bnf_rules_macro::bnf_rules!(
            #[generate_code = true]

            source   ::= [ expr ]
            expr     ::= factor { "+" factor }
            factor   ::= "-" primary | primary
            primary  ::= "(" expr ")" | number
            number   ::= r"\d+" // regex
        );

        #[test]
        fn test() {
            // A function named "parse_source" is automatically generated.
            let node: ASTNode = parse_source("10 + (200 + -10)").unwrap();
            dbg!(&node);

            assert_eq!(
                node,
                NonTerminal {
                    internal_symbol_id: None,
                    symbol_name: "source".to_string(),
                    children: [NonTerminal {
                        internal_symbol_id: None,
                        symbol_name: "expr".to_string(),
                        children: [
                            NonTerminal {
                                internal_symbol_id: None,
                                symbol_name: "factor".to_string(),
                                children: [NonTerminal {
                                    internal_symbol_id: None,
                                    symbol_name: "primary".to_string(),
                                    children: [NonTerminal {
                                        internal_symbol_id: None,
                                        symbol_name: "number".to_string(),
                                        children: [Terminal {
                                            internal_symbol_id: None,
                                            text: "10".to_string(),
                                            position: TokenPosition {
                                                start_position: 0,
                                                text_length: 2,
                                                line: 1,
                                                column: 1,
                                            },
                                        },]
                                        .to_vec(),
                                        position: TokenPosition {
                                            start_position: 0,
                                            text_length: 2,
                                            line: 1,
                                            column: 1,
                                        },
                                    },]
                                    .to_vec(),
                                    position: TokenPosition {
                                        start_position: 0,
                                        text_length: 2,
                                        line: 1,
                                        column: 1,
                                    },
                                },]
                                .to_vec(),
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
                                children: [NonTerminal {
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
                                                    children: [NonTerminal {
                                                        internal_symbol_id: None,
                                                        symbol_name: "primary".to_string(),
                                                        children: [NonTerminal {
                                                            internal_symbol_id: None,
                                                            symbol_name: "number".to_string(),
                                                            children: [Terminal {
                                                                internal_symbol_id: None,
                                                                text: "200".to_string(),
                                                                position: TokenPosition {
                                                                    start_position: 6,
                                                                    text_length: 3,
                                                                    line: 1,
                                                                    column: 7,
                                                                },
                                                            },]
                                                            .to_vec(),
                                                            position: TokenPosition {
                                                                start_position: 6,
                                                                text_length: 3,
                                                                line: 1,
                                                                column: 7,
                                                            },
                                                        },]
                                                        .to_vec(),
                                                        position: TokenPosition {
                                                            start_position: 6,
                                                            text_length: 3,
                                                            line: 1,
                                                            column: 7,
                                                        },
                                                    },]
                                                    .to_vec(),
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
                                                            children: [NonTerminal {
                                                                internal_symbol_id: None,
                                                                symbol_name: "number".to_string(),
                                                                children: [Terminal {
                                                                    internal_symbol_id: None,
                                                                    text: "10".to_string(),
                                                                    position: TokenPosition {
                                                                        start_position: 13,
                                                                        text_length: 2,
                                                                        line: 1,
                                                                        column: 14,
                                                                    },
                                                                },]
                                                                .to_vec(),
                                                                position: TokenPosition {
                                                                    start_position: 13,
                                                                    text_length: 2,
                                                                    line: 1,
                                                                    column: 14,
                                                                },
                                                            },]
                                                            .to_vec(),
                                                            position: TokenPosition {
                                                                start_position: 13,
                                                                text_length: 2,
                                                                line: 1,
                                                                column: 14,
                                                            },
                                                        },
                                                    ]
                                                    .to_vec(),
                                                    position: TokenPosition {
                                                        start_position: 12,
                                                        text_length: 3,
                                                        line: 1,
                                                        column: 13,
                                                    },
                                                },
                                            ]
                                            .to_vec(),
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
                                    ]
                                    .to_vec(),
                                    position: TokenPosition {
                                        start_position: 5,
                                        text_length: 11,
                                        line: 1,
                                        column: 6,
                                    },
                                },]
                                .to_vec(),
                                position: TokenPosition {
                                    start_position: 5,
                                    text_length: 11,
                                    line: 1,
                                    column: 6,
                                },
                            },
                        ]
                        .to_vec(),
                        position: TokenPosition {
                            start_position: 0,
                            text_length: 16,
                            line: 1,
                            column: 1,
                        },
                    },]
                    .to_vec(),
                    position: TokenPosition {
                        start_position: 0,
                        text_length: 16,
                        line: 1,
                        column: 1,
                    },
                }
            );
        }
    }

    mod test_quotes {
        mod bnf_rules {
            pub extern crate bnf_rules_macro;
            pub extern crate bnf_rules_parser;
        }

        crate::bnf_rules_macro::bnf_rules!(
            source       ::= quotedstring
            quotedstring ::= r#""[^"]*""#
        );

        #[test]
        fn test() {
            let node = parse_source(r#""Hello, world!""#).unwrap();
            dbg!(node);
        }
    }
}
