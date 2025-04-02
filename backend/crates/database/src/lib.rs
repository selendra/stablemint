pub mod db_connect;
pub mod service;

use std::sync::{Arc, Mutex};
use surrealdb::engine::any::Any;
use tokio::sync::OnceCell;

pub static DB_ARC: OnceCell<Arc<Database>> = OnceCell::const_new();

pub struct ConnectionPool {
    pub connection_url: String,
    pub connections: Arc<Mutex<Vec<surrealdb::Surreal<Any>>>>,
    pub max_size: usize,
}

pub struct Database {
    pub pool: ConnectionPool,
}

// A wrapper for a connection that returns it to the pool when dropped
pub struct PooledConnection<'a> {
    conn: Option<surrealdb::Surreal<Any>>,
    pool: &'a ConnectionPool,
}

impl<'a> PooledConnection<'a> {
    pub fn get_ref(&self) -> &surrealdb::Surreal<Any> {
        self.conn.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut surrealdb::Surreal<Any> {
        self.conn.as_mut().unwrap()
    }
}

impl<'a> Drop for PooledConnection<'a> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.pool.return_connection(conn);
        }
    }
}
