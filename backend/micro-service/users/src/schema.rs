use anyhow::Result;
use chrono::Utc;
use stablemint_models::user::{CreateUserInput, DBUser, User};
use stablemint_surrealdb::{services::DbService, types::Database};
use stablemint_utils::hash_password;
use uuid::Uuid;

pub struct UserService<'a> {
    db_service: DbService<'a, DBUser>,
}

impl<'a> UserService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self {
            db_service: DbService::new(db, "users"),
        }
    }

     // Get user by ID
     pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let user = self.db_service.get_record_by_id(id).await?;
        Ok(user.map(User::from_db))
    }

     // Create a new user
     pub async fn create_user(&self, input: CreateUserInput) -> Result<User> {
        let hashed_password = hash_password(&input.password)?;

        // Generate a fake private key (in a real app, this would use proper crypto)
        let private_key = format!("0x{}", Uuid::new_v4().to_string().replace("-", ""));

        let now = Utc::now();
        let user = DBUser {
            id: None,
            username: input.username,
            password: hashed_password,
            email: input.email,
            address: input.address,
            private_key,
            role: input.role,
            created_at: now,
            updated_at: now,
        };

        let created_user = self.db_service.create_record(user).await?.unwrap();
        Ok(User::from_db(created_user))
    }
}

