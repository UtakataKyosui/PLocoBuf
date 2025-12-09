
use axum::{
    body::Bytes,
    extract::{rejection::BytesRejection, FromRequest, Request},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use prost::Message;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtobufError {
    #[error("ProtoBuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("Missing or invalid Content-Type header")]
    InvalidContentType,
    #[error("Failed to read body: {0}")]
    ReadBody(#[from] BytesRejection),
}

impl IntoResponse for ProtobufError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ProtobufError::Decode(err) => (StatusCode::BAD_REQUEST, format!("Decode error: {}", err)),
            ProtobufError::InvalidContentType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Expected content-type: application/protobuf".to_string(),
            ),
            ProtobufError::ReadBody(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Read body error: {}", err),
            ),
        };
        (status, body).into_response()
    }
}

pub struct Protobuf<T>(pub T);

// #[axum::async_trait] removed for Axum 0.8
impl<T, S> FromRequest<S> for Protobuf<T>
where
    T: Message + Default,
    S: Send + Sync,
{
    type Rejection = ProtobufError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok());

        match content_type {
            Some("application/protobuf") => {
                let bytes = Bytes::from_request(req, state)
                    .await
                    .map_err(ProtobufError::ReadBody)?;
                let value = T::decode(bytes)?;
                Ok(Protobuf(value))
            }
            _ => Err(ProtobufError::InvalidContentType),
        }
    }
}

impl<T> IntoResponse for Protobuf<T>
where
    T: Message,
{
    fn into_response(self) -> Response {
        let mut buf = Vec::new();
        if let Err(e) = self.0.encode(&mut buf) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to encode protobuf: {}", e),
            )
                .into_response();
        }

        (
            [(header::CONTENT_TYPE, "application/protobuf")],
            buf,
        )
            .into_response()
    }
}
