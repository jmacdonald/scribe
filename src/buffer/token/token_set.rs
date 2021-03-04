use syntect::parsing::{SyntaxReference, SyntaxSet};
use buffer::token::TokenIterator;

pub struct TokenSet<'a> {
    data: String,
    syntax_reference: &'a SyntaxReference,
    syntax_set: &'a SyntaxSet,
}

impl<'a> TokenSet<'a> {
    pub fn new(data: String, syntax_ref: &'a SyntaxReference, syntax_set: &'a SyntaxSet) -> TokenSet<'a> {
        TokenSet {
            data,
            syntax_reference: syntax_ref,
            syntax_set,
        }
    }

    pub fn iter(&self) -> TokenIterator {
        TokenIterator::new(&self.data, self.syntax_reference, self.syntax_set)
    }
}
