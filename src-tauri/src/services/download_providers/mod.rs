pub mod http;
pub mod bittorrent;

use crate::{errors::EmuBoxError, models::{TransferOutcome, TransferProgress, TransferRequest}};

pub trait DownloadProvider: Send + Sync {
    fn transfer(&self, request: &TransferRequest<'_>, progress: &mut dyn FnMut(TransferProgress) -> Result<(), EmuBoxError>) -> Result<TransferOutcome, EmuBoxError>;
}

pub fn provider(id: crate::models::ProviderId) -> Box<dyn DownloadProvider> {
    match id { crate::models::ProviderId::Http => Box::new(http::HttpProvider), crate::models::ProviderId::BitTorrent => Box::new(bittorrent::BitTorrentProvider::default()) }
}

pub fn io_error(error: impl std::fmt::Display) -> EmuBoxError { EmuBoxError::StorageUnavailable(error.to_string()) }

pub fn safe_name(name: &str) -> String {
    let name: String = name.chars().map(|character| if character.is_control() || matches!(character, '/' | '\\' | ':') { '_' } else { character }).take(180).collect();
    if name.is_empty() || name.starts_with('.') { "content.bin".into() } else { name }
}