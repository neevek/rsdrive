use crate::{
    common::entity::{TransferControlMessage, TransferRequest, TransferResponse},
    server::entity::SyncFileInfo,
    storage::{file_storage::FileWriter, StorageContext},
};
use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tracing::{debug, error, info, warn};

#[derive(PartialEq)]
pub enum TransferState {
    Pending,
    Receiving(TransferRequest),
}

#[derive(Default)]
pub struct TransferTask {}

impl TransferTask {
    pub fn start(&self, socket: WebSocket, storage_ctx: Box<StorageContext>) {
        tokio::spawn(async {
            TransferTask::run(socket, storage_ctx).await.map_err(|e| {
                error!("{e}");
            })
        });
    }

    async fn run(socket: WebSocket, storage_ctx: Box<StorageContext>) -> Result<()> {
        let mut file_writer: Option<Box<dyn FileWriter>> = None;
        let mut file_info = SyncFileInfo::default();

        info!("start transferring...");

        let (mut sender, mut receiver) = socket.split();
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let trans_req = match TransferControlMessage::try_from(text.as_str()) {
                        Ok(TransferControlMessage::Request(req)) => req,
                        Ok(msg) => {
                            warn!("unexpected message:{msg:?}");
                            continue;
                        }
                        Err(e) => {
                            error!("invalid request, failed to convert to json:{e:?}");
                            break;
                        }
                    };

                    let mut trans_resp = TransferResponse {
                        file_hash: trans_req.file_hash.clone(),
                        sync_size: 0,
                    };

                    file_info = match storage_ctx.db.query_file_info(&trans_req.file_hash) {
                        Some(file_info) => {
                            debug!("transferring partial file:{file_info:?}");
                            trans_resp.sync_size = file_info.sync_size;
                            file_info
                        }
                        None => {
                            debug!("transferring new file:{}, size:{}", trans_req.file_hash, trans_req.file_size);
                            let file_info = SyncFileInfo {
                                file_hash: trans_req.file_hash.clone(),
                                file_dir: trans_req.file_dir.clone(),
                                file_name: trans_req.file_name.clone(),
                                file_size: trans_req.file_size,
                                sync_size: 0,
                                file_meta: "".to_string(),
                            };

                            storage_ctx.db.save_file_info(&file_info)?;
                            file_info
                        }
                    };

                    let writer = match storage_ctx.file_storage.open_writer(&file_info) {
                        Ok(writer) => writer,
                        Err(e) => {
                            error!("failed to open writer: {e:?}");
                            break;
                        }
                    };

                    Self::finalize_writer_if_needed(file_writer, &storage_ctx, &file_info)?;

                    file_writer = Some(writer);
                    sender.send(TransferControlMessage::Response(trans_resp).into()).await.unwrap();
                }

                Ok(Message::Binary(data)) => match &mut file_writer {
                    Some(writer) => {
                        file_info.sync_size += writer.write(&data)?;

                        // debug!("transferring, len:{}, {}/{}", data.len(), file_info.sync_size, file_info.file_size);
                        if file_info.sync_size >= file_info.file_size {
                            writer.close();
                            storage_ctx.db.update_sync_size(&file_info)?;
                            file_writer = None;
                            debug!("transfer completed, {}/{}", file_info.sync_size, file_info.file_size);
                            break;
                        }
                    }
                    None => {
                        warn!("received unexpected binary data:{}", data.len());
                    }
                },

                Ok(Message::Close(c)) => {
                    debug!("received close frame:{c:?}");
                    break;
                }

                Ok(msg) => {
                    debug!("received pingpong:{msg:?}");
                }

                Err(e) => {
                    error!("recv failed:{e}");
                    break;
                }
            }
        }

        Self::finalize_writer_if_needed(file_writer, &storage_ctx, &file_info)?;

        debug!("transfer task ended!");
        Ok(())
    }

    fn finalize_writer_if_needed(
        writer: Option<Box<dyn FileWriter>>,
        storage_ctx: &StorageContext,
        file_info: &SyncFileInfo,
    ) -> Result<()> {
        if let Some(mut writer) = writer {
            writer.close();
        }

        if !file_info.file_hash.is_empty() {
            storage_ctx.db.update_sync_size(file_info)?;
        }

        Ok(())
    }
}
