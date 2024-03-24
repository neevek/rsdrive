use dashmap::DashMap;
use std::{path::PathBuf, sync::Arc};

use crate::server::entity::User;

// #[derive(Clone, Debug)]
// struct UserInner {
//     pub uid: String,
//     pub username: String,
//     pub pwd_hash: String,
// }
//
// #[derive(Clone, Debug)]
// pub struct User {
//     inner: Arc<UserInner>,
// }
//
// impl User {
//     pub fn new(uid: &str, username: &str, pwd_hash: &str) -> Self {
//         Self {
//             inner: Arc::new(UserInner {
//                 uid: uid.to_string(),
//                 username: username.to_string(),
//                 pwd_hash: pwd_hash.to_string(),
//             }),
//         }
//     }
//
//     pub fn uid(&self) -> &str {
//         &self.inner.uid
//     }
//
//     pub fn username(&self) -> &str {
//         &self.inner.username
//     }
//
//     pub fn pwd_hash(&self) -> &str {
//         &self.inner.pwd_hash
//     }
// }

#[derive(Clone, Debug)]
pub struct AppState {
    users: Arc<DashMap<u32, User>>,
    assets_base_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users: Arc::new(DashMap::new()),
            assets_base_dir: homedir::get_my_home().unwrap_or(Some(PathBuf::from("./assets_base_dir"))).unwrap(),
        }
    }

    pub fn get_user(&self, uid: u32) -> Option<User> {
        self.users.get(&uid).map(|e| e.value().clone())
    }

    pub fn put_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    pub fn get_assets_base_dir(&self) -> &PathBuf {
        &self.assets_base_dir
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
