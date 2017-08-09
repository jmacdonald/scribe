error_chain! {
    errors {
        MissingSyntaxDefinition {
            description("buffer is missing a syntax definition")
            display("buffer is missing a syntax definition")
        }
        MissingScope {
            description("couldn't find any scopes at the cursor position")
            display("couldn't find any scopes at the cursor position")
        }
    }
}
