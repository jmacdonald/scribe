use std::cmp;
use buffer::{Lexeme, Position, Token};
use syntect::parsing::{ParseState, ScopeStack, ScopeStackOp, SyntaxReference, SyntaxSet};
use util::LineIterator;
use unicode_segmentation::UnicodeSegmentation;

pub struct TokenIterator<'a> {
    scopes: ScopeStack,
    parser: ParseState,
    lines: LineIterator<'a>,
    current_line: Option<&'a str>,
    current_byte_offset: usize,
    current_position: Position,
    line_events: Vec<(usize, ScopeStackOp)>,
    syntax_set: &'a SyntaxSet,
}

impl<'a> TokenIterator<'a> {
    pub fn new(data: &'a str, syntax_ref: &SyntaxReference, syntax_set: &'a SyntaxSet) -> TokenIterator<'a> {
        let mut token_iterator = TokenIterator{
            scopes: ScopeStack::new(),
            parser: ParseState::new(syntax_ref),
            lines: LineIterator::new(data),
            current_line: None,
            current_byte_offset: 0,
            current_position: Position{ line: 0, offset: 0 },
            line_events: Vec::new(),
            syntax_set,
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
            // Exclude trailing newlines (we have a Newline variant for that).
            let end_of_line = if line.ends_with('\n') {
                line.len() - 1
            } else {
                line.len()
            };

            while let Some((event_offset, scope_change)) = self.line_events.pop() {
                // We want to capture the full scope for a given token, so we
                // need to make sure we apply all of them and only capture it
                // once we've moved on to another token/offset.
                if event_offset > self.current_byte_offset {
                    // Don't include trailing newlines in lexemes.
                    let end_of_token = cmp::min(event_offset, end_of_line);

                    lexeme = Some(
                        Token::Lexeme(Lexeme{
                            value: &line[self.current_byte_offset..end_of_token],
                            scope: self.scopes.clone(),
                            position: self.current_position,
                        })
                    );

                    // The event/current offsets are byte-based, but
                    // position offsets should be grapheme cluster-based.
                    self.current_position.offset +=
                        line[self.current_byte_offset..end_of_token]
                        .graphemes(true)
                        .count();

                    self.current_byte_offset = event_offset;
                }

                // Apply the scope and keep a reference to it, so
                // that we can pair it with a token later on.
                self.scopes.apply(&scope_change);

                if lexeme.is_some() { return lexeme }
            }

            // Categorize the rest of the line with the last known scope.
            if self.current_byte_offset < end_of_line {
                lexeme = Some(
                    Token::Lexeme(Lexeme{
                        value: &line[self.current_byte_offset..end_of_line],
                        scope: self.scopes.clone(),
                        position: self.current_position,
                    })
                );
            }
        }

        // We've finished processing the current line; clean it up.
        self.current_line = None;

        lexeme
    }

    fn parse_next_line(&mut self) {
        if let Some((line_number, line)) = self.lines.next() {
            // We reverse the line elements so that we can pop them off one at a
            // time, handling each event while allowing us to stop at any point.
            let mut line_events = self.parser.parse_line(line, self.syntax_set);
            line_events.reverse();
            self.line_events = line_events;

            // Keep a reference to the line so that we can create slices of it.
            self.current_line = Some(line);

            // Track our position, which we'll pass to generated tokens.
            self.current_position = Position{ line: line_number, offset: 0 };

            // Reset byte-based line offset.
            self.current_byte_offset = 0;
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
    use buffer::{Lexeme, Position, ScopeStack, Token};
    use syntect::parsing::{Scope, SyntaxSet};

    #[test]
    fn token_iterator_returns_correct_tokens() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax_ref = syntax_set.find_syntax_by_extension("rs").unwrap();
        let iterator = TokenIterator::new(
            "struct Buffer {\n// comment\n  data: String\n}garbage\n\n", syntax_ref, &syntax_set);
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
        scope_stack.push(Scope::new("punctuation.section.block.begin.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "{",
            scope: scope_stack.clone(),
            position: Position{ line: 0, offset: 14 }
        }));
        expected_tokens.push(Token::Newline);
        scope_stack.pop();
        scope_stack.push(Scope::new("comment.line.double-slash.rust").unwrap());
        scope_stack.push(Scope::new("punctuation.definition.comment.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "//",
            scope: scope_stack.clone(),
            position: Position{ line: 1, offset: 0 }
        }));
        scope_stack.pop();
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: " comment",
            scope: scope_stack.clone(),
            position: Position{ line: 1, offset: 2 }
        }));
        expected_tokens.push(Token::Newline);
        scope_stack.pop();
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "  ",
            scope: scope_stack.clone(),
            position: Position{ line: 2, offset: 0 }
        }));
        scope_stack.push(Scope::new("variable.other.member.rust").unwrap());
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
        scope_stack.push(Scope::new("punctuation.section.block.end.rust").unwrap());
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

        // It's important to use a plain text lexer so that the last token
        // doesn't introduce a scope change, forcing the EOL handling logic.
        let syntax_ref = syntax_set.find_syntax_plain_text();
        let iterator = TokenIterator::new("struct", syntax_ref, &syntax_set);
        let mut expected_tokens = Vec::new();
        expected_tokens.push(
            Token::Lexeme(Lexeme{
                value: "struct",
                scope: ScopeStack::from_vec(vec![
                    Scope::new("text.plain").unwrap(),
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

    #[test]
    fn token_iterator_handles_unicode_characters() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax_ref = syntax_set.find_syntax_by_extension("rs").unwrap();
        let iterator = TokenIterator::new("€16", syntax_ref, &syntax_set);
        let mut scope_stack = ScopeStack::new();
        let mut expected_tokens = Vec::new();
        scope_stack.push(Scope::new("source.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "€",
            scope: scope_stack.clone(),
            position: Position{ line: 0, offset: 0 }
        }));
        scope_stack.push(Scope::new("constant.numeric.integer.decimal.rust").unwrap());
        expected_tokens.push(Token::Lexeme(Lexeme{
            value: "16",
            scope: scope_stack.clone(),
            position: Position{ line: 0, offset: 1 }
        }));

        let actual_tokens: Vec<Token> = iterator.collect();
        for (index, token) in expected_tokens.into_iter().enumerate() {
            assert_eq!(token, actual_tokens[index]);
        }
    }
}
