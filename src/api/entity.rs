use dashmap::DashMap;
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug)]
struct UserInner {
    pub uid: String,
    pub username: String,
    pub pwd_hash: String,
}

#[derive(Clone, Debug)]
pub struct User {
    inner: Arc<UserInner>,
}

impl User {
    pub fn new(uid: &str, username: &str, pwd_hash: &str) -> Self {
        Self {
            inner: Arc::new(UserInner {
                uid: uid.to_string(),
                username: username.to_string(),
                pwd_hash: pwd_hash.to_string(),
            }),
        }
    }

    pub fn uid(&self) -> &str {
        &self.inner.uid
    }

    pub fn username(&self) -> &str {
        &self.inner.username
    }

    pub fn pwd_hash(&self) -> &str {
        &self.inner.pwd_hash
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    users: Arc<DashMap<String, User>>,
    assets_base_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users: Arc::new(DashMap::new()),
            assets_base_dir: std::env::home_dir().unwrap(),
        }
    }

    pub fn get_user(&self, uid: &str) -> Option<User> {
        match self.users.get(uid) {
            Some(e) => Some(e.value().clone()),
            _ => None,
        }
    }

    pub fn put_user(&mut self, user: User) {
        self.users.insert(user.uid().to_string(), user);
    }

    pub fn get_assets_base_dir(&self) -> &PathBuf {
        &self.assets_base_dir
    }
}
