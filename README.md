<div align="center">
<h1>bnf_rules!</h1>
<p>LR(1) parser generator</p>
</div>

### Generate LR(1) parser at compilation time.

```rust
use bnf_rules::bnf_rules_macro::bnf_rules;

// Grammar
bnf_rules!(
    source   ::= expr
    expr     ::= factor { "+" factor }
    factor   ::= "-" primary | primary
    primary  ::= "(" expr ")" | number
    number   ::= r"\d+" // regex
);

pub fn parse() {

    // A function named "parse_source" is automatically generated.
    let ast_node: Result<ASTNode, ParseError> = parse_source("(100 + 200) + -100");
    dbg!(ast_node.unwrap());

}
```

### Usage
```toml
bnf_rules = "0.1.7"
```

### Extended BNF
|           Form           |                  Semantic                  |
|:------------------------:|:------------------------------------------:|
|          source          |          An entire input source.           |
|          ident           |    A non-terminal symbol named "ident".    |
|       "something"        |        A terminal symbol for text.         |
|          r"\d+"          |       A terminal symbol for regex.         |
|    fn (function_name)    | A custom tokenizer with user function.[^1] |
|       { pattern }        |   Zero or more repetitions of "pattern".   |
|      \[ pattern \]       |             "pattern" or null.             |
| pattern1 &#124; pattern2 |         "pattern1" or "pattern2".          |
|       ( patterns )       |            A group of patterns.            |

[^1]: Generic parameters are also available.

> Example 1: https://github.com/bea4dev/bnf_rules/blob/master/src/lib.rs

> Example 2: https://github.com/bea4dev/catla/blob/master/catla_parser/src/grammar.rs
