use std::collections::{HashMap, HashSet};
use std::mem;
use litrs::StringLit;
use proc_macro2::{Delimiter, TokenTree};
use syn::Error;
use syn::parse::{Parse, ParseStream};

pub mod lexer;
pub mod parser;


pub fn parse_rules(tokens: &Vec<TokenTree>) -> Result<HashMap<String, BNFRule>, Error> {

    let mut non_terminal_symbol_name = String::new();
    let mut buffered_tokens = Vec::<TokenTree>::new();
    let mut rule_map = HashMap::<String, BNFRule>::new();

    let mut i = 0;
    let mut token = &tokens[0];

    let mut non_duplicate_number = NonDuplicateNumber::new();
    let mut unnamed_pattern_map = HashMap::new();

    loop {
        buffered_tokens.push(token.clone());

        if i == 0 {
            buffered_tokens.pop();

            non_terminal_symbol_name = match token {
                TokenTree::Ident(ident) => ident.to_string(),
                _ => return Err(Error::new(token.span(), "Invalid non terminal symbol name."))
            };

            check_next_punct(&tokens, &mut i, ':')?;
            check_next_punct(&tokens, &mut i, ':')?;
            check_next_punct(&tokens, &mut i, '=')?;

            token = next(&tokens, &mut i)?;
        } else {
            match check_next_punct(&tokens, &mut i, ':') {
                Ok(_) => {
                    buffered_tokens.pop();

                    parse_rule(&mut rule_map, &mut non_terminal_symbol_name, &buffered_tokens, &mut non_duplicate_number, &mut unnamed_pattern_map)?;

                    buffered_tokens.clear();

                    non_terminal_symbol_name = match token {
                        TokenTree::Ident(ident) => ident.to_string(),
                        _ => return Err(Error::new(token.span(), "Invalid non terminal symbol name."))
                    };

                    check_next_punct(&tokens, &mut i, ':')?;
                    check_next_punct(&tokens, &mut i, '=')?;

                    token = next(&tokens, &mut i)?;
                },
                Err(_) => {
                    token = &tokens[i];
                }
            }
        }


        if i + 1 == tokens.len() {
            buffered_tokens.push(token.clone());
            parse_rule(&mut rule_map, &mut non_terminal_symbol_name, &buffered_tokens, &mut non_duplicate_number, &mut unnamed_pattern_map)?;
            break;
        }
    }

    return Ok(rule_map);
}



fn check_next_punct(tokens: &Vec<TokenTree>, i: &mut usize, char: char) -> Result<(), Error> {
    let token = next(tokens, i)?;

    return match token {
        TokenTree::Punct(punct) => {
            if punct.as_char() != char {
                Err(Error::new(punct.span(), "Invalid syntax."))
            } else {
                Ok(())
            }
        },
        _ => Err(Error::new(token.span(), "Invalid syntax."))
    };
}


fn next<'a>(tokens: &'a Vec<TokenTree>, i: &mut usize) -> Result<&'a TokenTree, Error> {
    *i += 1;
    if *i == tokens.len() {
        return Err(Error::new(tokens[tokens.len() - 1].span(), "Unexpected EOF."));
    }
    return Ok(&tokens[*i]);
}



fn parse_rule(rule_map: &mut HashMap<String, BNFRule>, non_terminal_symbol_name: &mut String, tokens: &Vec<TokenTree>, non_duplicate_number: &mut NonDuplicateNumber, unnamed_pattern_map: &mut HashMap<Vec<Vec<BNFSymbol>>, String>) -> Result<(), Error> {

    let mut rule = BNFRule::new(non_terminal_symbol_name.clone());

    let mut pattern = Vec::<BNFSymbol>::new();
    let or_patterns = &mut rule.or_patterns;

    let mut index = 0;
    loop {
        if index >= tokens.len() {
            break;
        }
        let token = &tokens[index];

        match token {
            TokenTree::Punct(punct) => {
                if punct.as_char() != '|' {
                    return Err(Error::new(punct.span(), "Invalid punctuation."));
                }
                if pattern.is_empty() {
                    pattern.push(BNFSymbol::Null);
                }

                let mut pattern_temp = Vec::new();
                mem::swap(&mut pattern_temp, &mut pattern);

                or_patterns.push(pattern_temp);
            },
            TokenTree::Ident(ident) => {
                if ident.to_string() == "fn" {
                    let next_index = index + 1;
                    if next_index >= tokens.len() {
                        return Err(Error::new(ident.span(), "A function must be specified."));
                    }
                    let next_token = &tokens[next_index];
                    if let TokenTree::Group(next_token) = next_token {
                        if next_token.delimiter() != Delimiter::Parenthesis {
                            return Err(Error::new(next_token.span(), "Invalid delimiter."));
                        }
                        let ident_chars = next_token.to_string().chars().collect::<Vec<char>>();
                        if ident_chars.len() <= 2 {
                            return Err(Error::new(next_token.span(), "A function must be specified."));
                        }
                        let func_string = ident_chars[1..(ident_chars.len() - 1)].iter().collect::<String>();
                        pattern.push(BNFSymbol::TerminalSymbolFunction(func_string));

                        index = next_index;
                    } else {
                        return Err(Error::new(ident.span(), "A function must be specified."));
                    }
                } else {
                    pattern.push(BNFSymbol::NonTerminalSymbolName(ident.to_string()));
                }
            },
            TokenTree::Group(group) => {
                let delimiter = group.delimiter();
                let symbols = group.stream().into_iter().collect::<Vec<TokenTree>>();
                let mut new_symbol_name = non_duplicate_number.as_symbol_name();

                match delimiter {
                    Delimiter::Parenthesis => {
                        parse_rule(rule_map, &mut new_symbol_name, &symbols, non_duplicate_number, unnamed_pattern_map)?;
                    },
                    Delimiter::Brace => { // new_symbol ::= null | new_pattern new_symbol
                        let mut new_pattern_name = non_duplicate_number.as_symbol_name();
                        parse_rule(rule_map, &mut new_pattern_name, &symbols, non_duplicate_number, unnamed_pattern_map)?;

                        let mut rule = BNFRule::new(new_symbol_name.clone());
                        let or_patterns = &mut rule.or_patterns;

                        or_patterns.push(vec![BNFSymbol::Null]);
                        or_patterns.push(vec![BNFSymbol::NonTerminalSymbolName(new_pattern_name), BNFSymbol::NonTerminalSymbolName(new_symbol_name.clone())]);

                        rule_map.insert(rule.non_terminal_symbol_name.clone(), rule);
                    },
                    Delimiter::Bracket => { // new_symbol ::= new_pattern | null
                        let mut new_pattern_name = non_duplicate_number.as_symbol_name();
                        parse_rule(rule_map, &mut new_pattern_name, &symbols, non_duplicate_number, unnamed_pattern_map)?;

                        let mut rule = BNFRule::new(new_symbol_name.clone());
                        let or_patterns = &mut rule.or_patterns;

                        or_patterns.push(vec![BNFSymbol::NonTerminalSymbolName(new_pattern_name)]);
                        or_patterns.push(vec![BNFSymbol::Null]);

                        rule_map.insert(rule.non_terminal_symbol_name.clone(), rule);
                    },
                    _ => {
                        return Err(Error::new(group.span(), "Invalid delimiter."));
                    }
                }

                pattern.push(BNFSymbol::NonTerminalSymbolName(new_symbol_name.clone()));
            },
            TokenTree::Literal(literal) => {
                let string = match StringLit::try_from(literal) {
                    Ok(string) => string.value().to_string(),
                    Err(err) => return Err(Error::new(literal.span(), format!("Invalid terminal symbol. {}", err.to_string())))
                };
                if literal.to_string().starts_with("r") {
                    pattern.push(BNFSymbol::TerminalSymbolFunction(string));
                } else {
                    pattern.push(BNFSymbol::TerminalSymbolString(string));
                }
            }
        }

        index += 1;
    }

    if !pattern.is_empty() {
        or_patterns.push(pattern);
    }

    // merge unnamed pattern
    if non_terminal_symbol_name.starts_with(' ') {
        match unnamed_pattern_map.get(or_patterns) {
            Some(temp_name) => {
                *non_terminal_symbol_name = temp_name.clone();
            },
            _ => {
                unnamed_pattern_map.insert(or_patterns.clone(), non_terminal_symbol_name.clone());

                rule.non_terminal_symbol_name = non_terminal_symbol_name.clone();
                rule_map.insert(non_terminal_symbol_name.clone(), rule);
            }
        }
    } else {
        rule.non_terminal_symbol_name = non_terminal_symbol_name.clone();
        rule_map.insert(non_terminal_symbol_name.clone(), rule);
    }

    return Ok(());
}


#[derive()]
pub struct TokenParser {
    pub symbols: Vec<TokenTree>
}


impl Parse for TokenParser {

    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut symbols = Vec::<TokenTree>::new();

        while !input.is_empty() {

            let token_tree = TokenTree::parse(input).unwrap();

            symbols.push(token_tree);
        }

        return Ok(TokenParser { symbols })
    }

}



#[derive(Debug)]
pub struct BNFRule {
    pub non_terminal_symbol_name: String,
    pub or_patterns: Vec<Vec<BNFSymbol>>,
    pub first_set: HashSet<BNFSymbol>,
    pub is_nullable: bool
}


impl BNFRule {

    pub fn new(non_terminal_symbol_name: String) -> Self {
        return Self {
            non_terminal_symbol_name,
            or_patterns: Vec::new(),
            first_set: HashSet::new(),
            is_nullable: false
        };
    }

}


#[derive(Debug, Eq, Clone, Hash, PartialEq)]
pub enum BNFSymbol {
    NonTerminalSymbolName(String),
    TerminalSymbolString(String),
    TerminalSymbolFunction(String),
    Null,
    EOF
}


impl BNFSymbol {

    pub fn is_terminal_symbol(&self) -> bool {
        return match self {
            BNFSymbol::NonTerminalSymbolName(_) => false,
            _ => true
        }
    }

    pub fn get_symbol_name(&self) -> &str {
        return match self {
            BNFSymbol::TerminalSymbolFunction(name) => name.as_str(),
            BNFSymbol::NonTerminalSymbolName(name) => name.as_str(),
            BNFSymbol::TerminalSymbolString(name) => name.as_str(),
            BNFSymbol::Null => "Null",
            BNFSymbol::EOF => "EOF"
        }
    }

}


pub struct NonDuplicateNumber {
    number: usize
}

impl NonDuplicateNumber {

    pub fn new() -> Self {
        return Self {
            number: 0
        };
    }

    pub fn next(&mut self) -> usize {
        self.number += 1;
        return self.number;
    }

    pub fn as_symbol_name(&mut self) -> String {
        return format!(" {}", self.next());
    }

}




pub struct ParserGenerator {
    rule_map: HashMap<String, BNFRule>,
    single_pattern_rules: Vec<SinglePatternRule>,
    symbol_id_map: HashMap<BNFSymbol, usize>
}


impl ParserGenerator {

    pub fn new(mut rule_map: HashMap<String, BNFRule>) -> Self {
        let mut source_rule = BNFRule::new(" source".to_string());
        source_rule.or_patterns.push(vec![BNFSymbol::NonTerminalSymbolName("source".to_string())]);

        rule_map.insert(" source".to_string(), source_rule);

        let mut single_pattern_rules = Vec::<SinglePatternRule>::new();
        for rule in rule_map.values() {
            for pattern in rule.or_patterns.iter() {
                let mut new_pattern = Vec::<BNFSymbol>::new();
                for symbol in pattern.iter() {
                    match symbol {
                        BNFSymbol::Null => {},
                        BNFSymbol::EOF => {},
                        _ => new_pattern.push(symbol.clone())
                    }
                }

                single_pattern_rules.push(SinglePatternRule::new(rule.non_terminal_symbol_name.clone(), new_pattern));
            }
        }

        let mut symbol_id_map = HashMap::<BNFSymbol, usize>::new();
        let mut last_id = 0;

        symbol_id_map.insert(BNFSymbol::EOF, last_id);
        last_id += 1;

        for rule in rule_map.values() {
            for pattern in rule.or_patterns.iter() {
                for symbol in pattern.iter() {
                    match symbol {
                        BNFSymbol::Null => {},
                        BNFSymbol::EOF => {},
                        _ => {
                            if !symbol_id_map.contains_key(symbol) {
                                symbol_id_map.insert(symbol.clone(), last_id);
                                last_id += 1;
                            }
                        }
                    }
                }
            }
        }

        symbol_id_map.insert(BNFSymbol::NonTerminalSymbolName(" source".to_string()), last_id);

        return Self {
            rule_map,
            single_pattern_rules,
            symbol_id_map
        }
    }


    pub fn generate(&mut self) -> Result<String, String> {
        self.search_nulls_and_first_set();
        return Ok(self.generate_parser()?);
    }


    fn search_nulls_and_first_set(&mut self) {

        let mut rule_nullable_map = HashMap::<String, bool>::new();

        loop {
            let mut retry = false;

            for rule in self.rule_map.values() {

                match rule_nullable_map.get(&rule.non_terminal_symbol_name) {
                    Some(is_nullable) => {
                        if *is_nullable {
                            continue
                        }
                    },
                    _ => {}
                }


                let mut is_nullable_rule = false;

                if rule.or_patterns.is_empty() {
                    is_nullable_rule = true;
                }

                for pattern in rule.or_patterns.iter() {

                    let mut nullable_count = 0;

                    for symbol in pattern.iter() {
                        match symbol {
                            BNFSymbol::NonTerminalSymbolName(name) => {
                                match rule_nullable_map.get(name) {
                                    Some(is_nullable) => {
                                        if *is_nullable {
                                            nullable_count += 1;
                                        }
                                    },
                                    _ => {}
                                }
                            },
                            BNFSymbol::Null => {
                                nullable_count += 1;
                            },
                            _ => {}
                        }
                    }

                    is_nullable_rule |= nullable_count == pattern.len();
                }

                rule_nullable_map.insert(rule.non_terminal_symbol_name.clone(), is_nullable_rule);
                if is_nullable_rule {
                    retry = true;
                }
            }

            if !retry {
                break;
            }
        }

        for entry in rule_nullable_map.iter() {
            self.rule_map.get_mut(entry.0).unwrap().is_nullable = *entry.1
        }

        let mut rule_first_set_map = HashMap::<String, HashSet<BNFSymbol>>::new();

        loop {
            let mut retry = false;

            for rule in self.rule_map.values() {

                match rule_first_set_map.get_mut(&rule.non_terminal_symbol_name) {
                    Some(_) => {},
                    _ => {
                        rule_first_set_map.insert(rule.non_terminal_symbol_name.clone(), HashSet::new());
                    }
                };

                let first_set = rule_first_set_map.get(&rule.non_terminal_symbol_name).unwrap();
                let mut first_set_add = HashSet::<BNFSymbol>::new();

                for pattern in rule.or_patterns.iter() {
                    for symbol in pattern.iter() {
                        match symbol {
                            BNFSymbol::NonTerminalSymbolName(name) => {
                                let is_nullable = match rule_nullable_map.get(name) {
                                    Some(is_nullable) => *is_nullable,
                                    _ => false
                                };

                                match rule_first_set_map.get(name) {
                                    Some(sub_set) => {
                                        for symbol in sub_set.iter() {
                                            if !first_set.contains(symbol) {
                                                first_set_add.insert(symbol.clone());
                                                retry = true;
                                            }
                                        }
                                    },
                                    _ => {}
                                }

                                if !is_nullable {
                                    break;
                                }
                            },
                            BNFSymbol::TerminalSymbolString(_) => {
                                if !first_set.contains(symbol) {
                                    first_set_add.insert(symbol.clone());
                                    retry = true;
                                }
                                break;
                            },
                            BNFSymbol::TerminalSymbolFunction(_) => {
                                if !first_set.contains(symbol) {
                                    first_set_add.insert(symbol.clone());
                                    retry = true;
                                }
                                break;
                            },
                            BNFSymbol::EOF => {
                                if !first_set.contains(symbol) {
                                    first_set_add.insert(symbol.clone());
                                    retry = true;
                                }
                                break;
                            },
                            _ => {}
                        }
                    }
                }

                let first_set = rule_first_set_map.get_mut(&rule.non_terminal_symbol_name).unwrap();
                first_set.extend(first_set_add);
            }

            if !retry {
                break;
            }
        }

        for entry in rule_nullable_map.iter() {
            let symbol_name = entry.0;
            let is_nullable = entry.1;
            let rule = self.rule_map.get_mut(symbol_name).unwrap();
            rule.is_nullable = *is_nullable;
        }

        for entry in rule_first_set_map.iter() {
            let symbol_name = entry.0;
            let first_set = entry.1;
            let rule = self.rule_map.get_mut(symbol_name).unwrap();
            rule.first_set = first_set.clone();
        }
    }


    fn generate_parser(&self) -> Result<String, String> {

        let mut lr_group_map = HashMap::<usize, LRGroup>::new();
        let mut not_scanned_group_list = Vec::<usize>::new();
        let mut last_group_number = 0;

        loop {
            let lr_group_number = if last_group_number == 0 {

                let mut first_group = LRGroup::new(0);

                let source_first_rule = self.rule_map.get(" source").unwrap();

                for pattern in source_first_rule.or_patterns.iter() {
                    let mut item = LRItem::new(" source".to_string());
                    item.pattern.extend(pattern.clone());
                    item.first_set.insert(BNFSymbol::EOF);
                    first_group.default_item_list.push(item.clone());
                    first_group.item_list.push(item);
                }

                self.add_items(&mut first_group);

                lr_group_map.insert(0, first_group);
                last_group_number += 1;
                0
            } else {
                match not_scanned_group_list.pop() {
                    Some(number) => number,
                    _ => break
                }
            };


            let lr_group = lr_group_map.get(&lr_group_number).unwrap();

            let mut next_group_map = HashMap::<BNFSymbol, Vec<&LRItem>>::new();
            for item in lr_group.item_list.iter() {
                match item.get_next_symbol() {
                    Some(symbol) => {
                        let list = next_group_map.entry(symbol).or_insert_with(|| Vec::new());
                        list.push(item);
                    }
                    _ => continue
                }
            }


            let mut next_group_number_map = HashMap::<BNFSymbol, usize>::new();
            let mut add_group_map = HashMap::<usize, LRGroup>::new();

            for entry in next_group_map.iter() {
                let symbol = entry.0;
                let items = entry.1;

                let mut next_group_items = Vec::<LRItem>::new();
                for item in items.iter() {
                    next_group_items.push(item.create_next().unwrap());
                }

                let mut group_number = Option::<usize>::None;
                for entry in lr_group_map.iter() {
                    let number = entry.0;
                    let lr_group = entry.1;

                    let mut is_all_matched = true;

                    if next_group_items.len() == lr_group.default_item_list.len() {
                        for i in 0..next_group_items.len() {
                            let item = &next_group_items[i];
                            let mut found = false;

                            for j in 0..lr_group.default_item_list.len() {
                                let group_item = &lr_group.default_item_list[j];

                                if item.root_name == group_item.root_name &&
                                    item.current_position == group_item.current_position &&
                                    item.pattern == group_item.pattern &&
                                    item.first_set == group_item.first_set {

                                    found = true;
                                    break;
                                }
                            }

                            if !found {
                                is_all_matched = false;
                                break;
                            }
                        }
                    } else {
                        is_all_matched = false;
                    }


                    if is_all_matched {
                        group_number = Some(*number);
                    }
                }

                match group_number {
                    Some(number) => {
                        next_group_number_map.insert(symbol.clone(), number);
                    },
                    _ => {
                        let mut new_group = LRGroup::new(last_group_number);
                        new_group.default_item_list = next_group_items.clone();
                        new_group.item_list = next_group_items;

                        self.add_items(&mut new_group);

                        add_group_map.insert(last_group_number, new_group);
                        not_scanned_group_list.push(last_group_number);

                        next_group_number_map.insert(symbol.clone(), last_group_number);

                        last_group_number += 1;
                    }
                }
            }

            let lr_group = lr_group_map.get_mut(&lr_group_number).unwrap();
            lr_group.next_group_number_map = next_group_number_map;

            lr_group_map.extend(add_group_map);
        }


        let mut table = Vec::<Vec<Option<Operation>>>::new();
        for group_number in 0..lr_group_map.len() {

            let group = lr_group_map.get(&group_number).unwrap();
            let mut operation_map = HashMap::<BNFSymbol, Operation>::new();

            for entry in group.next_group_number_map.iter() {
                let symbol = entry.0;
                let next_group_number = *entry.1;

                match symbol {
                    BNFSymbol::NonTerminalSymbolName(_) => {
                        self.insert_opreration(&mut operation_map, symbol, Operation::GoTo(next_group_number))?;
                    },
                    BNFSymbol::TerminalSymbolString(_) => {
                        self.insert_opreration(&mut operation_map, symbol, Operation::Shift(next_group_number))?;
                    },
                    BNFSymbol::TerminalSymbolFunction(_) => {
                        self.insert_opreration(&mut operation_map, symbol, Operation::Shift(next_group_number))?;
                    },
                    _ => {
                        return Err(format!("Unexpected symbol. {:?}", symbol));
                    }
                }
            }

            for item in group.item_list.iter() {
                if item.is_last_position() {
                    if item.root_name == " source" {
                        self.insert_opreration(&mut operation_map, &BNFSymbol::EOF, Operation::Accept)?;
                    } else {
                        let pattern_id = self.get_pattern_id(item)?;
                        for symbol in item.first_set.iter() {
                            self.insert_opreration(&mut operation_map, symbol, Operation::Reduce(pattern_id))?;
                        }
                    }
                }
            }

            let mut operations = Vec::<Option<Operation>>::new();
            for _ in 0..self.symbol_id_map.len() {
                operations.push(None);
            }

            for entry in operation_map.iter() {
                let symbol_id = self.get_symbol_id(entry.0)?;
                let operation = entry.1;
                operations[symbol_id] = Some(operation.clone());
            }

            table.push(operations);
        }


        let mut right_side_counts = Vec::<usize>::new();
        let mut left_side_symbol_ids = Vec::<usize>::new();

        for rule in self.single_pattern_rules.iter() {
            right_side_counts.push(rule.pattern.len());
            let symbol_id = self.get_symbol_id(&BNFSymbol::NonTerminalSymbolName(rule.root_symbol_name.clone()))?;
            left_side_symbol_ids.push(symbol_id);
        }

        let mut symbol_is_terminal = Vec::<bool>::new();
        for _ in 0..self.symbol_id_map.len() {
            symbol_is_terminal.push(false);
        }
        for symbol in self.symbol_id_map.keys() {
            let symbol_id = self.symbol_id_map[symbol];
            symbol_is_terminal[symbol_id] = symbol.is_terminal_symbol();
        }

        let mut code = "".to_string();
        code += "
        use bnf_rules::bnf_rules_parser::lexer::{*};
        use bnf_rules::bnf_rules_parser::parser::{*};
        use bnf_rules::bnf_rules_parser::parser::ASTNode::{NonTerminal, Terminal};
        ";

        code += "pub fn parse_source(source: &str) -> Result<ASTNode, ParseError> {";


        let mut array_str = String::new();
        for rule_root_name in self.single_pattern_rules.iter() {
            array_str += format!("\"{}\", ", &rule_root_name.root_symbol_name).as_str();
        }
        code += format!("static RULE_PATTERN_NAME: &[&str] = &[{}];", array_str).as_str();


        let mut group_array_str = String::new();
        for group in table.iter() {
            let mut array_str = String::new();

            for operation in group.iter() {
                match operation {
                    Some(operation) => {
                        let tuple = operation.to_tuple();
                        array_str += format!("({}, {}), ", tuple.0, tuple.1).as_str();
                    },
                    _ => array_str += format!("(0, 0), ").as_str()
                }
            }

            group_array_str += format!("&[{}], ", array_str).as_str();
        }
        code += format!("static LR_TABLE: &[&[(usize, usize)]] = &[{}];", group_array_str).as_str();


        let mut rule_array_str = String::new();
        for pattern in self.single_pattern_rules.iter() {
            let root_symbol_id = self.symbol_id_map[&BNFSymbol::NonTerminalSymbolName(pattern.root_symbol_name.clone())];

            let mut array_str = String::new();
            for symbol in pattern.pattern.iter() {
                array_str += format!("{}, ", self.symbol_id_map[symbol]).as_str();
            }

            rule_array_str += format!("({}, &[{}]), ", root_symbol_id, array_str).as_str();
        }

        code += format!("static BNF_RULES: &[(u32, &[u32])] = &[{}];", rule_array_str).as_str();


        code += "let mut terminal_symbols = Vec::<TerminalSymbol>::new();";
        for entry in self.symbol_id_map.iter() {
            let symbol = entry.0;
            let symbol_id = entry.1;

            match symbol {
                BNFSymbol::TerminalSymbolString(string) => {
                    code += format!("terminal_symbols.push(TerminalSymbol::new_from_string(\"{}\", {}));", string, symbol_id).as_str();
                },
                BNFSymbol::TerminalSymbolFunction(fn_string) => {
                    code += format!("terminal_symbols.push(TerminalSymbol::new_from_tokenizer_fn({}, {}));", fn_string, symbol_id).as_str();
                }
                _ => {}
            }
        }
        code += "let lexer = Lexer::new(terminal_symbols);";


        code += "let tokens = lexer.scan(source);";
        code += "return __parse(tokens, RULE_PATTERN_NAME, LR_TABLE, BNF_RULES);";
        code += "}";

        return Ok(code);
    }


    fn insert_opreration(&self, operation_map: &mut HashMap<BNFSymbol, Operation>, symbol: &BNFSymbol, operation: Operation) -> Result<(), String> {
        if operation_map.contains_key(symbol) {
            let mut message = format!("{:?} {:?} conflict!", operation_map.get(symbol).unwrap(), operation);

            match symbol {
                BNFSymbol::NonTerminalSymbolName(symbol_name) => {
                    if !symbol_name.is_empty() {
                        message += format!(" | Symbol : {} ::=", symbol_name).as_str();

                        let rule = self.rule_map.get(symbol_name).unwrap();
                        for pattern in rule.or_patterns.iter() {
                            for symbol in pattern.iter() {
                                message += format!(" {} ", symbol.get_symbol_name()).as_str();
                            }
                            message += "|";
                        }
                    }
                },
                _ => {
                    message += format!(" | Symbol : '{}'", symbol.get_symbol_name()).as_str();
                }
            }


            return Err(message);
        }
        operation_map.insert(symbol.clone(), operation);
        return Ok(());
    }



    fn add_items(&self, lr_group: &mut LRGroup) {
        loop {
            let mut is_all_scanned = true;
            let mut add_item_list = Vec::<LRItem>::new();

            for i in 0..lr_group.item_list.len() {
                let item = &mut lr_group.item_list[i];

                if item.is_scanned || item.is_last_position() {
                    continue;
                }
                is_all_scanned = false;
                item.is_scanned = true;

                let next_symbol_name = match item.get_next_nonterminal_symbol_name() {
                    Some(name) => name,
                    _ => continue
                };

                let item = &lr_group.item_list[i];

                let latter_pattern = item.get_latter_pattern();

                let first_set = self.get_first_set(&latter_pattern, &item.first_set);


                let rule = self.rule_map.get(&next_symbol_name).unwrap();

                for pattern in rule.or_patterns.iter() {

                    let mut found = false;
                    for item in lr_group.item_list.iter_mut() {
                        if item.root_name == next_symbol_name && &item.pattern == pattern && item.current_position == 0 {
                            found = true;
                            item.first_set.extend(first_set.clone());
                            break;
                        }
                    }

                    if !found {
                        let mut item = LRItem::new(next_symbol_name.clone());

                        for symbol in pattern.iter() {
                            match symbol {
                                BNFSymbol::Null => {},
                                BNFSymbol::EOF => {},
                                _ => {
                                    item.pattern.push(symbol.clone());
                                }
                            }
                        }
                        item.first_set = first_set.clone();
                        add_item_list.push(item);
                    }
                }

            }

            lr_group.item_list.extend(add_item_list);

            if is_all_scanned {
                break;
            }
        }
    }



    fn get_pattern_id(&self, item: &LRItem) -> Result<usize, String> {
        for i in 0..self.single_pattern_rules.len() {
            let pattern = &self.single_pattern_rules[i];
            if pattern.root_symbol_name == item.root_name && pattern.pattern == item.pattern {
                return Ok(i);
            }
        }
        return Err(format!("Internal error. Not found pattern. {:?}", item));
    }



    fn get_symbol_id(&self, symbol: &BNFSymbol) -> Result<usize, String> {
        return match self.symbol_id_map.get(symbol) {
            Some(id) => Ok(*id),
            _ => Err(format!("Internal error. Symbol's id is not found. {:?}", symbol))
        }
    }



    fn get_first_set(&self, symbol_list: &Vec<BNFSymbol>, item_first_set: &HashSet<BNFSymbol>) -> HashSet<BNFSymbol> {

        let mut first_set = HashSet::<BNFSymbol>::new();

        let mut is_nullable = true;
        for symbol in symbol_list.iter() {
            match symbol {
                BNFSymbol::NonTerminalSymbolName(name) => {
                    let rule = self.rule_map.get(name).unwrap();
                    first_set.extend(rule.first_set.clone());
                    if !rule.is_nullable {
                        is_nullable = false;
                        break;
                    }
                },
                BNFSymbol::Null => {},
                _ => {
                    first_set.insert(symbol.clone());
                    is_nullable = false;
                    break;
                }
            }
        }

        if is_nullable {
            first_set.extend(item_first_set.clone());
        }

        return first_set;
    }

}



#[derive(Debug)]
pub struct LRGroup {
    pub group_number: usize,
    pub default_item_list: Vec<LRItem>,
    pub item_list: Vec<LRItem>,
    pub next_group_number_map: HashMap<BNFSymbol, usize>
}


impl LRGroup {

    pub fn new(group_number: usize) -> Self {
        return Self {
            group_number,
            default_item_list: Vec::new(),
            item_list: Vec::new(),
            next_group_number_map: HashMap::new()
        }
    }

}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LRItem {
    pub root_name: String,
    pub pattern: Vec<BNFSymbol>,
    pub first_set: HashSet<BNFSymbol>,
    pub current_position: usize,
    pub is_scanned: bool
}


impl LRItem {

    pub fn new(root_name: String) -> Self {
        return Self {
            root_name,
            pattern: Vec::new(),
            first_set: HashSet::new(),
            current_position: 0,
            is_scanned: false
        }
    }

    pub fn is_last_position(&self) -> bool {
        return self.pattern.len() == self.current_position;
    }

    pub fn get_next_nonterminal_symbol_name(&self) -> Option<String> {
        return match self.pattern.get(self.current_position) {
            Some(symbol) => {
                match symbol {
                    BNFSymbol::NonTerminalSymbolName(name) => Some(name.clone()),
                    _ => None
                }
            },
            _ => None
        };
    }

    pub fn get_next_symbol(&self) -> Option<BNFSymbol> {
        return self.pattern.get(self.current_position).cloned();
    }

    pub fn get_latter_pattern(&self) -> Vec<BNFSymbol> {
        let mut latter_pattern = Vec::<BNFSymbol>::new();
        if self.current_position + 1 >= self.pattern.len() {
            return latter_pattern;
        }

        for i in (self.current_position + 1)..self.pattern.len() {
            let symbol = &self.pattern[i];
            latter_pattern.push(symbol.clone());
        }

        return latter_pattern;
    }

    pub fn create_next(&self) -> Option<Self> {
        if self.is_last_position() {
            return None;
        }

        let mut cloned = self.clone();
        cloned.current_position += 1;
        cloned.is_scanned = false;

        return Some(cloned);
    }

}



#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SinglePatternRule {
    pub root_symbol_name: String,
    pub pattern: Vec<BNFSymbol>,
}


impl SinglePatternRule {

    pub fn new(root_symbol_name: String, pattern: Vec<BNFSymbol>) -> Self {
        let mut new_pattern = Vec::<BNFSymbol>::new();
        for symbol in pattern.iter() {
            match symbol {
                BNFSymbol::Null => {},
                _ => {
                    new_pattern.push(symbol.clone());
                }
            }
        }

        return Self {
            root_symbol_name,
            pattern: new_pattern
        }
    }

}



#[derive(Debug, Clone)]
pub enum Operation {
    Shift(usize),
    Reduce(usize),
    GoTo(usize),
    Accept
}


pub const OPERATION_NONE: usize = 0;
pub const OPERATION_SHIFT: usize = 1;
pub const OPERATION_REDUCE: usize = 2;
pub const OPERATION_GOTO: usize = 3;
pub const OPERATION_ACCEPT: usize = 4;

impl Operation {

    pub fn to_tuple(&self) -> (usize, usize) {
        return match self {
            Operation::Shift(i) => (OPERATION_SHIFT, *i),
            Operation::Reduce(i) => (OPERATION_REDUCE, *i),
            Operation::GoTo(i) => (OPERATION_GOTO, *i),
            Operation::Accept => (OPERATION_ACCEPT, 0)
        };
    }

}