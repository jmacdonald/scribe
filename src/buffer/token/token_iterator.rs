use buffer::{Position, Token};
use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxDefinition};
use buffer::token::line_iterator::LineIterator;
use std::vec::IntoIter;

pub struct TokenIterator<'a> {
    scopes: ScopeStack,
    parser: ParseState,
    line_tokens: Option<IntoIter<Token<'a>>>,
    lines: LineIterator<'a>
}

impl<'a> TokenIterator<'a> {
    pub fn new(data: &'a str, def: &SyntaxDefinition) -> TokenIterator<'a> {
        TokenIterator{
            scopes: ScopeStack::new(),
            parser: ParseState::new(def),
            line_tokens: None,
            lines: LineIterator::new(data)
        }
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        // Try to fetch a token from the current line.
        if let Some(ref mut tokens) = self.line_tokens {
            if let Some(token) = tokens.next() {
                return Some(token)
            }
        }

        // We're done with this line; on to the next.
        self.parse_next_line();

        // If this returns none, we're done.
        if let Some(ref mut tokens) = self.line_tokens {
            tokens.next()
        } else {
            None
        }
    }

    fn parse_next_line(&mut self) {
        let mut tokens = Vec::new();
        let mut offset = 0;
        let mut last_scope: Option<Scope> = None;

        if let Some((line_number, line)) = self.lines.next() {
            for (change_offset, scope_change) in self.parser.parse_line(line) {
                // We only want to capture the deepest scope for a given token,
                // so we apply all of them and only capture once we move on to
                // another token/offset.
                if change_offset > offset {
                    if let Some(scope) = last_scope {
                        tokens.push(
                            Token{
                                lexeme: &line[offset..change_offset],
                                scope: scope.clone(),
                                position: Position{
                                    line: line_number,
                                    offset: offset
                                }
                            }
                        );
                        offset = change_offset;
                    }
                }

                // Apply the scope and keep a reference to it, so
                // that we can pair it with a token later on.
                self.scopes.apply(&scope_change);
                last_scope = self.scopes.as_slice().last().map(|s| s.clone());

            }

            // If the rest of the line doesn't trigger a scope
            // change, categorize it with the last known scope.
            if offset < line.len() - 1 {
                if let Some(scope) = last_scope {
                    tokens.push(
                        Token{
                            lexeme: &line[offset..line.len()],
                            scope: scope,
                            position: Position{
                                line: line_number,
                                offset: offset
                            }
                        }
                    );
                }
            }

            self.line_tokens = Some(tokens.into_iter());
        } else {
            self.line_tokens = None;
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.next_token() {
            return Some(token)
        }

        self.parse_next_line();
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::TokenIterator;
    use buffer::{Position, Scope, Token};
    use syntect::parsing::SyntaxSet;

    #[test]
    fn token_iterator_returns_correct_tokens() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let def = syntax_set.find_syntax_by_extension("rs");
        let iterator = TokenIterator::new("struct Buffer {\ndata: String", def.unwrap());
        let expected_tokens = vec![
            Token{
                lexeme: "struct",
                scope: Scope::new("storage.type.struct.rust").unwrap(),
                position: Position{ line: 0, offset: 0 }
            },
            Token{
                lexeme: " ",
                scope: Scope::new("meta.struct.rust").unwrap(),
                position: Position{ line: 0, offset: 6 }
            },
            Token{
                lexeme: "Buffer",
                scope: Scope::new("entity.name.struct.rust").unwrap(),
                position: Position{ line: 0, offset: 7 }
            },
            Token{
                lexeme: " ",
                scope: Scope::new("meta.struct.rust").unwrap(),
                position: Position{ line: 0, offset: 13 }
            },
            Token{
                lexeme: "{",
                scope: Scope::new("punctuation.definition.block.begin.rust").unwrap(),
                position: Position{ line: 0, offset: 14 }
            },
            Token{
                lexeme: "data",
                scope: Scope::new("variable.other.property.rust").unwrap(),
                position: Position{ line: 1, offset: 0 }
            },
            Token{
                lexeme: ":",
                scope: Scope::new("punctuation.separator.rust").unwrap(),
                position: Position{ line: 1, offset: 4 }
            },
            Token{
                lexeme: " String",
                scope: Scope::new("meta.block.rust").unwrap(),
                position: Position{ line: 1, offset: 5 }
            }
        ];
        let actual_tokens: Vec<Token> = iterator.collect();
        for (index, token) in expected_tokens.into_iter().enumerate() {
            assert_eq!(token, actual_tokens[index]);
        }
    }
}
