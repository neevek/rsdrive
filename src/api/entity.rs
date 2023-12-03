use std::sync::Arc;

use dashmap::DashMap;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct User {
    pub uid: String,
    pub username: String,
    pub pwd_hash: String,
}

#[derive(Clone, Debug)]
pub struct AppState {
    users: Arc<DashMap<String, Arc<User>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users: Arc::new(DashMap::new()),
        }
    }

    pub fn get_user(&self, uid: &str) -> Option<Arc<User>> {
        match self.users.get(uid) {
            Some(e) => Some(e.value().clone()),
            _ => None,
        }
    }

    pub fn put_user(&mut self, user: User) {
        self.users.insert(user.uid.clone(), Arc::new(user));
    }
}
