use crate::buffer::token::TokenIterator;
use crate::errors::*;
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub struct TokenSet<'a> {
    data: String,
    syntax_definition: &'a SyntaxReference,
    syntaxes: &'a SyntaxSet,
}

impl<'a> TokenSet<'a> {
    pub fn new(data: String, def: &'a SyntaxReference, syntaxes: &'a SyntaxSet) -> TokenSet<'a> {
        TokenSet {
            data,
            syntax_definition: def,
            syntaxes,
        }
    }

    pub fn iter(&self) -> Result<TokenIterator> {
        TokenIterator::new(&self.data, self.syntax_definition, self.syntaxes)
    }
}
