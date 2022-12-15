///! See https://docs.rs/vergen/latest/vergen/ for more info
use anyhow::Result;
use vergen::{Config, vergen, ShaKind};

fn main() -> Result<()> {
    let mut config = Config::default();

    *config.git_mut().sha_kind_mut() = ShaKind::Short;

    vergen(config)
}