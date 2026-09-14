mod catalog;
mod repository;
mod scanner;
#[cfg(test)]
mod tests;

pub use super::platforms::{PlatformSpec, PLATFORM_SPECS};
pub use crate::models::game::CatalogEntry;
pub struct GameService;
