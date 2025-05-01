use candid::{CandidType, Deserialize};
use ic_cdk::api::time;
use std::fmt;

/// Error code definition
#[derive(CandidType, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Hash)]
pub enum ErrorCode {
    // 1000-1999: Common Errors
    SystemError = 1000,
    InvalidInput = 1001,
    InvalidOperation = 1002,
    NotFound = 1003,
    AlreadyExists = 1004,
    Unauthorized = 1005,
    Forbidden = 1006,
    ServiceUnavailable = 1007,
    DependencyFailed = 1008,
    DataInconsistency = 1009,
    ResourceUnavailable = 1010,
    RateLimitExceeded = 1011,
    PermissionDenied = 1012,
    QuotaExceeded = 1013,
    UnexpectedError = 1099,
    // 2000-2999: Authentication and authorization errors
    AuthUnauthorized = 2000,
    AuthForbidden = 2001,
    InvalidCredentials = 2002,
    InsufficientPermissions = 2003,
    SessionExpired = 2004,
    // 3000-3999: Resource errors
    ResourceNotFound = 3000,
    ResourceAlreadyExists = 3001,
    ResourceNotAvailable = 3002,
    ResourceExhausted = 3003,
    // 4000-4999: Input validation errors
    ValidationFailed = 4000,
    ValidationInvalidInput = 4001,
    ContentTooLong = 4002,
    InvalidFormat = 4003,
    MissingRequiredField = 4004,
    // 5000-5999: Operation errors
    OperationFailed = 5001,
    OperationTimeout = 5003,
    OperationCancelled = 5004,
    // 6000-6999: Data errors
    InvalidData = 6000,
    DataCorruption = 6001,
    DataLoss = 6003,
    // 7000-7999: Service errors
    ServiceError = 7000,
    ServiceTimeout = 7001,
}

/// Error context information
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ErrorContext {
    /// Error timestamp
    pub timestamp: u64,
    /// Error module
    pub module: String,
    /// Error function
    pub function: String,
    /// Additional context information
    pub details: Option<String>,
    /// Related entity ID
    pub entity_id: Option<String>,
    /// Error severity
    pub severity: ErrorSeverity,
}

/// Error severity
#[derive(CandidType, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Informational error
    Info,
    /// Warning, may impact some functionality
    Warning,
    /// Error, affecting current operation
    Error,
    /// Critical error, may impact system stability
    Critical,
}

/// Enhanced error type
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SquareErrorEnhanced {
    /// Error code
    pub code: ErrorCode,
    /// Error message
    pub message: String,
    /// Error context
    pub context: ErrorContext,
    /// Whether the error is recoverable
    pub recoverable: bool,
    /// Suggested recovery action
    pub recovery_hint: Option<String>,
}

/// Main error type
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum SquareError {
    /// Resource not found
    NotFound(String),
    /// Resource already exists
    AlreadyExists(String),
    /// Unauthorized access
    Unauthorized(String),
    /// Validation failed
    ValidationFailed(String),
    /// Content too long
    ContentTooLong(String),
    /// Invalid operation
    InvalidOperation(String),
    /// System error
    SystemError(String),
    /// Enhanced error type
    Enhanced(SquareErrorEnhanced),
}

impl SquareError {
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        module: impl Into<String>,
        function: impl Into<String>,
        severity: ErrorSeverity,
    ) -> Self {
        SquareError::Enhanced(SquareErrorEnhanced {
            code,
            message: message.into(),
            context: ErrorContext {
                timestamp: time(),
                module: module.into(),
                function: function.into(),
                details: None,
                entity_id: None,
                severity,
            },
            recoverable: false,
            recovery_hint: None,
        })
    }
    
    /// Add details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        if let SquareError::Enhanced(ref mut enhanced) = self {
            enhanced.context.details = Some(details.into());
        }
        self
    }
    
    /// Add entity ID
    pub fn with_entity_id(mut self, entity_id: impl Into<String>) -> Self {
        if let SquareError::Enhanced(ref mut enhanced) = self {
            enhanced.context.entity_id = Some(entity_id.into());
        }
        self
    }
    
    /// Set as recoverable
    pub fn recoverable(mut self, hint: impl Into<String>) -> Self {
        if let SquareError::Enhanced(ref mut enhanced) = self {
            enhanced.recoverable = true;
            enhanced.recovery_hint = Some(hint.into());
        }
        self
    }
    
    /// Get error code
    pub fn code(&self) -> ErrorCode {
        match self {
            SquareError::NotFound(_) => ErrorCode::NotFound,
            SquareError::AlreadyExists(_) => ErrorCode::AlreadyExists,
            SquareError::Unauthorized(_) => ErrorCode::Unauthorized,
            SquareError::ValidationFailed(_) => ErrorCode::ValidationFailed,
            SquareError::ContentTooLong(_) => ErrorCode::ContentTooLong,
            SquareError::InvalidOperation(_) => ErrorCode::InvalidOperation,
            SquareError::SystemError(_) => ErrorCode::SystemError,
            SquareError::Enhanced(enhanced) => enhanced.code,
        }
    }
    
    /// Record error log
    // Empty log implementation to save cycles
    pub fn log(&self) {
        // Logging has been disabled to save cycles
        // No-op implementation
    }
}

impl fmt::Display for SquareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquareError::NotFound(msg) => write!(f, "Not found: {}", msg),
            SquareError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            SquareError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            SquareError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            SquareError::ContentTooLong(msg) => write!(f, "Content too long: {}", msg),
            SquareError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            SquareError::SystemError(msg) => write!(f, "System error: {}", msg),
            SquareError::Enhanced(enhanced) => {
                write!(f, "[{}] {}", enhanced.code as u32, enhanced.message)
            }
        }
    }
}

/// Result type
pub type SquareResult<T> = Result<T, SquareError>;

// From String
impl From<String> for SquareError {
    fn from(error: String) -> Self {
        SquareError::SystemError(error)
    }
}

// From &str
impl From<&str> for SquareError {
    fn from(error: &str) -> Self {
        SquareError::SystemError(error.to_string())
    }
}
