#![allow(unused)]

struct StyleSheet {
    rules: Vec<Rule>,
}

struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>
}

pub type Specificity = (usize, usize, usize);
#[derive(Debug)]
enum Selector {
    Simple(SimpleSelector)
}

impl Selector {
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(simple) = self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }
}

#[derive(Debug)]
struct SimpleSelector {
    tag_name: Option<String>,
    id: Option<String>,
    class: Vec<String>
}

struct Declaration {
    name: String,
    value: Value
}

#[derive(Debug, PartialEq)]
enum Unit {
    Px,
}

#[derive(Debug, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
#[derive(Debug, PartialEq)]
enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color)
}

impl Value {
    fn parse_hex(s: &str) -> u64 {
        let mut res: u64 = 0;
        s.as_bytes()
            .iter()
            .map(|i| { 
                match i {
                    b'0'..=b'9' => { res = res*16+ *i as u64 - b'0' as u64; }
                    b'a'..=b'f' => { res = res*16+ *i as u64 - b'a' as u64 + 10; }
                    b'A'..=b'F' => { res = res*16+ *i as u64 - b'A' as u64 +10; }
                    _ => {}
                }
            }).count();
        return res;
    }
    pub fn parse_as_color(s: &str) -> Option<Color> {
        if s.starts_with("#") {
            let color_hex = Value::parse_hex(&s[1..]);
            Some(Color {
                r: (color_hex >> 16 & 0xFF) as u8,
                g: (color_hex >>  8 & 0xFF) as u8,
                b: (color_hex >>  0 & 0xFF) as u8,
                a: 0,
            })
        } else if s.starts_with("rgba(") {
            let mut sp = s[5..s.len()-1].split(",");
            Some(Color {
                r: sp.next().unwrap_or("0").trim().parse().unwrap(),
                g: sp.next().unwrap_or("0").trim().parse().unwrap(),
                b: sp.next().unwrap_or("0").trim().parse().unwrap(),
                a: sp.next().unwrap_or("0").trim().parse().unwrap(),
            })
        } else {
            None
        }
    }
    pub fn parse_as_length(s: &str) -> Option<(f32, Unit)> {
        if s.ends_with("px") {
            Some((s[0..s.len()-2].parse().unwrap_or(0.0), Unit::Px))
        } else {
            None
        }
    }
}

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    pub fn new<S: ToString>(s: S) -> Parser {
        Parser {
            pos: 0 as usize,
            input: s.to_string()
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (offset, _) = iter.next().unwrap_or((1, ' '));
        self.pos += offset;
        cur_char
    }
    fn consume_while<F: Fn(char) -> bool>(&mut self, test: F) -> String {
        let mut res = String::new();

        while !self.eof() && test(self.next_char()) {
            res.push(self.consume_char());
        }

        return res;
    }
    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }
    
    fn valid_identifier_char(c: char) -> bool {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
            _ => false
        }
    }
    fn valid_declaration_name_char(c: char) -> bool {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' => true,
            _ => false
        }
    }
    fn valid_declaration_value_char(c: char) -> bool {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '#' | '\'' | '"' | '(' | ')' | ' ' | ',' => true,
            _ => false
        }
    }

    // parse a css rule, like
    // p.class1, p#id1          ------------this line is called selectors
    // {                        ------------below is called declarations
    //     color: red;
    //     padding: 10px
    // }
    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations()
        }
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                c => panic!("Unexpected character {} in selector list!", c)
            }
        }
        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));

        return selectors;
    }
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {tag_name: None, id: None, class: Vec::new()};
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                }
                c if Parser::valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break
            }
        }
        selector
    }
    fn parse_identifier(&mut self) -> String {
        self.consume_while(|c| Parser::valid_identifier_char(c))
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        assert!(self.consume_char() == '{');

        loop {
            self.consume_whitespace();
            match self.next_char() {
                '}' => { self.consume_char(); break }
                ';' => { self.consume_char(); }
                _ => declarations.push(self.parse_one_declaration())
            }
        }
        
        return declarations;
    }
    // parse one declaration like
    // color: red; padding: 10px; display: none
    fn parse_one_declaration(&mut self) -> Declaration {
        // parse name of one
        self.consume_whitespace();
        let name = self.consume_while(|c| Parser::valid_declaration_name_char(c));

        self.consume_whitespace();
        assert!(self.consume_char() == ':');
        self.consume_whitespace();
        let value = self.consume_while(|c| Parser::valid_declaration_value_char(c));
        let value = Parser::parse_one_declaration_value(value);

        Declaration {name, value}
    }
    fn parse_one_declaration_value(value_string: String) -> Value {
        if value_string.is_empty() { return Value::Keyword("".to_string()); }

        let mut value_str = &value_string[0..];
        if value_str.starts_with(|c| c == '\'' || c == '"') {
            value_str = &value_str[1..value_str.len()-1];
        }

        // parse this value
        if let Some(color)=Value::parse_as_color(value_str) {
            Value::ColorValue(color)
        } else if let Some(length)=Value::parse_as_length(value_str) {
            Value::Length(length.0, length.1)
        } else {
            Value::Keyword(value_string)
        }
    }
}


//------------test--------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_declaration() {
        // color1: red
        let mut parser = Parser::new("color1: red");
        let declaration = parser.parse_one_declaration();
        assert_eq!(declaration.name, "color1");
        assert_eq!(declaration.value, Value::Keyword("red".to_string()));

        // color2: #99A0AB
        let mut parser = Parser::new(" color2: #99A0AB");
        let declaration = parser.parse_one_declaration();
        assert_eq!(declaration.name, "color2");
        assert_eq!(declaration.value, Value::ColorValue(Color {r: 0x99, g: 0xA0, b: 0xAB, a: 0}));

        // color3: rgba(1,2,3,4)
        let mut parser = Parser::new("color3: rgba(1, 2,3 ,4 )");
        let declaration = parser.parse_one_declaration();
        assert_eq!(declaration.name, "color3");
        assert_eq!(declaration.value, Value::ColorValue(Color {r: 1, g: 2, b: 3, a: 4}));

        // padding: 100px
        let mut parser = Parser::new("padding: 100px");
        let declaration = parser.parse_one_declaration();
        assert_eq!(declaration.name, "padding");
        assert_eq!(declaration.value, Value::Length(100.0, Unit::Px));
    }

    #[test]
    fn test_css_parser() {
        let mut parser = Parser::new("h1.que, div#answer, .fuck, * {color: #FFFFFF; width: 10px; display:none; }");
        let mut res = parser.parse_rule();

        assert_eq!(res.selectors.len(), 4);
        assert_eq!(res.declarations.len(), 3);

        let Selector::Simple(selector1) = &res.selectors[0];
        let Selector::Simple(selector2) = &res.selectors[1];
        let Selector::Simple(selector3) = &res.selectors[2];
        let Selector::Simple(selector4) = &res.selectors[3];
        
        // after sort: first shoud be "div"
        assert_eq!(selector1.tag_name, Some("div".to_owned()));
        assert_eq!(selector1.id, Some("answer".to_owned()));
        assert_eq!(selector1.class, Vec::<String>::new());

        assert_eq!(selector2.tag_name, Some("h1".to_owned()));
        assert_eq!(selector2.id, None);
        assert_eq!(selector2.class, vec!["que".to_owned()]);

        assert_eq!(selector3.tag_name, None);
        assert_eq!(selector3.id, None);
        assert_eq!(selector3.class, vec!["fuck".to_owned()]);

        assert_eq!(selector4.tag_name, None);
        assert_eq!(selector4.id, None);
        assert_eq!(selector4.class, Vec::<String>::new());

        let declaration1 = &res.declarations[0];
        let declaration2 = &res.declarations[1];
        let declaration3 = &res.declarations[2];

        assert_eq!(declaration1.name, "color");
        assert_eq!(declaration1.value, Value::ColorValue(Color {r: 0xFF, g: 0xFF, b: 0xFF, a: 0}));

        assert_eq!(declaration2.name, "width");
        assert_eq!(declaration2.value, Value::Length(10.0, Unit::Px));

        assert_eq!(declaration3.name, "display");
        assert_eq!(declaration3.value, Value::Keyword("none".to_owned()));
    }

}
