mod dat;
mod repository;
mod sync;

#[cfg(test)]
mod tests;

pub use dat::{canonical_title, normalize_title};
pub use repository::{ensure_local_index, options};
pub use sync::sync_all;

const AUTHORITY: &str = "libretro";
const LICENSE: &str = "CC-BY-SA-4.0";

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
