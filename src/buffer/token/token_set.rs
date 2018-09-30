use syntect::parsing::SyntaxDefinition;
use buffer::token::TokenIterator;

pub struct TokenSet<'a> {
    data: String,
    syntax_definition: &'a SyntaxDefinition,
}

impl<'a> TokenSet<'a> {
    pub fn new(data: String, def: &SyntaxDefinition) -> TokenSet {
        TokenSet{
            data,
            syntax_definition: def
        }
    }

    pub fn iter(&self) -> TokenIterator {
        TokenIterator::new(&self.data, self.syntax_definition)
    }
}
