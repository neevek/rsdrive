use std::path::PathBuf;

use super::entity::{AppState, User};
use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use futures::{Stream, TryStreamExt};
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::info;

pub async fn upload_file(
    user: User,
    state: State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let mut uploaded_file_names = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(file_name) = field.file_name() {
            let file_name = file_name.to_owned();
            stream_to_file(state.get_assets_base_dir(), file_name.as_str(), field).await?;
            info!("uploaded file: {}", file_name.as_str());
            uploaded_file_names.push(file_name);
        };
    }

    Ok(Json::from(uploaded_file_names))
}

async fn stream_to_file<S, E>(
    base_dir: &PathBuf,
    path: &str,
    stream: S,
) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<axum::BoxError>,
{
    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error =
            stream.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
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
