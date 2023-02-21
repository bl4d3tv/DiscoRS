use anyhow::Result;
use vergen::{vergen, Config};

fn main() -> Result<()> {
    if let Ok(ci_build_id) = std::env::var("GITHUB_RUN_NUMBER") {
        println!("cargo:rustc-env=DISCO_GITHUB_RUN_NUMBER={}", ci_build_id)
    } else {
        println!("cargo:rustc-env=DISCO_GITHUB_RUN_NUMBER=N/A")
    };

    vergen(Config::default())
}
