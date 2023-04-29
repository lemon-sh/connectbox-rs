use reqwest::header::ToStrError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("session token not found, are you logged in?")]
    NoSessionToken,
    #[error("incorrect password")]
    IncorrectCode,
    #[error("unexpected response from the server: {0:?}")]
    UnexpectedResponse(String),

    #[error(transparent)]
    URLParseError(#[from] url::ParseError),
    #[error(transparent)]
    InvalidHeaderValue(#[from] ToStrError),
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
    #[error(transparent)]
    XMLDecodeError(#[from] quick_xml::de::DeError),
}
