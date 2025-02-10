use bnf_rules_parser::{parse_rules, ParserGenerator, TokenParser};
use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Generate LR(1) parser at compilation time.<br>
/// If the specified grammar is ambiguous, compilation is aborted with conflict.
///
/// # Examples
///
/// ```
/// use bnf_rules::bnf_rules_macro::bnf_rules;
///
/// // Grammar
/// bnf_rules!(
///     // If it specified false, it will only check whether the grammar contains ambiguity, with no generated code.
///     // This setting is optional.
///     #[generate_code = true]
///
///     source   ::= expr
///     expr     ::= factor { "+" factor }
///     factor   ::= "-" primary | primary
///     primary  ::= "(" expr ")" | number
///     number   ::= r"\d+" // regex
/// );
///
/// pub fn parse() {
///
///     // A function named "parse_source" is automatically generated.
///     let ast_node: Result<ASTNode, ParseError> = parse_source("(100 + 200) + -100");
///     dbg!(ast_node.unwrap());
///
/// }
/// ```
#[proc_macro]
pub fn bnf_rules(input: TokenStream) -> TokenStream {
    let token_parser = parse_macro_input!(input as TokenParser);
    let tokens = &token_parser.symbols;

    let (map, generate_code) = parse_rules(tokens).unwrap();

    let mut generator = ParserGenerator::new(map);
    return generator.generate(generate_code).unwrap().parse().unwrap();
}
