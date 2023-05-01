use anyhow::Result;
use vergen::EmitBuilder;

fn main() -> Result<()> {
    if let Ok(ci_build_id) = std::env::var("GITHUB_RUN_NUMBER") {
        println!("cargo:rustc-env=GITHUB_RUN_NUMBER={}", ci_build_id)
    } else {
        println!("cargo:rustc-env=GITHUB_RUN_NUMBER=N/A")
    };

    EmitBuilder::builder()
        .build_timestamp()
        .git_sha(false)
        .emit()?;

    Ok(())
}
