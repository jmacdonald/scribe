use buffer::{Lexeme, Position, Token};
use syntect::parsing::{ParseState, ScopeStack, ScopeStackOp, SyntaxDefinition};
use buffer::token::line_iterator::LineIterator;

pub struct TokenIterator<'a> {
    scopes: ScopeStack,
    parser: ParseState,
    lines: LineIterator<'a>,
    current_line: Option<&'a str>,
    current_position: Position,
    line_events: Vec<(usize, ScopeStackOp)>,
}

impl<'a> TokenIterator<'a> {
    pub fn new(data: &'a str, def: &SyntaxDefinition) -> TokenIterator<'a> {
        let mut token_iterator = TokenIterator{
            scopes: ScopeStack::new(),
            parser: ParseState::new(def),
            lines: LineIterator::new(data),
            current_line: None,
            current_position: Position{ line: 0, offset: 0 },
            line_events: Vec::new(),
        };

        // Preload the first line
        token_iterator.parse_next_line();

        token_iterator
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        // Try to fetch a token from the current line.
        if let Some(token) = self.build_next_token() {
            return Some(token)
        }

        // We're done with this line; on to the next.
        self.parse_next_line();
        if self.current_line.is_some() {
            Some(Token::Newline)
        } else {
            None
        }
    }

    fn build_next_token(&mut self) -> Option<Token<'a>> {
        let mut lexeme = None;

        if let Some(line) = self.current_line {
            while let Some((event_offset, scope_change)) = self.line_events.pop() {
                // We only want to capture the deepest scope for a given token,
                // so we apply all of them and only capture once we move on to
                // another token/offset.
                if event_offset > self.current_position.offset {
                    lexeme = Some(
                        Token::Lexeme(Lexeme{
                            value: &line[self.current_position.offset..event_offset],
                            scope: self.scopes.as_slice().last().map(|s| s.clone()),
                            position: self.current_position.clone(),
                        })
                    );
                    self.current_position.offset = event_offset;
                }

                // Apply the scope and keep a reference to it, so
                // that we can pair it with a token later on.
                self.scopes.apply(&scope_change);

                if lexeme.is_some() { return lexeme }
            }

            // We already have discrete variant for newlines,
            // so exclude them when considering content length.
            if let Some(end_of_line) = line.len().checked_sub(1) {
                if self.current_position.offset < end_of_line {
                    // The rest of the line hasn't triggered a scope
                    // change; categorize it with the last known scope.
                    lexeme = Some(
                        Token::Lexeme(Lexeme{
                            value: &line[self.current_position.offset..end_of_line],
                            scope: self.scopes.as_slice().last().map(|s| s.clone()),
                            position: self.current_position.clone(),
                        })
                    );
                }

            };
        }
        self.current_line = None;

        if lexeme.is_some() {
            lexeme
        } else {
            None
        }
    }

    fn parse_next_line(&mut self) {
        if let Some((line_number, line)) = self.lines.next() {
            // We reverse the line elements so that we can pop them off one at a
            // time, handling each event while allowing us to stop at any point.
            let mut line_events = self.parser.parse_line(line);
            line_events.reverse();
            self.line_events = line_events;

            // Keep a reference to the line so that we can create slices of it.
            self.current_line = Some(line);

            // Track our position, which we'll pass to generated tokens.
            self.current_position = Position{ line: line_number, offset: 0 };
        } else {
            self.current_line = None;
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::TokenIterator;
    use buffer::{Lexeme, Position, Scope, Token};
    use syntect::parsing::SyntaxSet;

    #[test]
    fn token_iterator_returns_correct_tokens() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let def = syntax_set.find_syntax_by_extension("rs");
        let iterator = TokenIterator::new("struct Buffer {\n// comment\n  data: String\n}garbage\n\n", def.unwrap());
        let expected_tokens = vec![
            Token::Lexeme(Lexeme{
                value: "struct",
                scope: Some(Scope::new("storage.type.struct.rust").unwrap()),
                position: Position{ line: 0, offset: 0 }
            }),
            Token::Lexeme(Lexeme{
                value: " ",
                scope: Some(Scope::new("meta.struct.rust").unwrap()),
                position: Position{ line: 0, offset: 6 }
            }),
            Token::Lexeme(Lexeme{
                value: "Buffer",
                scope: Some(Scope::new("entity.name.struct.rust").unwrap()),
                position: Position{ line: 0, offset: 7 }
            }),
            Token::Lexeme(Lexeme{
                value: " ",
                scope: Some(Scope::new("meta.struct.rust").unwrap()),
                position: Position{ line: 0, offset: 13 }
            }),
            Token::Lexeme(Lexeme{
                value: "{",
                scope: Some(Scope::new("punctuation.definition.block.begin.rust").unwrap()),
                position: Position{ line: 0, offset: 14 }
            }),
            Token::Newline,
            Token::Lexeme(Lexeme{
                value: "// comment",
                scope: Some(Scope::new("comment.line.double-slash.rust").unwrap()),
                position: Position{ line: 1, offset: 0 }
            }),
            Token::Newline,
            Token::Lexeme(Lexeme{
                value: "  ",
                scope: Some(Scope::new("meta.block.rust").unwrap()),
                position: Position{ line: 2, offset: 0 }
            }),
            Token::Lexeme(Lexeme{
                value: "data",
                scope: Some(Scope::new("variable.other.property.rust").unwrap()),
                position: Position{ line: 2, offset: 2 }
            }),
            Token::Lexeme(Lexeme{
                value: ":",
                scope: Some(Scope::new("punctuation.separator.rust").unwrap()),
                position: Position{ line: 2, offset: 6 }
            }),
            Token::Lexeme(Lexeme{
                value: " String",
                scope: Some(Scope::new("meta.block.rust").unwrap()),
                position: Position{ line: 2, offset: 7 }
            }),
            Token::Newline,
            Token::Lexeme(Lexeme{
                value: "}",
                scope: Some(Scope::new("punctuation.definition.block.end.rust").unwrap()),
                position: Position{ line: 3, offset: 0 }
            }),
            Token::Lexeme(Lexeme{
                value: "garbage",
                scope: Some(Scope::new("source.rust").unwrap()),
                position: Position{ line: 3, offset: 1 }
            }),
            Token::Newline,
            Token::Newline
        ];
        let actual_tokens: Vec<Token> = iterator.collect();
        for (index, token) in expected_tokens.into_iter().enumerate() {
            assert_eq!(token, actual_tokens[index]);
        }

        //assert_eq!(expected_tokens, actual_tokens);
    }

    #[test]
    fn token_iterator_handles_content_without_trailing_newline() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let def = syntax_set.find_syntax_by_extension("rs");
        let iterator = TokenIterator::new("struct", def.unwrap());
        let expected_tokens = vec![
            Token::Lexeme(Lexeme{
                value: "struct",
                scope: Some(Scope::new("storage.type.struct.rust").unwrap()),
                position: Position{ line: 0, offset: 0 }
            })
        ];
        let actual_tokens: Vec<Token> = iterator.collect();
        for (index, token) in expected_tokens.into_iter().enumerate() {
            assert_eq!(token, actual_tokens[index]);
        }

        //assert_eq!(expected_tokens, actual_tokens);
    }
}
