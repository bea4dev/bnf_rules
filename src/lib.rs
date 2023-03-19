
pub extern crate bnf_rules_macro;
pub extern crate bnf_rules_parser;


#[cfg(test)]
mod tests {
    use bnf_rules_macro::bnf_rules;
    use bnf_rules_parser::lexer::{*};
    use bnf_rules_parser::parser::{*};

    bnf_rules!(
        source   ::= expr
        expr     ::= factor { "+" factor }
        factor   ::= "-" primary | primary
        primary  ::= "(" expr ")" | number
        number   ::= r"\d+"
    );


    #[test]
    pub fn test() {
        let node = parse_source(r#"10 + 100 + (200 + -10)"#).unwrap();
        dbg!(node);
    }

}