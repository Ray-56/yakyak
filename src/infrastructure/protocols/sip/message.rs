//! SIP message types and parsing

use bytes::Bytes;
use rsip::{Header, Headers, Method, Request, Response, Uri};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SipError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<rsip::Error> for SipError {
    fn from(err: rsip::Error) -> Self {
        SipError::ParseError(err.to_string())
    }
}

/// SIP method types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SipMethod {
    Register,
    Invite,
    Ack,
    Cancel,
    Bye,
    Options,
    Info,
    Update,
    Prack,
    Subscribe,
    Notify,
    Refer,
    Message,
    Publish,
}

impl SipMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            SipMethod::Register => "REGISTER",
            SipMethod::Invite => "INVITE",
            SipMethod::Ack => "ACK",
            SipMethod::Cancel => "CANCEL",
            SipMethod::Bye => "BYE",
            SipMethod::Options => "OPTIONS",
            SipMethod::Info => "INFO",
            SipMethod::Update => "UPDATE",
            SipMethod::Prack => "PRACK",
            SipMethod::Subscribe => "SUBSCRIBE",
            SipMethod::Notify => "NOTIFY",
            SipMethod::Refer => "REFER",
            SipMethod::Message => "MESSAGE",
            SipMethod::Publish => "PUBLISH",
        }
    }

    pub fn from_rsip(method: &Method) -> Option<Self> {
        match method {
            Method::Register => Some(SipMethod::Register),
            Method::Invite => Some(SipMethod::Invite),
            Method::Ack => Some(SipMethod::Ack),
            Method::Cancel => Some(SipMethod::Cancel),
            Method::Bye => Some(SipMethod::Bye),
            Method::Options => Some(SipMethod::Options),
            _ => None, // Handle other methods as needed
        }
    }

    pub fn to_rsip(&self) -> Method {
        match self {
            SipMethod::Register => Method::Register,
            SipMethod::Invite => Method::Invite,
            SipMethod::Ack => Method::Ack,
            SipMethod::Cancel => Method::Cancel,
            SipMethod::Bye => Method::Bye,
            SipMethod::Options => Method::Options,
            _ => Method::Options, // Default fallback
        }
    }
}

impl fmt::Display for SipMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// SIP Request wrapper
#[derive(Debug, Clone)]
pub struct SipRequest {
    pub inner: Request,
}

impl SipRequest {
    pub fn new(inner: Request) -> Self {
        Self { inner }
    }

    pub fn parse(data: &[u8]) -> Result<Self, SipError> {
        let request = rsip::Request::try_from(data)?;
        Ok(Self::new(request))
    }

    pub fn method(&self) -> Option<SipMethod> {
        SipMethod::from_rsip(&self.inner.method)
    }

    pub fn uri(&self) -> &Uri {
        &self.inner.uri
    }

    pub fn headers(&self) -> &Headers {
        &self.inner.headers
    }

    pub fn body(&self) -> &[u8] {
        &self.inner.body
    }

    pub fn call_id(&self) -> Option<String> {
        self.inner
            .headers
            .iter()
            .find_map(|h| match h {
                Header::CallId(cid) => {
                    // rsip's CallId .to_string() includes "Call-ID: " prefix
                    let s = cid.to_string();
                    s.strip_prefix("Call-ID: ").map(|v| v.to_string()).or(Some(s))
                }
                _ => None,
            })
    }

    pub fn from_tag(&self) -> Option<String> {
        // Simplified - TODO: implement proper tag parsing
        None
    }

    pub fn to_tag(&self) -> Option<String> {
        // Simplified - TODO: implement proper tag parsing
        None
    }

    pub fn cseq(&self) -> Option<u32> {
        self.inner
            .headers
            .iter()
            .find_map(|h| match h {
                Header::CSeq(cseq) => {
                    // seq() returns Result, so we need to unwrap it
                    cseq.seq().ok().and_then(|s| s.to_string().parse().ok())
                }
                _ => None,
            })
    }

    pub fn to_bytes(&self) -> Bytes {
        Bytes::from(self.inner.to_string())
    }
}

/// SIP Response wrapper
#[derive(Debug, Clone)]
pub struct SipResponse {
    pub inner: Response,
}

impl SipResponse {
    pub fn new(inner: Response) -> Self {
        Self { inner }
    }

    pub fn parse(data: &[u8]) -> Result<Self, SipError> {
        let response = rsip::Response::try_from(data)?;
        Ok(Self::new(response))
    }

    pub fn status_code(&self) -> u16 {
        self.inner.status_code.clone().into()
    }

    pub fn headers(&self) -> &Headers {
        &self.inner.headers
    }

    pub fn body(&self) -> &[u8] {
        &self.inner.body
    }

    pub fn to_bytes(&self) -> Bytes {
        Bytes::from(self.inner.to_string())
    }
}

/// SIP Message (either request or response)
#[derive(Debug, Clone)]
pub enum SipMessage {
    Request(SipRequest),
    Response(SipResponse),
}

impl SipMessage {
    pub fn parse(data: &[u8]) -> Result<Self, SipError> {
        // Try parsing as request first
        if let Ok(request) = SipRequest::parse(data) {
            return Ok(SipMessage::Request(request));
        }

        // Try parsing as response
        if let Ok(response) = SipResponse::parse(data) {
            return Ok(SipMessage::Response(response));
        }

        Err(SipError::ParseError(
            "Could not parse as SIP request or response".to_string(),
        ))
    }

    pub fn is_request(&self) -> bool {
        matches!(self, SipMessage::Request(_))
    }

    pub fn is_response(&self) -> bool {
        matches!(self, SipMessage::Response(_))
    }

    pub fn as_request(&self) -> Option<&SipRequest> {
        match self {
            SipMessage::Request(req) => Some(req),
            _ => None,
        }
    }

    pub fn as_response(&self) -> Option<&SipResponse> {
        match self {
            SipMessage::Response(resp) => Some(resp),
            _ => None,
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        match self {
            SipMessage::Request(req) => req.to_bytes(),
            SipMessage::Response(resp) => resp.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_register_request() {
        let data = b"REGISTER sip:registrar.example.com SIP/2.0\r\n\
                     Via: SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds\r\n\
                     From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
                     To: Alice <sip:alice@example.com>\r\n\
                     Call-ID: a84b4c76e66710@pc33.example.com\r\n\
                     CSeq: 314159 REGISTER\r\n\
                     Contact: <sip:alice@192.168.1.100:5060>\r\n\
                     Expires: 3600\r\n\
                     Content-Length: 0\r\n\r\n";

        let msg = SipMessage::parse(data).unwrap();
        assert!(msg.is_request());

        let req = msg.as_request().unwrap();
        assert_eq!(req.method(), Some(SipMethod::Register));
        assert_eq!(req.call_id(), Some("a84b4c76e66710@pc33.example.com".to_string()));
        assert_eq!(req.cseq(), Some(314159));
    }

    #[test]
    fn test_parse_response() {
        let data = b"SIP/2.0 200 OK\r\n\
                     Via: SIP/2.0/UDP 192.168.1.100:5060;branch=z9hG4bK776asdhds\r\n\
                     From: Alice <sip:alice@example.com>;tag=1928301774\r\n\
                     To: Alice <sip:alice@example.com>;tag=a6c85cf\r\n\
                     Call-ID: a84b4c76e66710@pc33.example.com\r\n\
                     CSeq: 314159 REGISTER\r\n\
                     Contact: <sip:alice@192.168.1.100:5060>\r\n\
                     Content-Length: 0\r\n\r\n";

        let msg = SipMessage::parse(data).unwrap();
        assert!(msg.is_response());

        let resp = msg.as_response().unwrap();
        assert_eq!(resp.status_code(), 200);
    }
}
