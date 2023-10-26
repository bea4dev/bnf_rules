
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
        let node: ASTNode = parse_source("10 + (200 + -10)").unwrap();
        dbg!(&node);
    }

}