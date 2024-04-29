use proc_macro::TokenStream;
use bnf_rules_parser::{parse_rules, ParserGenerator, TokenParser};
use syn::parse_macro_input;

/// Generate LR(1) parser at compilation time.<br>
/// If the specified grammar is ambiguous, compilation is aborted with conflict.
///
/// # Examples
///
/// ``` 
/// // Grammar 
/// bnf_rules!(
///     source   ::= expr
///     expr     ::= factor { "+" factor }
///     factor   ::= "-" primary | primary
///     primary  ::= "(" expr ")" | number
///     number   ::= fn (number_tokenizer) // custom tokenizer with function
/// );
/// 
/// /// Custom tokenizer for numeric literal
/// fn number_tokenizer(source: &Vec<char>, mut current_position: usize) -> usize {
///     let mut iteration_count = 0;
///     loop {
///         let current_char = match source.get(current_position) {
///             Some(ch) => ch.clone(),
///             _ => break
///         };
///         if !current_char.is_numeric() {
///             break;
///         }
///         iteration_count += 1;
///         current_position += 1;
///     }
///     return iteration_count; // 0 means 'rejected', other means 'accepted' and 'length of token'.
/// }
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

    let map = parse_rules(tokens).unwrap();

    let mut generator = ParserGenerator::new(map);
    return generator.generate().unwrap().parse().unwrap();
}