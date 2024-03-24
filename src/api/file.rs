use std::{net::SocketAddr, path::PathBuf};

use crate::{
    server::entity::User,
    storage::{local_file_storage::LocalFileStorage, sqlite_database::SqliteDatabase, StorageContext},
    transfer::transfer_task::TransferTask,
};

use super::entity::AppState;
use axum::{
    extract::{ws::WebSocket, ConnectInfo, State, WebSocketUpgrade},
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use headers::UserAgent;
use tracing::{debug, info};

pub async fn ws_handler(
    user: User,
    state: State<AppState>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown UA")
    };
    debug!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket2(user, state, socket, addr))
}

async fn handle_socket2(user: User, state: State<AppState>, socket: WebSocket, addr: SocketAddr) {
    debug!(">>>>>>>>> haha handle_socket2");
    let task = TransferTask::default();
    let db = Box::new(SqliteDatabase::open("./test.db").unwrap());
    let mut base_dir = PathBuf::new();
    base_dir.push("./shared_files");
    let local_file_storage = Box::new(LocalFileStorage::new(base_dir));
    let storage_ctx = Box::new(StorageContext {
        db,
        file_storage: local_file_storage,
    });
    task.start(user.id, socket, storage_ctx);

    // tokio::select! {
    //     rv_a = (&mut send_task) => {
    //         match rv_a {
    //             Ok(a) => debug!("{a} messages sent to {addr}"),
    //             Err(a) => debug!("Error sending messages {a:?}")
    //         }
    //         recv_task.abort();
    //     },
    //     rv_b = (&mut recv_task) => {
    //         match rv_b {
    //             Ok(b) => debug!("Received {b} messages"),
    //             Err(b) => debug!("Error receiving messages {b:?}")
    //         }
    //         send_task.abort();
    //     }
    // }

    info!("connection ended: {addr}");
}
