<div align="center">
<h1>bnf_rules!</h1>
<p>LR(1) parser generator</p>
</div>

### Generate LR(1) parser at compilation time.

```rust
use bnf_rules::bnf_rules_macro::bnf_rules;
use bnf_rules::bnf_rules_parser::{*};
use bnf_rules::bnf_rules_parser::lexer::{*};
use bnf_rules::bnf_rules_parser::parser::{*};


bnf_rules!(
    source   ::= expr
    expr     ::= factor { "+" factor }
    factor   ::= "-" primary | primary
    primary  ::= "(" expr ")" | number
    number   ::= fn (number_tokenizer) // custom tokenizer with function
);

/// Custom tokenizer for numeric literal
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

pub fn parse() {

    let ast_node: Result<ASTNode, ParseError> = parse_source("(100 + 200) + -100");
    dbg!(ast_node.unwrap());

}
```

### Usage
```toml
bnf_rules = { git = "https://github.com/bea4dev/bnf_rules" }
```

### Extended BNF
|           Form           |                  Semantic                  |
|:------------------------:|:------------------------------------------:|
|          source          |          An entire input source.           |
|          ident           |    A non-terminal symbol named "ident".    |
|       "something"        |        A terminal symbol for text.         |
|    fn (function_name)    | A custom tokenizer with user function.[^1] |
|       { pattern }        |   Zero or more repetitions of "pattern".   |
|      \[ pattern \]       |             "pattern" or null.             |
| pattern1 &#124; pattern2 |         "pattern1" or "pattern2".          |
|       ( patterns )       |            A group of patterns.            |

[^1]: Generic parameters are also available.

> Example: https://github.com/bea4dev/bnf_rules/blob/master/src/lib.rs