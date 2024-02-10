

error_chain! {
    errors {
        EmptyWorkspace {
            description("the workspace is empty")
            display("the workspace is empty")
        }
        MissingScope {
            description("couldn't find any scopes at the cursor position")
            display("couldn't find any scopes at the cursor position")
        }
        MissingSyntax {
            description("no syntax definition for the current buffer")
            display("no syntax definition for the current buffer")
        }
    }

    foreign_links {
        Io(::std::io::Error) #[cfg(unix)];
        ParsingError(syntect::parsing::ParsingError);
        ScopeError(syntect::parsing::ScopeError);
        SyntaxLoadingError(syntect::LoadingError);
    }
}
