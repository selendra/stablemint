use std::sync::Arc;
use surrealdb::engine::any::Any;
use tokio::sync::OnceCell;

pub static DB_ARC: OnceCell<Arc<Database>> = OnceCell::const_new();

#[derive(Clone)]
pub struct Database {
    pub connection: surrealdb::Surreal<Any>,
}
