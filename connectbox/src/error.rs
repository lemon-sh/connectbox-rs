use thiserror::Error;

/// The error type used globally by the library
#[derive(Error, Debug)]
pub enum Error {
    #[error("session token not found, are you logged in?")]
    NoSessionToken,
    #[error("incorrect password")]
    IncorrectPassword,
    #[error("unexpected response from the server: {0:?}")]
    UnexpectedResponse(String),
    #[error("you are not logged in, or perhaps the session has expired")]
    NotAuthorized,
    #[error("access denied, most likely someone else is already logged in")]
    AccessDenied,
    #[error("an unexpected redirection has occurred: {0:?}")]
    UnexpectedRedirect(String),
    #[error("remote error: {0:?}")]
    Remote(String),

    #[error(transparent)]
    URLParseError(#[from] url::ParseError),
    #[error(transparent)]
    InvalidHeaderValue(#[from] reqwest::header::ToStrError),
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
    #[error(transparent)]
    XMLDecodeError(#[from] quick_xml::de::DeError),
}
