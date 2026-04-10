use std::fmt;

/// Top-level error type for the Nahook SDK.
#[derive(Debug)]
pub enum NahookError {
    /// The API returned an error response (4xx/5xx).
    Api(ApiError),
    /// A network-level failure occurred (no HTTP response received).
    Network(NetworkError),
    /// The request timed out.
    Timeout(TimeoutError),
}

impl fmt::Display for NahookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NahookError::Api(e) => write!(f, "API error {}: {} ({})", e.status, e.message, e.code),
            NahookError::Network(e) => write!(f, "Network error: {}", e.cause),
            NahookError::Timeout(e) => write!(f, "Request timed out after {}ms", e.timeout_ms),
        }
    }
}

impl std::error::Error for NahookError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NahookError::Network(e) => Some(&e.cause),
            _ => None,
        }
    }
}

/// API returned an error response (4xx/5xx).
#[derive(Debug)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub retry_after: Option<u64>,
}

impl ApiError {
    /// Whether this error is retryable (5xx or 429).
    pub fn is_retryable(&self) -> bool {
        self.status >= 500 || self.status == 429
    }

    /// Whether this is an authentication error (401 or 403 with token_disabled).
    pub fn is_auth_error(&self) -> bool {
        self.status == 401 || (self.status == 403 && self.code == "token_disabled")
    }

    /// Whether this is a 404 not found error.
    pub fn is_not_found(&self) -> bool {
        self.status == 404
    }

    /// Whether this is a 429 rate limit error.
    pub fn is_rate_limited(&self) -> bool {
        self.status == 429
    }

    /// Whether this is a 400 validation error.
    pub fn is_validation_error(&self) -> bool {
        self.status == 400
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API error {}: {} ({})", self.status, self.message, self.code)
    }
}

impl std::error::Error for ApiError {}

/// Network-level failure (no HTTP response received).
#[derive(Debug)]
pub struct NetworkError {
    pub cause: reqwest::Error,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Network error: {}", self.cause)
    }
}

impl std::error::Error for NetworkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

/// Request timed out.
#[derive(Debug)]
pub struct TimeoutError {
    pub timeout_ms: u64,
}

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request timed out after {}ms", self.timeout_ms)
    }
}

impl std::error::Error for TimeoutError {}

impl From<ApiError> for NahookError {
    fn from(e: ApiError) -> Self {
        NahookError::Api(e)
    }
}

impl From<NetworkError> for NahookError {
    fn from(e: NetworkError) -> Self {
        NahookError::Network(e)
    }
}

impl From<TimeoutError> for NahookError {
    fn from(e: TimeoutError) -> Self {
        NahookError::Timeout(e)
    }
}
