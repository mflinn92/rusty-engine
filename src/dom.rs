use std::collections::HashMap;

#[derive(Debug)]
pub struct Node {
    // data common to all nodes
    children: Vec<Node>,

    // data specific to a node type
    node_type: NodeType,
}

impl Node {
    pub fn new_text(data: String) -> Self {
        Node {
            children: Vec::new(),
            node_type: NodeType::Text(data),
        }
    }

    pub fn new_element(name: String, attrs: AttrMap, children: Vec<Node>) -> Self {
        Node {
            children,
            node_type: NodeType::Element(ElementData {
                tag_name: name,
                attributes: attrs,
            }),
        }
    }

    pub fn children(&self) -> &Vec<Node> {
        &self.children
    }

    pub fn node_type(&self) -> Option<String> {
        match self.node_type {
            NodeType::Text(_) => Some("text".to_string()),
            NodeType::Element(_) => Some("element".to_string()),
        }
    }

    /// Returns the node tag if exists
    pub fn get_tag(&self) -> Option<String> {
        match &self.node_type {
            NodeType::Element(elem) => Some(elem.tag_name.clone()),
            _ => None,
        }
    }

    pub fn get_text(&self) -> Option<String> {
        match &self.node_type {
            NodeType::Text(data) => Some(data.to_string()),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum NodeType {
    Text(String),
    Element(ElementData),
}

#[derive(Debug)]
struct ElementData {
    tag_name: String,
    attributes: AttrMap,
}

pub type AttrMap = HashMap<String, String>;

mod tests {
    use super::*;

    #[test]
    fn test_new_text_node() {
        let data = String::from("This is some text");
        let node = Node::new_text(data.clone());

        // node should have no children
        assert!(node.children.is_empty());

        // should contain data text
        if let NodeType::Text(got) = node.node_type {
            assert!(got == data);
        }
    }

    #[test]
    fn test_new_element_node() {
        let mut attrs = AttrMap::new();
        let test_key = "test_key";
        let test_attr = "test attribute";
        attrs.insert(String::from(test_key), String::from(test_attr));

        let tag_name = String::from("test_elem");
        let node = Node::new_element(tag_name, attrs.clone(), Vec::new());

        // should have no children
        assert!(node.children.is_empty());

        // Check element data
        if let NodeType::Element(elem) = node.node_type {
            assert!(elem.tag_name.starts_with("test_elem"));
            assert!(elem.attributes.contains_key(test_key));
            assert!(elem
                .attributes
                .get(test_key)
                .unwrap()
                .starts_with(test_attr));
        }
    }

    #[test]
    fn test_dom_tree() {
        // create a child node
        let data = String::from("This is some text");
        let text_node = Node::new_text(data.clone());

        // Set up root node
        let mut attrs = AttrMap::new();
        let test_key = "test_key";
        let test_attr = "test attribute";
        attrs.insert(String::from(test_key), String::from(test_attr));
        let tag_name = String::from("test_elem");
        let node = Node::new_element(tag_name, attrs.clone(), vec![text_node]);

        // confirm ability to access child through parent
        assert!(!node.children.is_empty());
        let child = node.children.get(0).unwrap();
        match &child.node_type {
            NodeType::Text(text) => {
                assert!(text == &data);
            }
            _ => panic!("Expected text node, found unexpected node type"),
        }
    }
}
