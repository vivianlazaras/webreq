use std::fmt;

/// Represents an HTTP status code with helper methods for inspection.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusCode(pub u16);

impl StatusCode {
    // Common Status Codes
    pub const OK: StatusCode = StatusCode(200);
    pub const CREATED: StatusCode = StatusCode(201);
    pub const ACCEPTED: StatusCode = StatusCode(202);
    pub const NO_CONTENT: StatusCode = StatusCode(204);
    pub const BAD_REQUEST: StatusCode = StatusCode(400);
    pub const UNAUTHORIZED: StatusCode = StatusCode(401);
    pub const FORBIDDEN: StatusCode = StatusCode(403);
    pub const NOT_FOUND: StatusCode = StatusCode(404);
    pub const INTERNAL_SERVER_ERROR: StatusCode = StatusCode(500);

    /// Returns the status code as a raw `u16` integer.
    pub fn as_u16(self) -> u16 {
        self.0
    }

    /// Checks if the status code is informational (100–199).
    pub fn is_informational(self) -> bool {
        (100..200).contains(&self.0)
    }

    /// Checks if the status code indicates success (200–299).
    pub fn is_success(self) -> bool {
        (200..300).contains(&self.0)
    }

    /// Checks if the status code indicates redirection (300–399).
    pub fn is_redirection(self) -> bool {
        (300..400).contains(&self.0)
    }

    /// Checks if the status code indicates a client error (400–499).
    pub fn is_client_error(self) -> bool {
        (400..500).contains(&self.0)
    }

    /// Checks if the status code indicates a server error (500–599).
    pub fn is_server_error(self) -> bool {
        (500..600).contains(&self.0)
    }
}

impl From<u16> for StatusCode {
    fn from(val: u16) -> Self {
        StatusCode(val)
    }
}

impl From<StatusCode> for u16 {
    fn from(status: StatusCode) -> Self {
        status.0
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}