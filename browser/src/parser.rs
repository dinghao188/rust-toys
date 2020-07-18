use crate::dom;
use std::collections::HashSet;

pub struct Parser {
    pos: usize,
    input: String,
    
    open_tags: HashSet<String>
}

impl Parser {
    pub fn new<S>(source: S) -> Parser where S: ToString {
        Parser {
            pos: 0,
            input: source.to_string(),
            open_tags: ["meta", "br", "link"].iter().map(|item| item.to_string()).collect()
        }
    }

    pub fn parse(&mut self) -> dom::Node {
        let mut nodes = self.parse_nodes();

        if nodes.len() == 1 {
            nodes.swap_remove(0)
        } else {
            dom::elem("html".to_string(), dom::AttrMap::new(), nodes)
        }
    }

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }
    fn consume_while<F>(&mut self, test: F) -> String
            where F: Fn(char) -> bool {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }
    fn partial_consume_while<F>(&mut self, test: F, filter: F) -> String
            where F: Fn(char) -> bool {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            let c = self.consume_char();
            if filter(c) {
                result.push(c);
            }
        }
        result
    }

    fn consume_until(&mut self, _str: &str) -> String {
        let mut result = String::new();
        while !self.eof() && !self.starts_with(_str) {
            result.push(self.consume_char());
        }
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' => true,
            _ => false
        })
    }

    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' =>  {
                if self.starts_with("<!--") {
                    self.parse_comment()
                } else {
                    self.parse_element()
                }
            },
            _ => self.parse_text()
        }
    }

    fn parse_text(&mut self) -> dom::Node {
        self.consume_whitespace();
        dom::text(self.consume_while(|c| c != '<'))
    }

    fn parse_comment(&mut self) -> dom::Node {
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '!');
        assert!(self.consume_char() == '-');
        assert!(self.consume_char() == '-');

        let res = dom::comment(self.consume_until("-->"));

        assert!(self.consume_char() == '-');
        assert!(self.consume_char() == '-');
        assert!(self.consume_char() == '>');

        return res;
    }

    fn parse_element(&mut self) -> dom::Node {
        // step1.1 parse the tag_name
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();

        // step2. parse attributes
        self.consume_whitespace();
        let attrs = self.parse_attributes();
        
        // step3. parse sub nodes
        let children;
        self.consume_whitespace();
        if self.open_tags.contains(&tag_name) && self.starts_with("/>") {
            assert!(self.consume_char() == '/');
            assert!(self.consume_char() == '>');
            return dom::elem(tag_name, attrs, Vec::new());
        } else {
            assert!(self.consume_char() == '>');
            children = self.parse_nodes();
            assert!(self.consume_char() == '<');
            assert!(self.consume_char() == '/');
            assert!(self.parse_tag_name() == tag_name);
            assert!(self.consume_char() == '>');
        }

        dom::elem(tag_name, attrs, children)
    }

    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        self.consume_whitespace();

        assert!(self.consume_char() == '=', format!("{}", &self.input[self.pos..self.pos+40]));
        let value = self.parse_attr_value();
        (name, value)
    }
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote=='"' || open_quote=='\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        value
    }

    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attrs = dom::AttrMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' || self.next_char() == '/' {
                break;
            } 
            let (name, value) = self.parse_attr();
            attrs.insert(name, value);
        };
        attrs
    }

    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();

        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }

        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut parser = Parser::new(String::from("Hello, Parser"));
        assert_eq!(parser.next_char(), 'H');
        assert_eq!(parser.pos, 0);

        assert_eq!(parser.starts_with("Hell "), false);
        assert_eq!(parser.starts_with("Hell"), true);

        let mut tmp = String::new();
        while !parser.eof() {
            tmp.push(parser.consume_char());
        }
        assert_eq!(parser.pos, tmp.len());
        assert_eq!(parser.input, tmp);
        println!("{}", tmp);
    }

    #[test]
    fn test_parse_tag_name() {
        let mut parser = Parser::new("<html_body>");

        parser.parse_tag_name();
        assert_eq!(parser.pos, 0);

        assert_eq!(parser.consume_char(), '<');
        let tag_name = parser.parse_tag_name();
        assert_eq!(tag_name, "html");
    }

    #[test]
    fn test_parse_all() {
        let content = "<html lang=\"en\" class='all'><!--html_comment--><body><!--body_comment-->fuck</body></html>";
        let html = Parser::new(content.to_owned()).parse();

        assert_eq!(html.get_attribute("lang"), Some("en".to_owned()));
        assert_eq!(html.get_attribute("class"), Some("all".to_owned()));
        assert_eq!(html.get_attribute("no_attr"), None);
        
        let comment = &html.children[0];
        assert_eq!("html_comment", if let dom::NodeType::Comment(_comment)=&comment.node_type {_comment} else {""});
        
        assert_eq!(format!("{}", html), "<html lang=\"en\" class=\"all\"><!--html_comment--><body><!--body_comment-->fuck</body></html>");
    }
}
