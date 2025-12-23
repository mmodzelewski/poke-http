use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Patch => write!(f, "PATCH"),
            Method::Delete => write!(f, "DELETE"),
            Method::Head => write!(f, "HEAD"),
            Method::Options => write!(f, "OPTIONS"),
        }
    }
}

impl std::str::FromStr for Method {
    type Err = crate::error::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "PATCH" => Ok(Method::Patch),
            "DELETE" => Ok(Method::Delete),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            _ => Err(crate::error::ParseError::InvalidMethod(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub name: Option<String>,
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Request {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            name: None,
            method,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("{} {}", self.method, self.url))
    }
}

#[derive(Debug, Clone)]
pub struct HttpFile {
    pub path: PathBuf,
    pub requests: Vec<Request>,
    pub variables: HashMap<String, String>,
}

impl HttpFile {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.into(),
            requests: Vec::new(),
            variables: HashMap::new(),
        }
    }
}
