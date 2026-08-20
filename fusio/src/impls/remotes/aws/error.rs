use thiserror::Error;

use crate::remotes::{aws::sign::AuthorizeError, http::HttpError};

#[derive(Debug, Error)]
pub enum S3Error {
    #[error("http error: {0}")]
    HttpError(#[from] HttpError),
    #[error("authorize error: {0}")]
    AuthorizeError(#[from] AuthorizeError),
    #[error("xml parse error: {0}")]
    XmlParseError(#[from] quick_xml::DeError),
    // quick-xml 0.41 split serialization out of `DeError`.
    #[error("xml serialize error: {0}")]
    XmlSerializeError(#[from] quick_xml::SeError),
}
