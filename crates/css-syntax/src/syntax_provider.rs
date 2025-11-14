use css_syntax_types::{AtRule, Function, Property, SpecLink, Type};

// Define a trait for types that have syntax and spec_link
pub trait SyntaxProvider {
    fn syntax(&self) -> &Option<String>;
    fn spec_link(&self) -> &Option<SpecLink>;
    fn extended_spec_links(&self) -> &Vec<SpecLink>;
}

// Implement the trait for all the types we need
impl SyntaxProvider for Property {
    fn syntax(&self) -> &Option<String> {
        &self.syntax
    }
    fn spec_link(&self) -> &Option<SpecLink> {
        &self.spec_link
    }
    fn extended_spec_links(&self) -> &Vec<SpecLink> {
        &self.extended_spec_links
    }
}

impl SyntaxProvider for AtRule {
    fn syntax(&self) -> &Option<String> {
        &self.syntax
    }
    fn spec_link(&self) -> &Option<SpecLink> {
        &self.spec_link
    }
    fn extended_spec_links(&self) -> &Vec<SpecLink> {
        &self.extended_spec_links
    }
}

impl SyntaxProvider for Function {
    fn syntax(&self) -> &Option<String> {
        &self.syntax
    }
    fn spec_link(&self) -> &Option<SpecLink> {
        &self.spec_link
    }
    fn extended_spec_links(&self) -> &Vec<SpecLink> {
        &self.extended_spec_links
    }
}

impl SyntaxProvider for Type {
    fn syntax(&self) -> &Option<String> {
        &self.syntax
    }
    fn spec_link(&self) -> &Option<SpecLink> {
        &self.spec_link
    }
    fn extended_spec_links(&self) -> &Vec<SpecLink> {
        &self.extended_spec_links
    }
}
