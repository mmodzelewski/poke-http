use crate::error::{ParseError, Result};
use crate::http::{HttpFile, Method, Request};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Parser;

impl Parser {
    pub fn parse_file(path: &Path) -> Result<HttpFile> {
        let content =
            fs::read_to_string(path).map_err(|err| ParseError::IoError(err.to_string()))?;

        let mut http_file = HttpFile::new(path);
        let (requests, variables) = Self::parse_content(&content)?;
        http_file.requests = requests;
        http_file.variables = variables;
        Ok(http_file)
    }

    pub fn parse_content(content: &str) -> Result<(Vec<Request>, HashMap<String, String>)> {
        let mut requests = Vec::new();
        let mut variables = HashMap::new();
        let mut current_name: Option<String> = None;
        let mut current_request: Option<RequestBuilder> = None;

        for line in content.lines() {
            let line = line.trim_end();

            if let Some((name, value)) = Self::try_parse_variable(line) {
                variables.insert(name, value);
                continue;
            }

            if line.starts_with("###") {
                if let Some(builder) = current_request.take() {
                    requests.push(builder.build()?);
                }

                let name = line.trim_start_matches('#').trim();
                current_name = if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                };
                continue;
            }

            if line.starts_with('#') || line.starts_with("//") {
                continue;
            }

            if line.is_empty() {
                if let Some(ref mut builder) = current_request {
                    if builder.headers_done {
                        builder.body_lines.push(String::new());
                    } else {
                        builder.headers_done = true;
                    }
                }
                continue;
            }

            if current_request.is_none()
                && let Some(builder) = Self::try_parse_request_line(line, current_name.take())?
            {
                current_request = Some(builder);
                continue;
            }

            if let Some(ref mut builder) = current_request {
                if !builder.headers_done {
                    if let Some((key, value)) = Self::try_parse_header(line) {
                        builder.headers.push((key, value));
                        continue;
                    }
                    builder.headers_done = true;
                }

                builder.body_lines.push(line.to_string());
            }
        }

        if let Some(builder) = current_request {
            requests.push(builder.build()?);
        }

        Ok((requests, variables))
    }

    fn try_parse_variable(line: &str) -> Option<(String, String)> {
        if !line.starts_with('@') {
            return None;
        }
        let line = &line[1..];
        let eq_pos = line.find('=')?;
        let name = line[..eq_pos].trim().to_string();
        let value = line[eq_pos + 1..].trim().to_string();
        if name.is_empty() {
            return None;
        }
        Some((name, value))
    }

    fn try_parse_request_line(line: &str, name: Option<String>) -> Result<Option<RequestBuilder>> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            return Ok(None);
        }

        let method: Method = match parts[0].parse() {
            Ok(m) => m,
            Err(_) => return Ok(None),
        };

        let url = parts[1].to_string();

        Ok(Some(RequestBuilder {
            name,
            method,
            url,
            headers: Vec::new(),
            body_lines: Vec::new(),
            headers_done: false,
        }))
    }

    fn try_parse_header(line: &str) -> Option<(String, String)> {
        let colon_pos = line.find(':')?;
        let key = line[..colon_pos].trim().to_string();
        let value = line[colon_pos + 1..].trim().to_string();

        if key.contains(' ') {
            return None;
        }

        Some((key, value))
    }
}

struct RequestBuilder {
    name: Option<String>,
    method: Method,
    url: String,
    headers: Vec<(String, String)>,
    body_lines: Vec<String>,
    headers_done: bool,
}

impl RequestBuilder {
    fn build(self) -> Result<Request> {
        let mut request = Request::new(self.method, self.url);
        request.name = self.name;

        for (key, value) in self.headers {
            request.headers.insert(key, value);
        }

        let body: Vec<_> = self.body_lines.into_iter().collect();

        let body = body.join("\n");
        let body = body.trim();

        if !body.is_empty() {
            request.body = Some(body.to_string());
        }

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get() {
        let content = "GET https://api.example.com/users";
        let (requests, _) = Parser::parse_content(content).unwrap();

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, Method::Get);
        assert_eq!(requests[0].url, "https://api.example.com/users");
    }

    #[test]
    fn test_parse_with_headers() {
        let content = r#"
GET https://api.example.com/users
Authorization: Bearer token123
Content-Type: application/json
"#;
        let (requests, _) = Parser::parse_content(content).unwrap();

        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            requests[0].headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_parse_with_body() {
        let content = r#"
POST https://api.example.com/users
Content-Type: application/json

{
    "name": "John",
    "email": "john@example.com"
}
"#;
        let (requests, _) = Parser::parse_content(content).unwrap();

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, Method::Post);
        assert!(requests[0].body.is_some());
        assert!(requests[0].body.as_ref().unwrap().contains("John"));
    }

    #[test]
    fn test_parse_multiple_requests() {
        let content = r#"
### Get all users
GET https://api.example.com/users

### Create user
POST https://api.example.com/users
Content-Type: application/json

{"name": "John"}
"#;
        let (requests, _) = Parser::parse_content(content).unwrap();

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].name, Some("Get all users".to_string()));
        assert_eq!(requests[1].name, Some("Create user".to_string()));
    }

    #[test]
    fn test_parse_variables() {
        let content = r#"
@baseUrl = https://api.example.com
@token = secret123

GET https://api.example.com/users
"#;
        let (requests, variables) = Parser::parse_content(content).unwrap();

        assert_eq!(requests.len(), 1);
        assert_eq!(variables.len(), 2);
        assert_eq!(
            variables.get("baseUrl"),
            Some(&"https://api.example.com".to_string())
        );
        assert_eq!(variables.get("token"), Some(&"secret123".to_string()));
    }

    #[test]
    fn test_parse_variable_without_spaces() {
        let content = "@key=value";
        let (_, variables) = Parser::parse_content(content).unwrap();

        assert_eq!(variables.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_variable_with_spaces_in_value() {
        let content = "@message = hello world";
        let (_, variables) = Parser::parse_content(content).unwrap();

        assert_eq!(variables.get("message"), Some(&"hello world".to_string()));
    }
}
