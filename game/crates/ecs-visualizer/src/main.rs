mod dashboard;
mod query;

use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut arguments = env::args_os().skip(1);
    let camera_path = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("../game/src/camera.rs"));
    let output_path = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("index.html"));

    if arguments.next().is_some() {
        bail!("usage: ecs-visualizer [camera.rs] [output.html]");
    }

    let camera_source = fs::read_to_string(&camera_path)
        .with_context(|| format!("failed to read {}", camera_path.display()))?;
    let queries = query::extract(&camera_source).context("failed to parse camera queries")?;
    let html = dashboard::render(&queries).context("failed to render the dashboard")?;
    fs::write(&output_path, html)
        .with_context(|| format!("failed to write {}", output_path.display()))?;

    println!(
        "Generated {} camera queries in {} from {}",
        queries.len(),
        output_path.display(),
        camera_path.display()
    );
    Ok(())
}
