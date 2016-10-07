use buffer::{Lexeme, Position, Token};
use syntect::parsing::{ParseState, ScopeStack, ScopeStackOp, SyntaxDefinition};
use buffer::token::line_iterator::LineIterator;
use buffer::token::parser::Parser;

pub struct TokenIterator<'a> {
    parser: Parser,
    lines: LineIterator<'a>,
    current_line: Option<&'a str>,
    line_events: Vec<(usize, ScopeStackOp)>,
}

impl<'a> TokenIterator<'a> {
    pub fn new(data: &'a str, syntax: &SyntaxDefinition) -> TokenIterator<'a> {
        let mut token_iterator = TokenIterator{
            parser: Parser::new(syntax),
            lines: LineIterator::new(data),
            current_line: None,
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
                // We want to capture the full scope for a given token, so we
                // need to make sure we apply all of them and only capture it
                // once we've moved on to another token/offset.
                if event_offset > self.parser.position.offset {
                    lexeme = Some(
                        Token::Lexeme(Lexeme{
                            value: &line[self.parser.position.offset..event_offset],
                            scope: self.parser.scope.clone(),
                            position: self.parser.position.clone(),
                        })
                    );
                    self.parser.position.offset = event_offset;
                }

                // Apply the scope and keep a reference to it, so
                // that we can pair it with a token later on.
                self.parser.scope.apply(&scope_change);

                if lexeme.is_some() { return lexeme }
            }

            // We already have discrete variant for newlines,
            // so exclude them when considering content length.
            if let Some(end_of_line) = line.len().checked_sub(1) {
                if self.parser.position.offset < end_of_line {
                    // The rest of the line hasn't triggered a scope
                    // change; categorize it with the last known scope.
                    lexeme = Some(
                        Token::Lexeme(Lexeme{
                            value: &line[self.parser.position.offset..end_of_line],
                            scope: self.parser.scope.clone(),
                            position: self.parser.position.clone(),
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
            self.parser.position = Position{ line: line_number, offset: 0 };
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
    use syntect::parsing::{ScopeStack, SyntaxSet};

    #[test]
    fn token_iterator_returns_correct_tokens() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let def = syntax_set.find_syntax_by_extension("rs");
        let iterator = TokenIterator::new("struct Buffer {\n// comment\n  data: String\n}garbage\n\n", def.unwrap());
        let mut scope_stack = ScopeStack::new();
        let mut expected_tokens = Vec::new();
        scope_stack.push(Scope::new("source.rust").unwrap());
        scope_stack.push(Scope::new("meta.struct.rust").unwrap());
        scope_stack.push(Scope::new("storage.type.struct.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "struct",
            scope: scope_stack.clone(),
            position: Position{ line: 0, offset: 0 }
        }));
        scope_stack.pop();
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: " ",
            scope: scope_stack.clone(),
            position: Position{ line: 0, offset: 6 }
        }));
        scope_stack.push(Scope::new("entity.name.struct.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "Buffer",
            scope: scope_stack.clone(),
            position: Position{ line: 0, offset: 7 }
        }));
        scope_stack.pop();
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: " ",
            scope: scope_stack.clone(),
            position: Position{ line: 0, offset: 13 }
        }));
        scope_stack.push(Scope::new("meta.block.rust").unwrap());
        scope_stack.push(Scope::new("punctuation.definition.block.begin.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "{",
            scope: scope_stack.clone(),
            position: Position{ line: 0, offset: 14 }
        }));
        expected_tokens.push(Token::Newline);
        scope_stack.pop();
        scope_stack.push(Scope::new("comment.line.double-slash.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "// comment",
            scope: scope_stack.clone(),
            position: Position{ line: 1, offset: 0 }
        }));
        expected_tokens.push(Token::Newline);
        scope_stack.pop();
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "  ",
            scope: scope_stack.clone(),
            position: Position{ line: 2, offset: 0 }
        }));
        scope_stack.push(Scope::new("variable.other.property.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "data",
            scope: scope_stack.clone(),
            position: Position{ line: 2, offset: 2 }
        }));
        scope_stack.pop();
        scope_stack.push(Scope::new("punctuation.separator.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: ":",
            scope: scope_stack.clone(),
            position: Position{ line: 2, offset: 6 }
        }));
        scope_stack.pop();
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: " String",
            scope: scope_stack.clone(),
            position: Position{ line: 2, offset: 7 }
        }));
        expected_tokens.push(Token::Newline);
        scope_stack.push(Scope::new("punctuation.definition.block.end.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "}",
            scope: scope_stack.clone(),
            position: Position{ line: 3, offset: 0 }
        }));
        scope_stack.pop();
        scope_stack.pop();
        scope_stack.pop();
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "garbage",
            scope: scope_stack.clone(),
            position: Position{ line: 3, offset: 1 }
        }));
        expected_tokens.push(Token::Newline);
        expected_tokens.push(Token::Newline);
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
        let mut expected_tokens = Vec::new();
        expected_tokens.push(
            Token::Lexeme(Lexeme{
                value: "struct",
                scope: ScopeStack::from_vec(vec![
                    Scope::new("source.rust").unwrap(),
                    Scope::new("storage.type.struct.rust").unwrap()
                ]),
                position: Position{ line: 0, offset: 0 }
            })
        );
        let actual_tokens: Vec<Token> = iterator.collect();
        for (index, token) in expected_tokens.into_iter().enumerate() {
            assert_eq!(token, actual_tokens[index]);
        }

        //assert_eq!(expected_tokens, actual_tokens);
    }
}
