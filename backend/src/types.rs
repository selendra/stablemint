use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any};

pub type Database = Arc<Surreal<Any>>;
