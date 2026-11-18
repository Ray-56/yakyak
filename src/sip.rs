use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub struct RequestLine {
    pub method: String,
    pub uri: String,
    pub version: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StatusLine {
    pub version: String,
    pub status_code: u16,
    pub reason_phrase: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StartLine {
    Request(RequestLine),
    Response(StatusLine),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    pub fn matches_name(&self, name: &str) -> bool {
        let stored_name = canonical_header_name(&self.name);
        let query_name = canonical_header_name(name);

        stored_name.eq_ignore_ascii_case(query_name)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SipMessage {
    pub start_line: StartLine,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SipUri {
    pub scheme: String,
    pub userinfo: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub uri_params: Vec<(String, Option<String>)>,
    pub uri_headers: Vec<(String, String)>,
}

fn canonical_header_name(name: &str) -> &str {
    if name.eq_ignore_ascii_case("v") {
        "Via"
    } else if name.eq_ignore_ascii_case("i") {
        "Call-ID"
    } else if name.eq_ignore_ascii_case("m") {
        "Contact"
    } else if name.eq_ignore_ascii_case("e") {
        "Content-Encoding"
    } else if name.eq_ignore_ascii_case("l") {
        "Content-Length"
    } else if name.eq_ignore_ascii_case("c") {
        "Content-Type"
    } else if name.eq_ignore_ascii_case("f") {
        "From"
    } else if name.eq_ignore_ascii_case("s") {
        "Subject"
    } else if name.eq_ignore_ascii_case("k") {
        "Supported"
    } else if name.eq_ignore_ascii_case("t") {
        "To"
    } else {
        name
    }
}

fn validate_percent_escapes(text: &str) -> Result<(), String> {
    let mut bytes = text.bytes();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let first = bytes.next();
            let second = bytes.next();

            if !first.is_some_and(|b| b.is_ascii_hexdigit())
                || !second.is_some_and(|b| b.is_ascii_hexdigit())
            {
                return Err(String::from("URI 百分号转义无效"));
            }
        }
    }
    Ok(())
}

fn validate_uri_param_chars(text: &str) -> Result<(), String> {
    validate_percent_escapes(text)?;

    if !text
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"-_.!~*'()[]/:&+$%".contains(&b))
    {
        return Err(String::from("URI 参数包含非法字符"));
    }

    Ok(())
}

fn validate_uri_header_chars(text: &str) -> Result<(), String> {
    validate_percent_escapes(text)?;

    if !text
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"-_.!~*'()[]/?:+$%".contains(&b))
    {
        return Err(String::from("URI header 包含非法字符"));
    }

    Ok(())
}

pub fn parse_sip_uri(input: &str) -> Result<SipUri, String> {
    if input.bytes().any(|b| b == b' ' || b.is_ascii_control()) {
        return Err(String::from("URI 不能包含空白或控制字符"));
    }

    let (scheme, rest) = input
        .split_once(':')
        .ok_or_else(|| "`:` 符号不能为空".to_string())?;

    if scheme.is_empty() {
        return Err(String::from("scheme 不能为空"));
    }

    if !scheme.eq_ignore_ascii_case("sip") && !scheme.eq_ignore_ascii_case("sips") {
        return Err(String::from("不支持的 URI scheme"));
    }

    let mut sipuri = SipUri {
        scheme: scheme.to_string(),
        userinfo: None,
        host: String::new(),
        port: None,
        uri_params: Vec::new(),
        uri_headers: Vec::new(),
    };

    if rest.contains('@') {
        let (userinfo, host) = rest
            .split_once('@')
            .ok_or_else(|| String::from("`@`符号不能为空"))?;
        if userinfo.is_empty() {
            return Err(String::from("有'@'符号时 userinfo 不能为空"));
        }

        validate_percent_escapes(userinfo)?;

        let (user, password) = match userinfo.split_once(':') {
            Some((user, pwd)) => (user, Some(pwd)),
            None => (userinfo, None),
        };

        if user.is_empty() {
            return Err(String::from("URI user 不能为空"));
        }
        if !user
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.!~*'()&=+$,;?/%".contains(&b))
        {
            return Err(String::from("URI user 包含非法字符"));
        }

        if let Some(pwd) = password
            && !pwd
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.!~*'()&=+$,%".contains(&b))
        {
            return Err(String::from("URI password 包含非法字符"));
        }

        sipuri.userinfo = Some(userinfo.to_string());
        sipuri.host = host.to_string();
    } else {
        sipuri.userinfo = None;
        sipuri.host = rest.to_string();
    }

    let (hostport_and_params, headers_text) = match sipuri.host.split_once('?') {
        Some((before, text)) => (before, Some(text)),
        None => (sipuri.host.as_str(), None),
    };

    let uri_headers = match headers_text {
        Some(text) => {
            validate_percent_escapes(text)?;
            text.split('&')
                .map(|item| match item.split_once('=') {
                    Some((name, value)) => {
                        if name.is_empty() {
                            return Err(String::from("header 参数名不能为空"));
                        }
                        validate_uri_header_chars(name)?;
                        validate_uri_header_chars(value)?;
                        Ok((name.to_string(), value.to_string()))
                    }
                    None => Err(String::from("header 必须包含等号")),
                })
                .collect::<Result<Vec<_>, String>>()?
        }
        None => Vec::new(),
    };

    let (hostport, params_text) = match hostport_and_params.split_once(';') {
        Some((hostport, text)) => (hostport, Some(text)),
        None => (hostport_and_params, None),
    };

    if hostport.contains('%') {
        return Err(String::from("host 不能包含百分号转义"));
    }

    let uri_params = match params_text {
        Some(text) => text
            .split(';')
            .map(|item| match item.split_once('=') {
                Some((name, value)) => {
                    if name.is_empty() {
                        return Err(String::from("URI 参数名不能为空"));
                    }
                    validate_uri_param_chars(name)?;
                    if value.is_empty() {
                        return Err(String::from("URI 参数值不能为空"));
                    }

                    if name.eq_ignore_ascii_case("transport")
                        || name.eq_ignore_ascii_case("user")
                        || name.eq_ignore_ascii_case("method")
                    {
                        if !value
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"-.!%*_+`'~".contains(&b))
                        {
                            return Err(String::from("URI token 参数值无效"));
                        }
                    } else if name.eq_ignore_ascii_case("ttl") {
                        if !value.bytes().all(|b| b.is_ascii_digit()) {
                            return Err(String::from("ttl 参数必须是十进制数字"));
                        }
                        if value.len() > 3 {
                            return Err(String::from("ttl 参数必须是 1 到 3 位数字"));
                        }
                        if value.parse::<u8>().is_err() {
                            return Err(String::from("ttl 参数超出 0..255 范围"));
                        }
                    } else if name.eq_ignore_ascii_case("maddr") {
                        if value.starts_with('[') {
                            let address = value
                                .strip_prefix('[')
                                .and_then(|value| value.strip_suffix(']'))
                                .ok_or_else(|| String::from("maddr host 无效"))?;

                            address
                                .parse::<std::net::Ipv6Addr>()
                                .map_err(|_| String::from("maddr host 无效"))?;
                        } else {
                            validate_hostname_or_ipv4(value)
                                .map_err(|_| String::from("maddr host 无效"))?;
                        }
                    } else {
                        validate_uri_param_chars(value)?;
                    }

                    Ok((name.to_string(), Some(value.to_string())))
                }
                None => {
                    if item.is_empty() {
                        return Err(String::from("URI 参数名不能为空"));
                    }
                    validate_uri_param_chars(item)?;
                    if item.eq_ignore_ascii_case("ttl") {
                        return Err(String::from("ttl 参数必须包含值"));
                    } else if item.eq_ignore_ascii_case("maddr") {
                        return Err(String::from("maddr 参数必须包含值"));
                    } else if item.eq_ignore_ascii_case("transport") {
                        return Err(String::from("transport 参数必须包含值"));
                    } else if item.eq_ignore_ascii_case("user") {
                        return Err(String::from("user 参数必须包含值"));
                    } else if item.eq_ignore_ascii_case("method") {
                        return Err(String::from("method 参数必须包含值"));
                    }
                    Ok((item.to_string(), None))
                }
            })
            .collect::<Result<Vec<_>, String>>()?,
        None => Vec::new(),
    };

    let mut seen = HashSet::new();

    for (name, _) in &uri_params {
        if !seen.insert(name.to_ascii_lowercase()) {
            return Err(String::from("URI 参数名不能重复"));
        }
    }

    let (host, port_text) = if hostport.starts_with('[') {
        let close = hostport
            .find(']')
            .ok_or_else(|| String::from("IPv6 缺少闭括号"))?;

        hostport[1..close]
            .parse::<std::net::Ipv6Addr>()
            .map_err(|_| String::from("IPv6 地址无效"))?;

        let (host, tail) = hostport.split_at(close + 1);

        let port_text = if tail.is_empty() {
            None
        } else {
            Some(
                tail.strip_prefix(':')
                    .ok_or_else(|| String::from("IPv6 闭括号只能接端口"))?,
            )
        };
        (host, port_text)
    } else if let Some((host, port_text)) = hostport.split_once(':') {
        (host, Some(port_text))
    } else {
        (hostport, None)
    };

    let port = if let Some(port_text) = port_text {
        if port_text.is_empty() || !port_text.bytes().all(|b| b.is_ascii_digit()) {
            return Err(String::from("port 必须是十进制数字"));
        }

        Some(
            port_text
                .parse::<u16>()
                .map_err(|_| String::from("port 超出 u16 范围"))?,
        )
    } else {
        None
    };

    if host.is_empty() {
        return Err(String::from("host 不能为空"));
    }
    if !host.starts_with('[') {
        validate_hostname_or_ipv4(host)?;
    }

    sipuri.host = host.to_string();
    sipuri.port = port;
    sipuri.uri_params = uri_params;
    sipuri.uri_headers = uri_headers;

    Ok(sipuri)
}

fn validate_hostname_or_ipv4(host: &str) -> Result<(), String> {
    let has_only_valid_chars = host
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"-.".contains(&b));

    let host_for_labels = host.strip_suffix('.').unwrap_or(host);

    let has_invalid_label = host_for_labels
        .split('.')
        .any(|label| label.is_empty() || label.starts_with('-') || label.ends_with('-'));

    if !has_only_valid_chars || has_invalid_label {
        return Err(String::from("hostname 无效"));
    }

    let is_ipv4 = host.parse::<std::net::Ipv4Addr>().is_ok();

    if !is_ipv4 {
        let top_label = host_for_labels.rsplit('.').next().unwrap_or("");

        if !top_label.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
            return Err(String::from("hostname 无效"));
        }
    }
    Ok(())
}

fn parse_request_line(input: &str) -> Result<RequestLine, String> {
    // req success "INVITE sip:bob@example.com SIP/2.0\r\n"
    // req failed "INVITE SIP/2.0\r\n"
    let line = input
        .strip_suffix("\r\n")
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "请求行必须以 CRLF 结尾".to_string())?;

    if line.contains('\r') || line.contains('\n') || line.contains('\t') {
        return Err("请求行不能包含回车、换行或制表符".to_string());
    }

    let mut parts = line.split(" ").filter(|v| !v.is_empty());
    let method = parts.next().ok_or_else(|| "method 不存在".to_string())?;
    let uri = parts.next().ok_or_else(|| "uri 不存在".to_string())?;
    let version = parts.next().ok_or_else(|| "version 不存在".to_string())?;
    if parts.next().is_some() {
        return Err("参数超过三个".to_string());
    }

    Ok(RequestLine {
        method: method.to_string(),
        uri: uri.to_string(),
        version: version.to_string(),
    })
}

fn parse_status_line(input: &str) -> Result<StatusLine, String> {
    // "SIP/2.0 486 Busy Here\r\n"
    let line = input
        .strip_suffix("\r\n")
        .ok_or_else(|| String::from("状态行必须以 CRLF 结尾"))?;

    if line.contains('\r') || line.contains('\n') {
        return Err("状态行不能包含回车、换行".to_string());
    }

    let mut parts = line.split(" ");
    let version = parts
        .next()
        .ok_or_else(|| "version 不存在".to_string())?
        .to_string();
    if version.contains('\t') {
        return Err("version 不能包含制表符".to_string());
    }
    let code = parts
        .next()
        .ok_or_else(|| "status_code 不存在".to_string())?;
    if code.contains('\t') {
        return Err("status_code 不能包含制表符".to_string());
    }
    let status_code = code
        .parse::<u16>()
        .map_err(|_| "status_code 无法转换为 u16".to_string())?;
    let reason_phrase = parts.collect::<Vec<_>>().join(" ");

    Ok(StatusLine {
        version,
        status_code,
        reason_phrase,
    })
}

fn parse_start_line(input: &str) -> Result<StartLine, String> {
    let is_response = input
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("SIP/"));

    if is_response {
        let status_line = parse_status_line(input)?;
        let start_line = StartLine::Response(status_line);
        Ok(start_line)
    } else {
        let request_line = parse_request_line(input)?;
        let start_line = StartLine::Request(request_line);
        Ok(start_line)
    }
}

fn parse_header(input: &str) -> Result<Header, String> {
    let line = input
        .strip_suffix("\r\n")
        .ok_or_else(|| "头字段必须以 CRLF 结尾".to_string())?;

    if line.is_empty() {
        return Err(String::from("头字段不能为空"));
    }

    let (name, value) = match line.split_once(':') {
        Some((name, value)) => (name, value),
        None => return Err(String::from("头字段缺少 ':' 分隔符")),
    };

    let name = name.trim_matches([' ', '\t']).to_string();
    let value = value.trim_matches([' ', '\t']).to_string();

    if name.is_empty() {
        return Err(String::from("头字段名不能为空"));
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"-.!%*_+`'~".contains(&b))
    {
        return Err(String::from("头字段名包含非法字符"));
    }

    if value.contains(['\0', '\r', '\n']) {
        return Err(String::from("头字段值包含非法控制字符"));
    }

    Ok(Header { name, value })
}

fn parse_headers(input: &str) -> Result<Vec<Header>, String> {
    let mut headers = Vec::new();
    let mut current = String::new();

    for line in input.split_inclusive("\r\n") {
        let text = line
            .strip_suffix("\r\n")
            .ok_or_else(|| String::from("头字段必须以 CRLF 结尾"))?;

        if text.is_empty() {
            return Err(String::from("头字段不能为空"));
        }

        if text.starts_with(' ') || text.starts_with('\t') {
            // 续行：追加到当前头字段
            if current.is_empty() {
                return Err(String::from("续行前缺少头字段"));
            }

            current.push(' ');
            current.push_str(text.trim_start_matches([' ', '\t']));
        } else {
            // 遇到新头：先保存上一条。
            if !current.is_empty() {
                current.push_str("\r\n");
                headers.push(parse_header(&current)?);
                current.clear();
            }

            // 名称和冒号必须在第一行中
            if !text.contains(':') {
                return Err(String::from("头字段缺少 ':' 分隔符"));
            }

            current.push_str(text);
        }
    }

    // 这里保存最后一条
    if !current.is_empty() {
        current.push_str("\r\n");
        headers.push(parse_header(&current)?);
    }

    Ok(headers)
}

fn split_sip_head_and_body(input: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let pos = input
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .ok_or_else(|| String::from("SIP 消息缺少头部结束空行"))?;

    let head = &input[..pos + 2];
    let body = &input[pos + 4..];

    Ok((head, body))
}

pub fn parse_sip_message(input: &[u8]) -> Result<SipMessage, String> {
    let (head, body) = split_sip_head_and_body(input)?;
    let pos = head
        .windows(2)
        .position(|part| part == b"\r\n")
        .ok_or_else(|| String::from("SIP 起始行缺少 CRLF 结束符"))?;

    let start_line = std::str::from_utf8(&head[..pos + 2])
        .map_err(|_| String::from("SIP 起始行不是有效 UTF-8"))?;
    let heads = std::str::from_utf8(&head[pos + 2..])
        .map_err(|_| String::from("SIP 头字段不是有效 UTF-8"))?;

    Ok(SipMessage {
        start_line: parse_start_line(start_line)?,
        headers: parse_headers(heads)?,
        body: body.to_vec(),
    })
}

fn serialize_start_line(line: &StartLine) -> String {
    match line {
        StartLine::Request(request) => {
            format!("{} {} {}\r\n", request.method, request.uri, request.version)
        }
        StartLine::Response(response) => format!(
            "{} {} {}\r\n",
            response.version, response.status_code, response.reason_phrase
        ),
    }
}

fn serialize_headers(headers: &[Header]) -> String {
    let mut rst = String::new();
    for h in headers {
        rst.push_str(&format!("{}: {}\r\n", h.name, h.value));
    }

    rst
}

pub fn serialize_sip_message(message: &SipMessage) -> Vec<u8> {
    let mut start_line = serialize_start_line(&message.start_line);
    start_line.push_str(&serialize_headers(&message.headers));
    start_line.push_str("\r\n");

    let mut rst = start_line.into_bytes();
    rst.extend_from_slice(&message.body);

    rst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_serialize_headers_with_whitespace() {
        let input = concat!(
            "X-Trace-ID \t: \tAbC\r\n",
            " \t123\r\n",
            "X-Trace-ID: DeF\r\n",
        );
        let headers = parse_headers(input).unwrap();

        let expected = concat!("X-Trace-ID: AbC 123\r\n", "X-Trace-ID: DeF\r\n",);
        assert_eq!(serialize_headers(&headers), expected);
        assert_eq!(
            headers,
            vec![
                Header {
                    name: String::from("X-Trace-ID"),
                    value: String::from("AbC 123")
                },
                Header {
                    name: String::from("X-Trace-ID"),
                    value: String::from("DeF")
                }
            ]
        );
    }

    #[test]
    fn parse_and_serialize_extension_method() {
        let input = "X-PING sip:bob@example.com SIP/2.0\r\n";
        let line = parse_start_line(input).unwrap();

        assert_eq!(
            line,
            StartLine::Request(RequestLine {
                method: String::from("X-PING"),
                uri: String::from("sip:bob@example.com"),
                version: String::from("SIP/2.0")
            })
        );
        assert_eq!(serialize_start_line(&line), input);
    }

    #[test]
    fn parse_and_serialize_provisional_response() {
        let input = "SIP/2.0 183 Session Progress\r\n";
        let line = parse_start_line(input).unwrap();

        assert_eq!(
            line,
            StartLine::Response(StatusLine {
                version: String::from("SIP/2.0"),
                status_code: 183,
                reason_phrase: String::from("Session Progress")
            })
        );
        assert_eq!(serialize_start_line(&line), input);
    }

    #[test]
    fn serialize_sip_message_preserves_binary_body() {
        let body = b"\xff\0\r\n\r\n";
        let sip_message = SipMessage {
            start_line: StartLine::Response(StatusLine {
                version: String::from("SIP/2.0"),
                status_code: 200,
                reason_phrase: String::from("OK"),
            }),
            headers: vec![
                Header {
                    name: String::from("Via"),
                    value: String::from("SIP/2.0/UDP client.example.com;branch=z9hG4bK-test"),
                },
                Header {
                    name: String::from("From"),
                    value: String::from("<sip:alice@example.com>;tag=123"),
                },
                Header {
                    name: String::from("To"),
                    value: String::from("<sip:bob@example.com>;tag=456"),
                },
                Header {
                    name: String::from("Call-ID"),
                    value: String::from("test@example.com"),
                },
                Header {
                    name: String::from("CSeq"),
                    value: String::from("1 OPTIONS"),
                },
                Header {
                    name: String::from("Content-Type"),
                    value: String::from("application/octet-stream"),
                },
                Header {
                    name: String::from("Content-Length"),
                    value: String::from("6"),
                },
            ],
            body: body.to_vec(),
        };
        let mut expected = concat!(
            "SIP/2.0 200 OK\r\n",
            "Via: SIP/2.0/UDP client.example.com;branch=z9hG4bK-test\r\n",
            "From: <sip:alice@example.com>;tag=123\r\n",
            "To: <sip:bob@example.com>;tag=456\r\n",
            "Call-ID: test@example.com\r\n",
            "CSeq: 1 OPTIONS\r\n",
            "Content-Type: application/octet-stream\r\n",
            "Content-Length: 6\r\n",
            "\r\n",
        )
        .as_bytes()
        .to_vec();
        expected.extend_from_slice(body);

        assert_eq!(serialize_sip_message(&sip_message), expected)
    }

    #[test]
    fn serialize_sip_message_with_response_and_body() {
        let sip_message = SipMessage {
            start_line: StartLine::Response(StatusLine {
                version: String::from("SIP/2.0"),
                status_code: 200,
                reason_phrase: String::from("OK"),
            }),
            headers: vec![
                Header {
                    name: String::from("Via"),
                    value: String::from("SIP/2.0/UDP client.example.com;branch=z9hG4bK-test"),
                },
                Header {
                    name: String::from("From"),
                    value: String::from("<sip:alice@example.com>;tag=123"),
                },
                Header {
                    name: String::from("To"),
                    value: String::from("<sip:bob@example.com>;tag=456"),
                },
                Header {
                    name: String::from("Call-ID"),
                    value: String::from("test@example.com"),
                },
                Header {
                    name: String::from("CSeq"),
                    value: String::from("1 OPTIONS"),
                },
                Header {
                    name: String::from("Content-Type"),
                    value: String::from("text/plain"),
                },
                Header {
                    name: String::from("Content-Length"),
                    value: String::from("5"),
                },
            ],
            body: b"hello".to_vec(),
        };
        let expected = concat!(
            "SIP/2.0 200 OK\r\n",
            "Via: SIP/2.0/UDP client.example.com;branch=z9hG4bK-test\r\n",
            "From: <sip:alice@example.com>;tag=123\r\n",
            "To: <sip:bob@example.com>;tag=456\r\n",
            "Call-ID: test@example.com\r\n",
            "CSeq: 1 OPTIONS\r\n",
            "Content-Type: text/plain\r\n",
            "Content-Length: 5\r\n",
            "\r\n",
            "hello",
        );

        assert_eq!(serialize_sip_message(&sip_message), expected.as_bytes());
    }

    #[test]
    fn serialize_sip_message_without_body() {
        let sip_message = SipMessage {
            start_line: StartLine::Request(RequestLine {
                method: String::from("OPTIONS"),
                uri: String::from("sip:bob@example.com"),
                version: String::from("SIP/2.0"),
            }),
            headers: vec![
                Header {
                    name: String::from("Via"),
                    value: String::from("SIP/2.0/UDP client.example.com;branch=z9hG4bK-test"),
                },
                Header {
                    name: String::from("Max-Forwards"),
                    value: String::from("70"),
                },
                Header {
                    name: String::from("From"),
                    value: String::from("<sip:alice@example.com>;tag=123"),
                },
                Header {
                    name: String::from("To"),
                    value: String::from("<sip:bob@example.com>"),
                },
                Header {
                    name: String::from("Call-ID"),
                    value: String::from("test@example.com"),
                },
                Header {
                    name: String::from("CSeq"),
                    value: String::from("1 OPTIONS"),
                },
                Header {
                    name: String::from("Content-Length"),
                    value: String::from("0"),
                },
            ],
            body: Vec::new(),
        };
        let expected = concat!(
            "OPTIONS sip:bob@example.com SIP/2.0\r\n",
            "Via: SIP/2.0/UDP client.example.com;branch=z9hG4bK-test\r\n",
            "Max-Forwards: 70\r\n",
            "From: <sip:alice@example.com>;tag=123\r\n",
            "To: <sip:bob@example.com>\r\n",
            "Call-ID: test@example.com\r\n",
            "CSeq: 1 OPTIONS\r\n",
            "Content-Length: 0\r\n",
            "\r\n"
        );

        assert_eq!(serialize_sip_message(&sip_message), expected.as_bytes())
    }

    #[test]
    fn serialize_headers_preserves_names_values_and_order() {
        let headers = vec![
            Header {
                name: String::from("vIa"),
                value: String::from("SIP/2.0/UDP first.example.com;branch=z9hG4bK-first"),
            },
            Header {
                name: String::from("v"),
                value: String::from("SIP/2.0/TCP second.example.com;branch=z9hG4bK-second"),
            },
            Header {
                name: String::from("X-Trace-ID"),
                value: String::from("AbC-123"),
            },
        ];
        let expected = concat!(
            "vIa: SIP/2.0/UDP first.example.com;branch=z9hG4bK-first\r\n",
            "v: SIP/2.0/TCP second.example.com;branch=z9hG4bK-second\r\n",
            "X-Trace-ID: AbC-123\r\n"
        );

        assert_eq!(serialize_headers(&headers), expected);
        assert_eq!(serialize_headers(&[]), "");
    }

    #[test]
    fn serialize_start_line_formats_request_and_response() {
        let request = StartLine::Request(RequestLine {
            method: String::from("OPTIONS"),
            uri: String::from("sip:bob@example.com"),
            version: String::from("SIP/2.0"),
        });
        let response = StartLine::Response(StatusLine {
            version: String::from("SIP/2.0"),
            status_code: 200,
            reason_phrase: String::from("OK"),
        });

        assert_eq!(
            serialize_start_line(&request),
            String::from("OPTIONS sip:bob@example.com SIP/2.0\r\n")
        );
        assert_eq!(
            serialize_start_line(&response),
            String::from("SIP/2.0 200 OK\r\n")
        );
    }

    #[test]
    fn parse_headers_preserves_via_list_and_repeated_header() {
        let input = concat!(
            "Via: SIP/2.0/UDP first.example.com:5060",
            ";branch=z9hG4bK-first;received=192.0.2.1, ",
            "SIP/2.0/TCP second.example.com:5070;branch=z9hG4bK-second\r\n",
            "v: SIP/2.0/UDP third.example.com:5080;branch=z9hG4bK-third\r\n"
        );
        let headers = parse_headers(input).unwrap();

        assert_eq!(
            headers,
            vec![
                Header {
                    name: String::from("Via"),
                    value: String::from(
                        "SIP/2.0/UDP first.example.com:5060;branch=z9hG4bK-first;received=192.0.2.1, SIP/2.0/TCP second.example.com:5070;branch=z9hG4bK-second"
                    ),
                },
                Header {
                    name: String::from("v"),
                    value: String::from("SIP/2.0/UDP third.example.com:5080;branch=z9hG4bK-third")
                }
            ]
        );
        assert_eq!(serialize_headers(&headers), input);
        assert!(headers[0].matches_name("Via"));
        assert!(headers[1].matches_name("Via"));
    }

    #[test]
    fn parse_headers_preserves_contact_list_and_repeated_header() {
        let input = concat!(
            "Contact: \"Alice, Desktop\" <sip:alice@desktop.example.com;transport=tcp>",
            ";expires=60;q=0.8;x-note=\"desk, primary\", ",
            "<sip:alice@moblie.example.com;transport=udp>;expires=120;q=0.5\r\n",
            "m: <sip:alice@tablet.example.com>;expires=180;q=0.3\r\n",
        );
        let headers = parse_headers(input).unwrap();

        assert_eq!(
            headers,
            vec![
                Header {
                    name: String::from("Contact"),
                    value: String::from(
                        "\"Alice, Desktop\" <sip:alice@desktop.example.com;transport=tcp>;expires=60;q=0.8;x-note=\"desk, primary\", <sip:alice@moblie.example.com;transport=udp>;expires=120;q=0.5"
                    )
                },
                Header {
                    name: String::from("m"),
                    value: String::from("<sip:alice@tablet.example.com>;expires=180;q=0.3")
                }
            ]
        );
        assert_eq!(serialize_headers(&headers), input);
        assert!(headers[0].matches_name("Contact"));
        assert!(headers[1].matches_name("Contact"));
    }

    #[test]
    fn parse_headers_preserves_contacts_and_parameters() {
        let input = concat!(
            "Contact: \"Alice, Desktop\" <sip:alice@desktop.example.com;transport=tcp>;expires=60;q=0.8\r\n",
            "m: <sip:alice@moblie.example.com;transport=udp>;expires=120;q=0.5\r\n",
        );
        let result = parse_headers(input);

        assert_eq!(
            result,
            Ok(vec![
                Header {
                    name: String::from("Contact"),
                    value: String::from(
                        "\"Alice, Desktop\" <sip:alice@desktop.example.com;transport=tcp>;expires=60;q=0.8"
                    )
                },
                Header {
                    name: String::from("m"),
                    value: String::from(
                        "<sip:alice@moblie.example.com;transport=udp>;expires=120;q=0.5"
                    )
                }
            ])
        );

        let headers = result.unwrap();

        assert!(headers[0].matches_name("Contact"));
        assert!(headers[1].matches_name("Contact"));
    }

    #[test]
    fn parse_sip_message_with_response_and_body() {
        let input = concat!(
            "SIP/2.0 200 OK\r\n",
            "Via: SIP/2.0/UDP client.example.com;branch=z9hG4bK-test\r\n",
            "From: <sip:alice@example.com>;tag=123\r\n",
            "To: <sip:bob@example.com>;tag=456\r\n",
            "Call-ID: test@example.com\r\n",
            "CSeq: 1 OPTIONS\r\n",
            "Content-Type: text/plain\r\n",
            "Content-Length: 5\r\n",
            "\r\n",
            "hello",
        );

        let result = parse_sip_message(input.as_bytes());

        assert_eq!(
            result,
            Ok(SipMessage {
                start_line: StartLine::Response(StatusLine {
                    version: String::from("SIP/2.0"),
                    status_code: 200,
                    reason_phrase: String::from("OK")
                }),
                headers: vec![
                    Header {
                        name: String::from("Via"),
                        value: String::from("SIP/2.0/UDP client.example.com;branch=z9hG4bK-test")
                    },
                    Header {
                        name: String::from("From"),
                        value: String::from("<sip:alice@example.com>;tag=123")
                    },
                    Header {
                        name: String::from("To"),
                        value: String::from("<sip:bob@example.com>;tag=456")
                    },
                    Header {
                        name: String::from("Call-ID"),
                        value: String::from("test@example.com")
                    },
                    Header {
                        name: String::from("CSeq"),
                        value: String::from("1 OPTIONS"),
                    },
                    Header {
                        name: String::from("Content-Type"),
                        value: String::from("text/plain")
                    },
                    Header {
                        name: String::from("Content-Length"),
                        value: String::from("5")
                    }
                ],
                body: b"hello".to_vec()
            })
        )
    }

    #[test]
    fn parse_sip_message_preserves_binary_body() {
        let header_text = concat!(
            "OPTIONS sip:bob@example.com SIP/2.0\r\n",
            "Via: SIP/2.0/UDP client.example.com;branch=z9hG4bK-test\r\n",
            "Max-Forwards: 70\r\n",
            "From: <sip:alice@example.com>;tag=123\r\n",
            "To: <sip:bob@example.com>\r\n",
            "Call-ID: test@example.com\r\n",
            "CSeq: 1 OPTIONS\r\n",
            "Content-Length: 6\r\n",
            "Content-Type: application/octet-stream\r\n",
            "\r\n"
        );
        let mut input = header_text.as_bytes().to_vec();
        let body = b"\xff\0\r\n\r\n";
        input.extend_from_slice(body);

        let message = parse_sip_message(&input).unwrap();
        assert_eq!(message.body, body.to_vec());
    }

    #[test]
    fn parse_sip_message_withou_body() {
        let input = concat!(
            "OPTIONS sip:bob@example.com SIP/2.0\r\n",
            "Via: SIP/2.0/UDP client.example.com;branch=z9hG4bK-test\r\n",
            "Max-Forwards: 70\r\n",
            "From: <sip:alice@example.com>;tag=123\r\n",
            "To: <sip:bob@example.com>\r\n",
            "Call-ID: test@example.com\r\n",
            "CSeq: 1 OPTIONS\r\n",
            "Content-Length: 0\r\n",
            "\r\n"
        );
        let result = parse_sip_message(input.as_bytes());

        assert_eq!(
            result,
            Ok(SipMessage {
                start_line: StartLine::Request(RequestLine {
                    method: String::from("OPTIONS"),
                    uri: String::from("sip:bob@example.com"),
                    version: String::from("SIP/2.0")
                }),
                headers: vec![
                    Header {
                        name: String::from("Via"),
                        value: String::from("SIP/2.0/UDP client.example.com;branch=z9hG4bK-test")
                    },
                    Header {
                        name: String::from("Max-Forwards"),
                        value: String::from("70")
                    },
                    Header {
                        name: String::from("From"),
                        value: String::from("<sip:alice@example.com>;tag=123")
                    },
                    Header {
                        name: String::from("To"),
                        value: String::from("<sip:bob@example.com>")
                    },
                    Header {
                        name: String::from("Call-ID"),
                        value: String::from("test@example.com")
                    },
                    Header {
                        name: String::from("CSeq"),
                        value: String::from("1 OPTIONS")
                    },
                    Header {
                        name: String::from("Content-Length"),
                        value: String::from("0")
                    }
                ],
                body: Vec::new()
            })
        )
    }

    #[test]
    fn split_sip_head_and_body_rejects_missing_separator() {
        let result = split_sip_head_and_body(
            b"OPTIONS sip:bob@example.com SIP/2.0\r\nContent-Length: 0\r\n",
        );
        assert_eq!(result, Err(String::from("SIP 消息缺少头部结束空行")));

        let result = split_sip_head_and_body(b"");
        assert_eq!(result, Err(String::from("SIP 消息缺少头部结束空行")));
    }

    #[test]
    fn split_sip_head_and_body_preserves_binary_body() {
        let result = split_sip_head_and_body(
            b"OPTIONS sip:bob@example.com SIP/2.0\r\nContent-Length: 6\r\n\r\n\xff\0\r\n\r\n",
        );

        let expected_head: &[u8] = b"OPTIONS sip:bob@example.com SIP/2.0\r\nContent-Length: 6\r\n";
        let expected_body: &[u8] = b"\xff\0\r\n\r\n";

        assert_eq!(result, Ok((expected_head, expected_body)));
    }

    #[test]
    fn split_sip_head_and_body_with_body() {
        let result = split_sip_head_and_body(
            b"OPTIONS sip:bob@example.com SIP/2.0\r\nContent-Length: 5\r\n\r\nhello",
        );

        let expected_head: &[u8] = b"OPTIONS sip:bob@example.com SIP/2.0\r\nContent-Length: 5\r\n";
        let expected_body: &[u8] = b"hello";

        assert_eq!(result, Ok((expected_head, expected_body)));
    }

    #[test]
    fn header_name_comparison_supports_compact_forms() {
        let cases = [
            ("Call-ID", "i"),
            ("Contact", "m"),
            ("Content-Encoding", "e"),
            ("Content-Length", "l"),
            ("Content-Type", "c"),
            ("From", "f"),
            ("Subject", "s"),
            ("Supported", "k"),
            ("To", "t"),
        ];

        for (full, short) in cases {
            let full_header = Header {
                name: full.to_string(),
                value: String::new(),
            };
            let short_header = Header {
                name: short.to_string(),
                value: String::new(),
            };

            // 完整名称能匹配紧凑形式
            assert!(full_header.matches_name(short), "{full} -> {short}");
            assert!(full_header.matches_name(&short.to_ascii_uppercase()));

            // 紧凑形式也能匹配完整名称
            assert!(short_header.matches_name(full), "{short} -> {full}");
            assert!(short_header.matches_name(&full.to_ascii_uppercase()));

            // 保存的原始名称保持不变
            assert_eq!(full_header.name, full);
            assert_eq!(short_header.name, short);
        }
    }

    #[test]
    fn header_name_comparison_supports_compact_via() {
        let header = parse_header("v: SIP/2.0/UDP example.com\r\n").unwrap();
        assert_eq!(header.name, "v");
        assert!(header.matches_name("Via"));
        assert!(header.matches_name("VIA"));
        assert!(header.matches_name("v"));
        assert!(header.matches_name("V"));
        assert!(!header.matches_name("Contact"));

        let header = parse_header("Via: SIP/2.0/UDP example.com\r\n").unwrap();
        assert_eq!(header.name, "Via");
        assert!(header.matches_name("v"));
        assert!(header.matches_name("V"));

        let header = parse_header("abc: hello\r\n").unwrap();
        assert!(header.matches_name("ABC"));
        assert!(!header.matches_name("Via"));
        assert_eq!(header.name, "abc");
    }

    #[test]
    fn header_name_comparison_is_case_insensitive() {
        let header = parse_header("vIa: SIP/2.0/UDP first.example.com:5060\r\n").unwrap();

        assert_eq!(header.name, "vIa");
        assert!(header.matches_name("vIa"));
        assert!(header.matches_name("Via"));
        assert!(header.matches_name("VIA"));
        assert!(header.matches_name("via"));
        assert!(!header.matches_name("Contact"));
    }

    #[test]
    fn rejects_headers_with_orphan_continuation() {
        let result = parse_headers(" world\r\n");
        assert_eq!(result, Err(String::from("续行前缺少头字段")));

        let result = parse_headers("\tworld\r\n");
        assert_eq!(result, Err(String::from("续行前缺少头字段")));
    }

    #[test]
    fn parse_headers_preserves_header_after_fold() {
        let result =
            parse_headers("Subject: hello\r\n world\r\n\tthere\r\nX-Trace-ID: abc-123\r\n");

        assert_eq!(
            result,
            Ok(vec![
                Header {
                    name: String::from("Subject"),
                    value: String::from("hello world there")
                },
                Header {
                    name: String::from("X-Trace-ID"),
                    value: String::from("abc-123")
                }
            ])
        );
    }

    #[test]
    fn parse_header_with_folded_value() {
        let result = parse_headers("Subject: hello\r\n world\r\n");

        assert_eq!(
            result,
            Ok(vec![Header {
                name: String::from("Subject"),
                value: String::from("hello world"),
            }])
        );
    }

    #[test]
    fn parse_headers_preserves_repeated_via() {
        let result = parse_headers(
            "Via: SIP/2.0/UDP first.example.com:5060\r\nVia: SIP/2.0/TCP second.example.com:5070\r\n",
        );

        assert_eq!(
            result,
            Ok(vec![
                Header {
                    name: String::from("Via"),
                    value: String::from("SIP/2.0/UDP first.example.com:5060")
                },
                Header {
                    name: String::from("Via"),
                    value: String::from("SIP/2.0/TCP second.example.com:5070")
                }
            ])
        )
    }

    #[test]
    fn rejects_header_with_embedded_line_breaks() {
        let result = parse_header("Subject: hello\nworld\r\n");
        assert_eq!(result, Err(String::from("头字段值包含非法控制字符")));

        let result = parse_header("Subject: hello\rworld\r\n");
        assert_eq!(result, Err(String::from("头字段值包含非法控制字符")));
    }

    #[test]
    fn rejects_header_with_nul_in_value() {
        let result = parse_header("Subject: hello\0world\r\n");

        assert_eq!(result, Err(String::from("头字段值包含非法控制字符")));
    }

    #[test]
    fn rejects_header_with_invalid_name_character() {
        let result = parse_header("Bad@Name: hello\r\n");

        assert_eq!(result, Err(String::from("头字段名包含非法字符")));
    }

    #[test]
    fn rejects_header_with_space_in_name() {
        let result = parse_header("Bad Name: hello\r\n");

        assert_eq!(result, Err(String::from("头字段名包含非法字符")));
    }

    #[test]
    fn rejects_header_with_empty_line() {
        let result = parse_header("\r\n");

        assert_eq!(result, Err(String::from("头字段不能为空")));
    }

    #[test]
    fn rejects_header_with_whitespace_only_name() {
        let result = parse_header("\t: hello\r\n");

        assert_eq!(result, Err(String::from("头字段名不能为空")));
    }

    #[test]
    fn parse_header_with_via() {
        let result = parse_header("Via: SIP/2.0/UDP client.example.com:5056\r\n");

        assert_eq!(
            result,
            Ok(Header {
                name: String::from("Via"),
                value: String::from("SIP/2.0/UDP client.example.com:5056")
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_percent_encoded_password() {
        let result = parse_sip_uri("sip:bob:secret%5Eword@example.com");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob:secret%5Eword")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_invalid_password_character() {
        let result = parse_sip_uri("sip:bob:secret^word@example.com");

        assert_eq!(result, Err(String::from("URI password 包含非法字符")));
    }

    #[test]
    fn rejects_sip_uri_with_invalid_user_character() {
        let result = parse_sip_uri("sip:bo^b@example.com");

        assert_eq!(result, Err(String::from("URI user 包含非法字符")));
    }

    #[test]
    fn rejects_sip_uri_with_invalid_header_name_character() {
        let result = parse_sip_uri("sip:bob@example.com?sub^ject=hello");

        assert_eq!(result, Err(String::from("URI header 包含非法字符")));
    }

    #[test]
    fn parse_sip_uri_with_question_marks_in_header() {
        let result = parse_sip_uri("sip:bob@example.com?sub?ject=hello?world");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: vec![(String::from("sub?ject"), String::from("hello?world"))]
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_invalid_header_value_character() {
        let result = parse_sip_uri("sip:bob@example.com?subject=hello^world");

        assert_eq!(result, Err(String::from("URI header 包含非法字符")));
    }

    #[test]
    fn parse_sip_uri_with_percent_encoded_parameter_names() {
        let result = parse_sip_uri("sip:bob@example.com;x%5Emode=blue;x%2Fflag");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![
                    (String::from("x%5Emode"), Some(String::from("blue"))),
                    (String::from("x%2Fflag"), None)
                ],
                uri_headers: Vec::new()
            })
        )
    }

    #[test]
    fn rejects_sip_uri_with_invalid_extension_parameter_name() {
        let result = parse_sip_uri("sip:bob@example.com;x^mode=blue");
        assert_eq!(result, Err(String::from("URI 参数包含非法字符")));

        let result = parse_sip_uri("sip:bob@example.com;x^mode");
        assert_eq!(result, Err(String::from("URI 参数包含非法字符")))
    }

    #[test]
    fn parse_sip_uri_with_percent_encoded_extension_parameter_value() {
        let result = parse_sip_uri("sip:bob@example.com;x-mode=blue%5Ered");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![(String::from("x-mode"), Some(String::from("blue%5Ered")))],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_apostrophe_in_extension_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;x-note=it's");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![(String::from("x-note"), Some(String::from("it's")))],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_invalid_extension_paramter_value() {
        let result = parse_sip_uri("sip:bob@example.com;x-mode=blue^red");

        assert_eq!(result, Err(String::from("URI 参数包含非法字符")));
    }

    #[test]
    fn parse_sip_uri_with_extension_parameters() {
        let result = parse_sip_uri("sip:bob@example.com;x-mode=blue;x-flag");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![
                    (String::from("x-mode"), Some(String::from("blue"))),
                    (String::from("x-flag"), None),
                ],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_method_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;method=REGISTER");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![(String::from("method"), Some(String::from("REGISTER"))),],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_valueless_method_paramter() {
        let result = parse_sip_uri("sip:bob@example.com;method");

        assert_eq!(result, Err(String::from("method 参数必须包含值")));
    }

    #[test]
    fn parse_sip_uri_with_user_phone_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;user=phone");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![(String::from("user"), Some(String::from("phone")))],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_valueless_user_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;user");

        assert_eq!(result, Err(String::from("user 参数必须包含值")));
    }

    #[test]
    fn rejects_sip_uri_with_valueless_transport() {
        let result = parse_sip_uri("sip:bob@exmple.com;transport");

        assert_eq!(result, Err(String::from("transport 参数必须包含值")));
    }

    #[test]
    fn parse_sip_uri_with_hostname_maddr() {
        let result = parse_sip_uri("sip:bob@example.com;maddr=proxy.example.net");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![(
                    String::from("maddr"),
                    Some(String::from("proxy.example.net"))
                )],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_valueless_maddr() {
        let result = parse_sip_uri("sip:bob@example.com;maddr");

        assert_eq!(result, Err(String::from("maddr 参数必须包含值")));
    }

    #[test]
    fn rejects_sip_uri_with_ipv6_maddr_port() {
        let result = parse_sip_uri("sip:bob@example.com;maddr=[2001:db8::1]:5056");

        assert_eq!(result, Err(String::from("maddr host 无效")));
    }

    #[test]
    fn rejects_sip_uri_with_unclosed_ipv6_maddr() {
        let result = parse_sip_uri("sip:bob@example.com;maddr=[2001:db8::1");

        assert_eq!(result, Err(String::from("maddr host 无效")));
    }

    #[test]
    fn rejects_sip_uri_with_invalid_ipv6_maddr() {
        let result = parse_sip_uri("sip:bob@example.com;maddr=[not-ipv6]");

        assert_eq!(result, Err(String::from("maddr host 无效")));
    }

    #[test]
    fn parse_sip_uri_with_ipv6_maddr() {
        let result = parse_sip_uri("sip:bob@example.com;maddr=[2001:db8::1]");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![(String::from("maddr"), Some(String::from("[2001:db8::1]")))],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_invalid_maddr_host() {
        let result = parse_sip_uri("sip:bob@exmaple.com;transport=udp;maddr=bad_host.example.com");
        assert_eq!(result, Err(String::from("maddr host 无效")));

        let result = parse_sip_uri("sip:bob@example.com;maddr=-bad.example.com");
        assert_eq!(result, Err(String::from("maddr host 无效")));
    }

    #[test]
    fn parse_sip_uri_with_minimum_ttl_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=0");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![
                    (String::from("transport"), Some(String::from("udp"))),
                    (String::from("maddr"), Some(String::from("239.255.255.1"))),
                    (String::from("ttl"), Some(String::from("0")))
                ],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_valueless_ttl_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl");

        assert_eq!(result, Err(String::from("ttl 参数必须包含值")));
    }

    #[test]
    fn parse_sip_uri_with_leading_zeros_in_ttl_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=003");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![
                    (String::from("transport"), Some(String::from("udp"))),
                    (String::from("maddr"), Some(String::from("239.255.255.1"))),
                    (String::from("ttl"), Some(String::from("003")))
                ],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_ttl_parameter_longer_than_three_digits() {
        let result =
            parse_sip_uri("sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=0000");

        assert_eq!(result, Err(String::from("ttl 参数必须是 1 到 3 位数字")))
    }

    #[test]
    fn parse_sip_uri_with_maximum_ttl_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=255");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![
                    (String::from("transport"), Some(String::from("udp"))),
                    (String::from("maddr"), Some(String::from("239.255.255.1"))),
                    (String::from("ttl"), Some(String::from("255")))
                ],
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_out_of_range_ttl_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=256");

        assert_eq!(result, Err(String::from("ttl 参数超出 0..255 范围")));
    }

    #[test]
    fn rejects_sip_uri_with_non_numeric_ttl_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=abc");
        assert_eq!(result, Err(String::from("ttl 参数必须是十进制数字")));

        let result = parse_sip_uri("sip:bob@example.com;transport=udp;maddr=239.255.255.1;TTL=abc");
        assert_eq!(result, Err(String::from("ttl 参数必须是十进制数字")));
    }

    #[test]
    fn parse_sip_uri_with_hostname_label_starting_with_digit() {
        let result = parse_sip_uri("sip:bob@1example.com");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("1example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_hostname_top_label_starting_with_digit() {
        let result = parse_sip_uri("sip:bob@example.1com");

        assert_eq!(result, Err(String::from("hostname 无效")))
    }

    #[test]
    fn parse_sip_uri_with_ipv4_host() {
        let result = parse_sip_uri("sip:bob@192.0.2.1");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("192.0.2.1"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new()
            })
        )
    }

    #[test]
    fn rejects_sip_uri_with_empty_hostname_label() {
        let result = parse_sip_uri("sip:bob@example..com");
        assert_eq!(result, Err(String::from("hostname 无效")));

        let result = parse_sip_uri("sip:bob@.example.com");
        assert_eq!(result, Err(String::from("hostname 无效")));

        let result = parse_sip_uri("sip:bob@example.com..");
        assert_eq!(result, Err(String::from("hostname 无效")));
    }

    #[test]
    fn parse_sip_uri_with_trailing_dot_in_hostname() {
        let result = parse_sip_uri("sip:bob@example.com.");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com."),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new()
            })
        )
    }

    #[test]
    fn rejects_sip_uri_with_hostname_label_ending_with_hyphen() {
        let result = parse_sip_uri("sip:bob@a.example-.com");

        assert_eq!(result, Err(String::from("hostname 无效")))
    }

    #[test]
    fn rejects_sip_uri_with_hostname_label_startig_with_hyphen() {
        let result = parse_sip_uri("sip:bob@a.-example.com");

        assert_eq!(result, Err(String::from("hostname 无效")));
    }

    #[test]
    fn rejects_sip_uri_with_invalid_hostname_character() {
        let result = parse_sip_uri("sip:bob@exa_mple.com");

        assert_eq!(result, Err(String::from("hostname 无效")));

        let result = parse_sip_uri("sip:bob@exa+mple.com");

        assert_eq!(result, Err(String::from("hostname 无效")));
    }

    #[test]
    fn rejects_sip_uri_with_percent_encoded_host() {
        let result = parse_sip_uri("sip:bob@%65xample.com");

        assert_eq!(result, Err(String::from("host 不能包含百分号转义")));
    }

    #[test]
    fn parse_sip_uri_with_percent_encoded_header_value() {
        let result = parse_sip_uri("sip:bob@example.com?subject=hello%20world");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: vec![(String::from("subject"), String::from("hello%20world"))]
            })
        )
    }

    #[test]
    fn reject_sip_uri_with_invalid_percent_escape_in_headers() {
        let result = parse_sip_uri("sip:bob@example.com?sub%2Gject=hello");
        assert_eq!(result, Err(String::from("URI 百分号转义无效")));

        let result = parse_sip_uri("sip:bob@example.com?subject=%2G");
        assert_eq!(result, Err(String::from("URI 百分号转义无效")));
    }

    #[test]
    fn rejects_sip_uri_with_invalid_percent_escape_in_parameter_name() {
        let result = parse_sip_uri("sip:bob@example;fo%2G=bar");
        assert_eq!(result, Err(String::from("URI 百分号转义无效")));

        let result = parse_sip_uri("sip:bob@example;fo%2G");
        assert_eq!(result, Err(String::from("URI 百分号转义无效")));
    }

    #[test]
    fn parse_sip_uri_with_percent_in_transport_token() {
        let result = parse_sip_uri("sip:bob@example.com;transport=x%y");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![(String::from("transport"), Some(String::from("x%y")))],
                uri_headers: Vec::new()
            })
        )
    }

    #[test]
    fn rejects_sip_uri_with_invalid_precent_escape_in_parameter_value() {
        let result = parse_sip_uri("sip:bob@example.com;foo=%2G");

        assert_eq!(result, Err(String::from("URI 百分号转义无效")))
    }

    #[test]
    fn rejects_sip_uri_with_invalid_percent_escape() {
        let result = parse_sip_uri("sip:bo%2Gb@example.com");

        assert_eq!(result, Err(String::from("URI 百分号转义无效")));
    }

    #[test]
    fn rejects_sip_uri_with_incomplete_percent_escape() {
        let result = parse_sip_uri("sip:bob%@example.com");
        assert_eq!(result, Err(String::from("URI 百分号转义无效")));

        let result = parse_sip_uri("sip:bob%2@example.com");
        assert_eq!(result, Err(String::from("URI 百分号转义无效")));
    }

    #[test]
    fn parse_sip_uri_with_percent_encoded_userinfo() {
        let result = parse_sip_uri("sip:bo%20b@example.com");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bo%20b")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_unescaped_control_character() {
        let result = parse_sip_uri("sip:bo b@example.com");

        assert_eq!(result, Err(String::from("URI 不能包含空白或控制字符")));

        let result = parse_sip_uri("sip:bo\0b@example.com");

        assert_eq!(result, Err(String::from("URI 不能包含空白或控制字符")));
    }

    #[test]
    fn parse_sip_uri_with_sips_scheme() {
        let result = parse_sip_uri("sips:pop@example.com");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sips"),
                userinfo: Some(String::from("pop")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_mixed_case_scheme() {
        let result = parse_sip_uri("sIpS:pop@example.com");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sIpS"),
                userinfo: Some(String::from("pop")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new()
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_header() {
        let result = parse_sip_uri("sip:bob@example.com?subject=hello");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: vec![(String::from("subject"), String::from("hello"))]
            })
        )
    }

    #[test]
    fn parse_sip_uri_with_empty_header_value() {
        let result = parse_sip_uri("sip:bob@example.com?subject=");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: vec![(String::from("subject"), String::from(""))]
            })
        )
    }

    #[test]
    fn parse_sip_uri_with_multiple_headers() {
        let result = parse_sip_uri("sip:bob@example.com?subject=hello&priority=urgent");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: vec![
                    (String::from("subject"), String::from("hello")),
                    (String::from("priority"), String::from("urgent"))
                ]
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_parameters_and_headers() {
        let result =
            parse_sip_uri("sip:bob@example.com;transport=udp;lr?subject=hello&priority=urgent");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![
                    (String::from("transport"), Some(String::from("udp"))),
                    (String::from("lr"), None)
                ],
                uri_headers: vec![
                    (String::from("subject"), String::from("hello")),
                    (String::from("priority"), String::from("urgent"))
                ]
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_delimiters_inuserinfo() {
        let result = parse_sip_uri("sip:bob;tag?x@example.com;transport=udp?subject=hello");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob;tag?x")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![(String::from("transport"), Some(String::from("udp")))],
                uri_headers: vec![(String::from("subject"), String::from("hello"))]
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_empty_scheme() {
        let result = parse_sip_uri(":bob@example.com");

        assert_eq!(result, Err(String::from("scheme 不能为空")));
    }

    #[test]
    fn rejects_sip_uri_with_unsupoorted_scheme() {
        let result = parse_sip_uri("http:bob@example.com");

        assert_eq!(result, Err(String::from("不支持的 URI scheme")));
    }

    #[test]
    fn rejects_sip_uri_with_header_without_equals() {
        let result = parse_sip_uri("sip:bob@example.com?subject");

        assert_eq!(result, Err(String::from("header 必须包含等号")));
    }

    #[test]
    fn rejects_sip_uri_with_empty_header_name() {
        let result = parse_sip_uri("sip:bob@example.com?=hello");

        assert_eq!(result, Err(String::from("header 参数名不能为空")));
    }

    #[test]
    fn parse_sip_uri_with_transport_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![(String::from("transport"), Some(String::from("udp")))],
                uri_headers: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_multiple_parameters() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp;lr");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: vec![
                    (String::from("transport"), Some(String::from("udp"))),
                    (String::from("lr"), None)
                ],
                uri_headers: Vec::new()
            })
        )
    }

    #[test]
    fn parse_sip_uri_with_ipv6_port_and_parmeters() {
        let result = parse_sip_uri("sip:bob@[2001:db8::1]:5056;transport=udp;lr");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("[2001:db8::1]"),
                port: Some(5056),
                uri_params: vec![
                    (String::from("transport"), Some(String::from("udp"))),
                    (String::from("lr"), None)
                ],
                uri_headers: Vec::new(),
            })
        )
    }

    #[test]
    fn rejects_sip_uri_with_empty_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;=udp");

        assert_eq!(result, Err(String::from("URI 参数名不能为空")));
    }

    #[test]
    fn rejects_sip_uri_with_empty_parameter_value() {
        let result = parse_sip_uri("sip:bob@example.com;transport=;");

        assert_eq!(result, Err(String::from("URI 参数值不能为空")));
    }

    #[test]
    fn rejects_sip_uri_with_duplicate_parameter() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp;transport=udp");

        assert_eq!(result, Err(String::from("URI 参数名不能重复")));
    }

    #[test]
    fn rejects_sip_uri_with_duplicate_parameter_ignoring_case() {
        let result = parse_sip_uri("sip:bob@example.com;Transport=udp;transport=udp");

        assert_eq!(result, Err(String::from("URI 参数名不能重复")));
    }

    #[test]
    fn rejects_sip_uri_with_trailing_parameter_separator() {
        let result = parse_sip_uri("sip:bob@example.com;transport=udp;");

        assert_eq!(result, Err(String::from("URI 参数名不能为空")))
    }

    #[test]
    fn parse_sip_uri_with_ipv6() {
        let result = parse_sip_uri("sip:bob@[2001:db8::1]");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("[2001:db8::1]"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_ipv6_port() {
        let result = parse_sip_uri("sip:bob@[2001:db8::1]:5056");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("[2001:db8::1]"),
                port: Some(5056),
                uri_params: Vec::new(),
                uri_headers: Vec::new(),
            })
        )
    }

    #[test]
    fn rejects_sip_uri_with_ipv6_missing_closing_bracket() {
        let result = parse_sip_uri("sip:bob@[::1");

        assert_eq!(result, Err(String::from("IPv6 缺少闭括号")));
    }

    #[test]
    fn rejects_sip_uri_with_invalid_ipv6_suffix() {
        let result = parse_sip_uri("sip:bob@[::1]abc");

        assert_eq!(result, Err(String::from("IPv6 闭括号只能接端口")));
    }

    #[test]
    fn rejects_sip_uri_with_empty_ipv6_address() {
        let result = parse_sip_uri("sip:bob@[]");

        assert_eq!(result, Err(String::from("IPv6 地址无效")));
    }

    #[test]
    fn rejects_sip_uri_with_invalid_ipv6_address() {
        let result = parse_sip_uri("sip:bob@[abc]");

        assert_eq!(result, Err(String::from("IPv6 地址无效")));
    }

    #[test]
    fn rejects_sip_uri_with_empty_ipv6_port() {
        let result = parse_sip_uri("sip:bob@[::1]:");

        assert_eq!(result, Err(String::from("port 必须是十进制数字")));
    }

    #[test]
    fn parse_sip_uri_with_maximum_port() {
        let result = parse_sip_uri("sip:bob@example.com:65535");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: Some(65535),
                uri_params: Vec::new(),
                uri_headers: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_out_of_range_port() {
        let result = parse_sip_uri("sip:bob@example.com:65536");

        assert_eq!(result, Err(String::from("port 超出 u16 范围")));
    }

    #[test]
    fn parse_sip_uri_with_invalid_port_format() {
        let result = parse_sip_uri("sip:bob@example.com:");
        assert_eq!(result, Err(String::from("port 必须是十进制数字")));

        let result = parse_sip_uri("sip:bob@example.com:abc");
        assert_eq!(result, Err(String::from("port 必须是十进制数字")));

        let result = parse_sip_uri("sip:bob@example.com:+505");
        assert_eq!(result, Err(String::from("port 必须是十进制数字")));
    }

    #[test]
    fn parse_sip_uri_with_empty_host() {
        let result = parse_sip_uri("sip:bob@:5056");

        assert_eq!(result, Err(String::from("host 不能为空")));
    }

    #[test]
    fn parse_sip_uri_with_port() {
        let result = parse_sip_uri("sip:bob@example.com:5056");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: Some(5056),
                uri_params: Vec::new(),
                uri_headers: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_sip_uri_with_userinfo() {
        let result = parse_sip_uri("sip:bob@example.com");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: Some(String::from("bob")),
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_sip_uri_without_userinfo() {
        let result = parse_sip_uri("sip:example.com");

        assert_eq!(
            result,
            Ok(SipUri {
                scheme: String::from("sip"),
                userinfo: None,
                host: String::from("example.com"),
                port: None,
                uri_params: Vec::new(),
                uri_headers: Vec::new(),
            })
        );
    }

    #[test]
    fn rejects_sip_uri_with_empty_host() {
        let result = parse_sip_uri("sip:bob@");
        assert_eq!(result, Err(String::from("host 不能为空")));

        let result = parse_sip_uri("sip:");
        assert_eq!(result, Err(String::from("host 不能为空")));
    }

    #[test]
    fn rejects_sip_uri_with_empty_userinfo() {
        let result = parse_sip_uri("sip:@example.com");

        assert_eq!(result, Err(String::from("有'@'符号时 userinfo 不能为空")));
    }

    #[test]
    fn rejects_sip_uri_without_scheme_separator() {
        let result = parse_sip_uri("bob@example.com");

        assert_eq!(result, Err(String::from("`:` 符号不能为空")));
    }

    #[test]
    fn parse_start_line_as_request() {
        let result = parse_start_line("INVITE sip:bob@example.com SIP/2.0\r\n");

        assert_eq!(
            result,
            Ok(StartLine::Request(RequestLine {
                method: String::from("INVITE"),
                uri: String::from("sip:bob@example.com"),
                version: String::from("SIP/2.0")
            }))
        );
    }

    #[test]
    fn parse_start_line_as_response() {
        let result = parse_start_line("SIP/2.0 486 Busy Here\r\n");

        assert_eq!(
            result,
            Ok(StartLine::Response(StatusLine {
                version: String::from("SIP/2.0"),
                status_code: 486,
                reason_phrase: String::from("Busy Here")
            }))
        );
    }

    #[test]
    fn parse_start_line_with_mixed_case_response_prefix() {
        let result = parse_start_line("sIp/2.0 486 Busy Here\r\n");

        assert_eq!(
            result,
            Ok(StartLine::Response(StatusLine {
                version: String::from("sIp/2.0"),
                status_code: 486,
                reason_phrase: String::from("Busy Here")
            }))
        );
    }

    #[test]
    fn propagates_request_line_error() {
        let result = parse_start_line("INVITE sip:bob@examle.com\r\n");

        assert_eq!(result, Err(String::from("version 不存在")));
    }

    #[test]
    fn propagates_status_line_error() {
        let result = parse_start_line("SIP/2.0 abc Busy Here\r\n");

        assert_eq!(result, Err(String::from("status_code 无法转换为 u16")));
    }

    #[test]
    fn parse_status_line_with_single_spaces() {
        let result = parse_status_line("SIP/2.0 486 Busy Here\r\n");

        assert_eq!(
            result,
            Ok(StatusLine {
                version: String::from("SIP/2.0"),
                status_code: 486,
                reason_phrase: String::from("Busy Here")
            })
        );
    }

    #[test]
    fn parse_status_line_with_tab_in_reason_phrase() {
        let result = parse_status_line("SIP/2.0 486 Busy\tHere\r\n");

        assert_eq!(
            result,
            Ok(StatusLine {
                version: "SIP/2.0".to_string(),
                status_code: 486,
                reason_phrase: "Busy\tHere".to_string()
            })
        );
    }

    #[test]
    fn parse_status_line_with_empty_reason_phrase() {
        let result = parse_status_line("SIP/2.0 200 \r\n");

        assert_eq!(
            result,
            Ok(StatusLine {
                version: "SIP/2.0".to_string(),
                status_code: 200,
                reason_phrase: "".to_string()
            })
        );
    }

    #[test]
    fn parse_request_line_with_single_spaces() {
        let result = parse_request_line("INVITE sip:bob@example.com SIP/2.0\r\n");

        assert_eq!(
            result,
            Ok(RequestLine {
                method: String::from("INVITE"),
                uri: String::from("sip:bob@example.com"),
                version: String::from("SIP/2.0")
            })
        );
    }

    #[test]
    fn parse_request_line_with_multiple_spaces() {
        let result = parse_request_line("INVITE  sip:bob@example.com  SIP/2.0\r\n");

        assert_eq!(
            result,
            Ok(RequestLine {
                method: String::from("INVITE"),
                uri: String::from("sip:bob@example.com"),
                version: String::from("SIP/2.0")
            })
        );
    }

    #[test]
    fn rejects_request_line_with_missing_fields() {
        let result = parse_request_line("INVITE SIP/2.0\r\n");

        assert_eq!(result, Err(String::from("version 不存在")));
    }
}
