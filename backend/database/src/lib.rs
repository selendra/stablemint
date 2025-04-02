
pub mod db_connect;
pub mod service;

use std::sync::Arc;
use surrealdb::engine::any::Any;
use tokio::sync::OnceCell;

pub static DB_ARC: OnceCell<Arc<Database>> = OnceCell::const_new();

pub struct Database {
    pub connection: surrealdb::Surreal<Any>,
}
