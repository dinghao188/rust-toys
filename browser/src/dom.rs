#![allow(unused)]
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct Node {
    pub children: Vec<Node>,

    pub node_type: NodeType
}

impl Node {
    pub fn append_child(&mut self, child: Node) {
        self.children.push(child);
    }

    pub fn add_attributes(&mut self, name: String, value: String) {
        match &mut self.node_type {
            NodeType::Element(_elem) => _elem.attributes.insert(name, value),
            _ => None
        };
    }

    pub fn get_attribute<K: std::string::ToString>(&self, name: K) -> Option<String> {
        match &self.node_type {
            NodeType::Element(_elem) => {
                if let Some(_res)=_elem.attributes.get(&name.to_string()) {
                    Some(_res.clone())
                } else {
                    None
                }
            },
            _ => None
        }
    }
}

impl Display for Node {
    fn fmt(&self,  f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.node_type {
            NodeType::Text(_text) => writeln!(f, "{}", _text),
            NodeType::Element(_elem) => {
                let mut attrs_string = String::new();
                for (name, value) in _elem.attributes.iter() {
                    attrs_string += " ";
                    attrs_string = format!("{}{}=\"{}\"", attrs_string,  name, value);
                }

                writeln!(f, "<{}{}>", _elem.tag_name, attrs_string)?;
                
                for child in &self.children {
                    write!(f, "{}", child)?;
                }
                
                writeln!(f, "</{}>", _elem.tag_name)
            },
            NodeType::Comment(_comment) => writeln!(f, "<!--{}-->", _comment)
        }
    }
}

#[derive(Debug)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
    Comment(String)
}

#[derive(Debug)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttrMap
}

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> std::collections::HashSet<&str>{
        match self.attributes.get("class") {
            Some(class_list) => {class_list.split(' ').collect()},
            None => std::collections::HashSet::new()
        }
    }
}

pub type AttrMap = std::collections::HashMap<String, String>;

pub fn text(data: String) -> Node {
    Node { children: Vec::new(), node_type: NodeType::Text(data) }
}

pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        children: children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs
        })
    }
}

pub fn comment(content: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Comment(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom() {
        //<html><body>hello<!-- comment --></body></html>
        let mut html = elem("html".to_owned(), AttrMap::new(), Vec::new());
        let mut body = elem("body".to_owned(), AttrMap::new(), Vec::new());
        body.append_child(text("hello".to_owned()));
        body.append_child(comment(" comment ".to_owned()));
        html.append_child(body);

        assert_eq!(html.children.len(), 1);
        assert_eq!(html.children[0].children.len(), 2);
    }
}
