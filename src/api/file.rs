use std::{
    borrow::Cow,
    net::SocketAddr,
    ops::ControlFlow,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    storage::{local_file_storage::LocalFileStorage, sqlite_database::SqliteDatabase, StorageContext},
    transfer::transfer_task::TransferTask,
};

use super::entity::{AppState, User};
use axum::{
    body::Bytes,
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket},
        ConnectInfo, Multipart, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::TypedHeader;
use futures::{SinkExt, Stream, StreamExt, TryStreamExt};
use headers::UserAgent;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::{debug, error, info};
use tracing_subscriber::field::debug;

pub async fn upload_file(user: User, state: State<AppState>, mut multipart: Multipart) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let mut uploaded_file_names = Vec::new();
    while let Ok(Some(mut field)) = multipart.next_field().await {
        while let chunk = field.chunk().await
        // .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))
        {
            match chunk {
                Ok(chunk) => info!("received {:?} bytes", chunk.unwrap().len()),
                Err(e) => error!(" error: {e}"),
            }
        }

        // if let Some(file_name) = field.file_name() {
        //     let file_name = file_name.to_owned();
        //     stream_to_file(state.get_assets_base_dir(), file_name.as_str(), field).await?;
        //     info!("uploaded file: {}", file_name.as_str());
        //     uploaded_file_names.push(file_name);
        // };
    }

    Ok(Json::from(uploaded_file_names))
}

async fn stream_to_file<S, E>(base_dir: &PathBuf, path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<axum::BoxError>,
{
    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = base_dir.join(path);
        let path = File::create(path).await?;
        let mut file = BufWriter::new(path);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, std::io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

pub async fn ws_handler(ws: WebSocketUpgrade, user_agent: Option<TypedHeader<UserAgent>>, ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    debug!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket2(socket, addr))
}

async fn handle_socket2(socket: WebSocket, addr: SocketAddr) {
    debug!(">>>>>>>>> haha handle_socket2");
    let task = TransferTask::default();
    let db = Box::new(SqliteDatabase::open(123, "./test.db").unwrap());
    let mut base_dir = PathBuf::new();
    base_dir.push("./shared_files");
    let local_file_storage = Box::new(LocalFileStorage::new(base_dir));
    let storage_ctx = Box::new(StorageContext { db, file_storage: local_file_storage });
    task.start(socket, storage_ctx);

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

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    //send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        debug!("Pinged {who}...");
    } else {
        debug!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    // receive single message from a client (we can either receive or send with socket).
    // this will likely be the Pong for our Ping or a hello message from client.
    // waiting for message from a client will block this task, but will not block other client's
    // connections.
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg, who).is_break() {
                return;
            }
        } else {
            debug!("client {who} abruptly disconnected");
            return;
        }
    }

    // Since each client gets individual statemachine, we can pause handling
    // when necessary to wait for some external event (in this case illustrated by sleeping).
    // Waiting for this client to finish getting its greetings does not prevent other clients from
    // connecting to server and receiving their greetings.
    for i in 1..5 {
        if socket.send(Message::Text(format!("Hi {i} times!"))).await.is_err() {
            debug!("client {who} abruptly disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut sender, mut receiver) = socket.split();

    // Spawn a task that will push several messages to the client (does not matter what client does)
    let mut send_task = tokio::spawn(async move {
        let n_msg = 20;
        for i in 0..n_msg {
            // In case of any websocket error, we exit.
            if sender.send(Message::Text(format!("Server message {i} ..."))).await.is_err() {
                return i;
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        debug!("Sending close to {who}...");
        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: close_code::NORMAL,
                reason: Cow::from("Goodbye"),
            })))
            .await
        {
            debug!("Could not send Close due to {e}, probably it is ok?");
        }
        n_msg
    });

    // This second task will receive messages from client and print them on server console
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            if process_message(msg, who).is_break() {
                break;
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => debug!("{a} messages sent to {who}"),
                Err(a) => debug!("Error sending messages {a:?}")
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => debug!("Received {b} messages"),
                Err(b) => debug!("Error receiving messages {b:?}")
            }
            send_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    debug!("Websocket context {who} destroyed");
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            debug!(">>> {who} sent str: {t:?}");
        }
        Message::Binary(d) => {
            debug!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                debug!(">>> {} sent close with code {} and reason `{}`", who, cf.code, cf.reason);
            } else {
                debug!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            debug!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            debug!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}
