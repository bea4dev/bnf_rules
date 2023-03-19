use proc_macro::TokenStream;
use bnf_rules_parser::{parse_rules, ParserGenerator, TokenParser};
use syn::parse_macro_input;

#[proc_macro]
pub fn bnf_rules(input: TokenStream) -> TokenStream {
    let token_parser = parse_macro_input!(input as TokenParser);
    let tokens = &token_parser.symbols;

    let map = parse_rules(tokens).unwrap();

    let mut generator = ParserGenerator::new(map);
    return generator.generate().unwrap().parse().unwrap();
}