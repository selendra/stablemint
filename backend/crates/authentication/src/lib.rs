pub mod jwt;
pub mod password;
pub mod service;

// Re-export key items for convenience
pub use jwt::{Claims, JwtService};
pub use password::{hash_password, verify_password};
pub use service::AuthService;
