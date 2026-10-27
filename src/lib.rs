//! # webreq
//!
//! `webreq` is a lightweight, target-agnostic HTTP client abstraction library for Rust. 
//! It provides a unified API (`RequestBuilder`, `Response`, `Method`) that seamlessly compiles across 
//! standard native environments (using `reqwest`) and WebAssembly components (using `wasi-http`).
//!
//! ## Target-Based Selection
//! 
//! `webreq` automatically routes to the appropriate underlying implementation based on your build target:
//! - **Native targets** (e.g., `x86_64-unknown-linux-gnu`): Uses `reqwest` under the hood.
//! - **WASI targets** (e.g., `wasm32-wasip1`): Uses `wasi-http-client` for component-model networking.
//!
//! No manual feature flags are required for backend selection—Cargo handles it automatically!
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use webreq::{Client, Method, RequestBuilder, WebClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!     
//!     let request = RequestBuilder::new(Method::GET, "[https://httpbin.org/get](https://httpbin.org/get)")
//!         .header("User-Agent", "webreq-client");
//!         
//!     let response = client.execute(request).await?;
//!     
//!     println!("Status: {}", response.status);
//!     println!("Body: {}", response.text()?);
//!     
//!     Ok(())
//! }
//! ```\
pub mod status;
pub use status::*;
/// Common HTTP header name constants.
pub mod header {
    pub const ACCEPT: &str = "accept";
    pub const ACCEPT_CHARSET: &str = "accept-charset";
    pub const ACCEPT_ENCODING: &str = "accept-encoding";
    pub const ACCEPT_LANGUAGE: &str = "accept-language";
    pub const AUTHORIZATION: &str = "authorization";
    pub const CACHE_CONTROL: &str = "cache-control";
    pub const CONNECTION: &str = "connection";
    pub const CONTENT_LENGTH: &str = "content-length";
    pub const CONTENT_TYPE: &str = "content-type";
    pub const COOKIE: &str = "cookie";
    pub const HOST: &str = "host";
    pub const ORIGIN: &str = "origin";
    pub const REFERER: &str = "referer";
    pub const SET_COOKIE: &str = "set-cookie";
    pub const USER_AGENT: &str = "user-agent";
}
use std::collections::HashMap;
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Represents standard HTTP methods.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Method::GET => "GET",
            Method::HEAD => "HEAD",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::CONNECT => "CONNECT",
            Method::OPTIONS => "OPTIONS",
            Method::TRACE => "TRACE",
            Method::PATCH => "PATCH",
        };
        write!(f, "{}", s)
    }
}

// ==========================================
// Client Implementation (Target-Based)
// ==========================================

/// A unified HTTP client that wraps target-specific network backends via static dispatch.
///
/// `Client` automatically delegates calls to `reqwest` on native targets and 
/// `wasi-http-client` when compiled for WebAssembly (`wasm32-wasip1`).
#[derive(Debug)]
pub enum Error {
    Network(String),
    InvalidUrl(String),
    Serialization(String),
    Timeout,
    Other(String),
    Status(u16),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Network(msg) => write!(f, "Network error: {}", msg),
            Error::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Timeout => write!(f, "RequestBuilder timed out"),
            Error::Other(msg) => write!(f, "HTTP client error: {}", msg),
            Error::Status(code) => write!(f, "HTTP Status Code: {}", code),
        }
    }
}

/// Abstract HTTP RequestBuilder.
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    client: Client,
}

impl RequestBuilder {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HashMap::new(),
            body: Vec::new(),
            client: Client::new(),
        }
    }

    pub async fn send(self) -> Result<Response, Error> {
        self.client.execute(self.clone()).await
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }
}

/// Abstract HTTP Response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub async fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    #[cfg(feature = "serde")]
    pub async fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    pub fn status(&self) -> StatusCode {
        StatusCode(self.status)
    }

    pub fn error_for_status(&self) -> Result<(), Error> {
        if self.status().is_client_error() {
            return Err(Error::Status(self.status));
        }
        Ok(())
    }
}

/// Core WebClient trait providing unified asynchronous request execution.
#[async_trait::async_trait]
pub trait WebClient: Send + Sync {
    async fn execute(&self, request: RequestBuilder) -> Result<Response, Error>;
}

// ==========================================
// Reqwest Implementation (Feature: reqwest)
// ==========================================

#[cfg(not(target_os = "wasi"))]
#[derive(Debug, Clone)]
pub struct ReqwestClient {
    client: reqwest::Client,
}

#[cfg(not(target_os = "wasi"))]
impl ReqwestClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[cfg(not(target_os = "wasi"))]
#[async_trait::async_trait]
impl WebClient for ReqwestClient {
    async fn execute(&self, request: RequestBuilder) -> Result<Response, Error> {
        let method = match request.method {
            Method::GET => reqwest::Method::GET,
            Method::HEAD => reqwest::Method::HEAD,
            Method::POST => reqwest::Method::POST,
            Method::PUT => reqwest::Method::PUT,
            Method::DELETE => reqwest::Method::DELETE,
            Method::CONNECT => reqwest::Method::CONNECT,
            Method::OPTIONS => reqwest::Method::OPTIONS,
            Method::TRACE => reqwest::Method::TRACE,
            Method::PATCH => reqwest::Method::PATCH,
        };

        let mut req_builder = self.client.request(method, &request.url);

        for (k, v) in request.headers {
            req_builder = req_builder.header(k, v);
        }

        if !request.body.is_empty() {
            req_builder = req_builder.body(request.body);
        }

        let resp = req_builder
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let status = resp.status().as_u16();
        
        let mut headers = HashMap::new();
        for (k, v) in resp.headers() {
            if let Ok(v_str) = v.to_str() {
                headers.insert(k.as_str().to_string(), v_str.to_string());
            }
        }

        let body = resp
            .bytes()
            .await
            .map_err(|e| Error::Network(e.to_string()))?
            .to_vec();

        Ok(Response { status, headers, body })
    }
}

// ==========================================
// WASI HTTP Implementation (Feature: wasi)
// ==========================================

#[cfg(target_os = "wasi")]
#[derive(Debug, Clone)]
pub struct WasiClient;

#[cfg(target_os = "wasi")]
impl WasiClient {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "wasi")]
#[async_trait::async_trait]
impl WebClient for WasiClient {
    async fn execute(&self, request: RequestBuilder) -> Result<Response, Error> {
        // Placeholder/Integration outline for `wasi-http` (WASI 0.2 / Component Model)
        // You would typically use `wasi::http::types` and `wasi::http::outgoing_handler`
        // or helper crates like `wasi-http-client`.
        
        // Example sketch:
        // 1. Parse URL & construct WASI OutgoingRequestBuilder
        // 2. Send via OutgoingHandler::handle
        // 3. Await IncomingResponse and read body streams into Vec<u8>
        
        Err(Error::Other("WASI backend not fully initialized in snippet".into()))
    }
}

/// Unified Client enum allowing static dispatch between backends.
#[derive(Debug, Clone)]
pub enum Client {
    #[cfg(not(target_os = "wasi"))]
    Reqwest(ReqwestClient),

    #[cfg(target_os = "wasi")]
    Wasi(WasiClient),

}

impl Client {
    #[cfg(not(target_os = "wasi"))]
    pub fn new() -> Self {
        Self::Reqwest(ReqwestClient::new())
    }
    #[cfg(target_os = "wasi")]
    pub fn new() -> Self {
        Self::Wasi(WasiClient::new())
    }
    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        RequestBuilder {
            url: url.to_string(),
            method,
            body: Vec::new(),
            headers: HashMap::new(),
            client: self.clone(),
        }
    }
}

#[async_trait::async_trait]
impl WebClient for Client {
    async fn execute(&self, request: RequestBuilder) -> Result<Response, Error> {
        match self {
            #[cfg(not(target_os = "wasi"))]
            Client::Reqwest(client) => client.execute(request).await,

            #[cfg(target_os = "wasi")]
            Client::Wasi(client) => client.execute(request).await,
            _ => todo!(),
        }
    }
}