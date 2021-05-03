use std::usize;

struct Stylesheet {
    rules: Vec<Rule>,
}

struct Rule {
    /// Currently only supports simple selectors
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

#[derive(Debug)]
/// css seletor used to select dom nodes to apply styles to.
enum Selector {
    /// Can be a tag name, id prefixed with # or class prefixed with .
    /// `*` acts as universal selector
    Simple(SimpleSelector),
}

/// Used to decide which style applies in a conflict
pub type Specificity = (usize, usize, usize);

impl Selector {
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
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
    class: Vec<String>,
}

impl SimpleSelector {
    fn new() -> Self {
        SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        }
    }
}

/// A key value pair separated by a `:`
/// used to specify css properties
struct Declaration {
    name: String,
    value: Value,
}

/// For simplicity, only support a small subset of css values
enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

/// Units for css properties
enum Unit {
    Px,
}

/// Color using rgba values
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct Css {
    pos: usize,
    input: String,
}

impl Css {
    fn next_char(&self) -> char {
        self.input.get(self.pos..).unwrap().chars().next().unwrap()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input.get(self.pos..).unwrap().starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input.get(self.pos..).unwrap().char_indices();
        let (_, curr) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        curr
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

    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector::new();
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
                    // universal selector
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    // fn parse_rule(&mut self) -> Rule {
    //     Rule {
    //         selectors: self.parse_selectors(),
    //         declarations: self.parse_declarations(),
    //     }
    // }

    /// Parse comma separated list of selectors
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
                '{' => break, // start of declarations
                c => panic!("Unexpected character: {} in selector list", c),
            }
        }
        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        selectors
    }
}

fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_selector_id() {
        let mut css = Css {
            pos: 0,
            input: String::from("#test_id"),
        };
        let selector = css.parse_simple_selector();
        assert!(&selector.id.unwrap() == "test_id");
    }

    #[test]
    fn test_parse_simple_selector_tag() {
        let mut css = Css {
            pos: 0,
            input: String::from("p"),
        };
        let selector = css.parse_simple_selector();
        assert!(&selector.tag_name.unwrap() == "p");
    }

    #[test]
    fn test_parse_simple_selector_class() {
        let mut css = Css {
            pos: 0,
            input: String::from(".test_class"),
        };
        let selector = css.parse_simple_selector();
        assert!(selector.class.len() == 1);
        let class = selector.class.get(0);
        assert!(class.unwrap() == "test_class");
    }

    #[test]
    fn test_multiple_selector() {
        let mut css = Css {
            pos: 0,
            input: String::from("#test_id, p, .test_class1, .test_class2 {}"),
        };
        let selectors = css.parse_selectors();

        // test 1st selector
        match selectors.get(0).unwrap() {
            Selector::Simple(s1) => {
                assert!(s1.id.as_ref().unwrap() == "test_id");
            }
        }
        // test 2nd selector
        match selectors.get(1).unwrap() {
            Selector::Simple(s2) => {
                assert!(s2.class.get(0).unwrap() == "test_class1");
            }
        }
        // test 3rd selector
        match selectors.get(2).unwrap() {
            Selector::Simple(s3) => {
                assert!(s3.class.get(0).unwrap() == "test_class2");
            }
        }
        // test 4th selector
        match selectors.get(3).unwrap() {
            Selector::Simple(s4) => {
                assert!(s4.tag_name.as_ref().unwrap() == "p");
            }
        }
    }
}
