use std::usize;

use crate::dom::{AttrMap, Node};

pub struct Html {
    pos: usize,
    input: String,
}

impl Html {
    /// Read the current character with out consuming it
    fn next_char(&self) -> char {
        self.input.get(self.pos..).unwrap().chars().next().unwrap()
    }

    /// Do the next characters start with the given string
    fn starts_with(&self, s: &str) -> bool {
        self.input.get(self.pos..).unwrap().starts_with(s)
    }

    /// Return true if input is consumed
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input.get(self.pos..).unwrap().char_indices();
        let (_, curr_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        curr_char
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Parse a tag or attribute name
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false,
        })
    }

    fn parse_node(&mut self) -> Option<Node> {
        if self.starts_with("<!--") {
            self.parse_comment();
            return None;
        } else if self.starts_with("</") {
            return None;
        }
        match self.next_char() {
            '<' => Some(self.parse_element()),
            _ => Some(self.parse_text()),
        }
    }

    fn parse_text(&mut self) -> Node {
        let mut data = String::new();
        loop {
            let text = self.consume_while(|c| c != '<');
            data.push_str(&text);
            if self.starts_with("<!--") {
                self.parse_comment();
            } else {
                break;
            }
        }
        Node::new_text(data)
    }

    fn parse_element(&mut self) -> Node {
        // parse opening tag
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        // ensure closing tag is next
        assert!(self.consume_char() == '>');

        // get contents
        let children = self.parse_nodes();

        // parse closing tag
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        Node::new_element(tag_name, attrs, children)
    }

    /// Parse single key value attr pair
    fn parse_attr(&mut self) -> (String, String) {
        let key = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        (key, value)
    }

    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        value
    }

    fn parse_attributes(&mut self) -> AttrMap {
        let mut attributes = AttrMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (key, value) = self.parse_attr();
            attributes.insert(key, value);
        }
        attributes
    }

    fn parse_comment(&mut self) {
        // consume the comment opening
        for _ in 0..4 {
            self.consume_char();
        }
        // look for comment closing
        while !self.starts_with("-->") {
            self.consume_char();
        }

        // consume the comment close
        for _ in 0..3 {
            self.consume_char();
        }
    }

    /// Parse child nodes recursively
    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            // check for comment node which should not be added to nodes
            if self.starts_with("<!--") {
                self.parse_comment();
            } else if self.eof() || self.starts_with("</") {
                break;
            }
            match self.parse_node() {
                Some(node) => nodes.push(node),
                None => continue,
            }
        }
        nodes
    }

    /// Parse an html document and return the root node
    pub fn parse(source: String) -> Node {
        let mut nodes = Html {
            pos: 0,
            input: source,
        }
        .parse_nodes();

        // if there is only one node, return it
        if nodes.len() == 1 {
            nodes.swap_remove(0)
        } else {
            Node::new_element("html".to_string(), AttrMap::new(), nodes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let html = "<h1>Hello, <i>world!</i></h1>".to_string();
        let test_dom = Html::parse(html);

        // ensure the root node has the correct number of children
        assert!(test_dom.children().len() == 2);

        // ensure the root node is the correct type
        assert!(test_dom.node_type().unwrap() == "element".to_string());
        let tag = test_dom.get_tag().unwrap();
        assert!(&tag == "h1");

        // ensure first child is a text node
        let first_child = test_dom.children().get(0).unwrap();
        assert!(first_child.node_type().unwrap() == "text".to_string());
        // test text content
        let expected = "Hello, ".to_string();
        assert!(first_child.get_text().unwrap() == expected);

        // test second child
        let second_child = test_dom.children().get(1).unwrap();

        // test that second child is an i tag
        let tag = second_child.get_tag().unwrap();
        assert!(&tag == "i");

        // test that second child has a child
        assert!(!second_child.children().is_empty());

        // test the grand child
        let grand_child = second_child.children().get(0).unwrap();
        assert!(&grand_child.node_type().unwrap() == "text");
        let expected = "world!";
        assert!(&grand_child.get_text().unwrap() == expected);
    }

    #[test]
    fn test_parse_comment_text() {
        let html = "<h1>Hello <!-- this is a comment --> world</h1>".to_string();
        let root = Html::parse(html);

        // Ensure the root node only has one child, the text within h1 tags
        assert!(root.children().len() == 1);

        // Ensure the text only contains the full text with comment removed
        let child = root.children().get(0).unwrap();
        assert!(&child.get_text().unwrap() == "Hello  world");
    }

    #[test]
    fn test_parse_comment_node() {
        let html = "<h1><!-- comment --></h1>".to_string();
        let root = Html::parse(html);

        // assert that only one node was parsed
        assert!(root.children().is_empty());

        // assert the h1 tag is parsed properly
        assert!(&root.get_tag().unwrap() == "h1");
    }
}
