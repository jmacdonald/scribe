error_chain! {
    errors {
        EmptyWorkspace {
            description("the workspace is empty")
            display("the workspace is empty")
        }
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
