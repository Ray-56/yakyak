//! SIP message builder utilities (Simplified version)

use super::message::{SipError, SipRequest, SipResponse};
use rsip::{Header, Headers, Response, StatusCode, Version};

/// Build a simple SIP response from a request
pub struct ResponseBuilder {
    status_code: u16,
    headers: Vec<Header>,
    body: Vec<u8>,
}

impl ResponseBuilder {
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(200)
    }

    pub fn unauthorized() -> Self {
        Self::new(401)
    }

    pub fn server_internal_error() -> Self {
        Self::new(500)
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn header(mut self, header: Header) -> Self {
        self.headers.push(header);
        self
    }

    pub fn build_for_request(mut self, request: &SipRequest) -> Result<SipResponse, SipError> {
        // Copy essential headers from request
        for header in request.headers().iter() {
            match header {
                Header::Via(_) | Header::From(_) | Header::To(_) | Header::CallId(_) | Header::CSeq(_) => {
                    self.headers.push(header.clone());
                }
                _ => {}
            }
        }

        // Add Content-Length
        self.headers.push(Header::ContentLength(
            if self.body.is_empty() {
                "0".into()
            } else {
                self.body.len().to_string().into()
            },
        ));

        let response = Response {
            status_code: StatusCode::from(self.status_code),
            headers: Headers::from(self.headers),
            body: self.body,
            version: Version::V2,
        };

        Ok(SipResponse::new(response))
    }
}

/// Build a simple REGISTER response
pub fn build_register_response(
    request: &SipRequest,
    status_code: u16,
) -> Result<SipResponse, SipError> {
    ResponseBuilder::new(status_code).build_for_request(request)
}
