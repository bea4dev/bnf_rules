<div align="center">
<h1>bnf_rules!</h1>
<p>LR(1) parser generator</p>
</div>

### Generate LR(1) parser at compile time.

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
    number   ::= r"\d+"
);

pub fn parse() {

    let ast_node: Result<ASTNode, ParseError> = parse_source("(100 + 200) + -100");
    dbg!(ast_node.unwrap());

}


```


### Extended BNF
|        Form        |                Semantic                |
|:------------------:|:--------------------------------------:|
|       source       |        An entire input source.         |
|     some_ident     |       Non-terminal symbol name.        |
|        "+"         |       Terminal symbol for text.        |
|       r"\d+"       |       Terminal symbol for regex.       |
|    { pattern }     | Zero or more repetitions of "pattern". |
|   \[ pattern \]    |           "pattern" or null.           |
| patt1 &#124; patt2 |          "patt1" or "patt2".           |