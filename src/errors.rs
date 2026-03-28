use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("the workspace is empty")]
    EmptyWorkspace,
    #[error("buffer doesn't have a path")]
    MissingPath,
    #[error("no syntax definition for the current buffer")]
    MissingSyntax,
    #[cfg(unix)]
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    ParsingError(#[from] syntect::parsing::ParsingError),
    #[error(transparent)]
    ScopeError(#[from] syntect::parsing::ScopeError),
    #[error(transparent)]
    SyntaxLoadingError(#[from] syntect::LoadingError),
}
