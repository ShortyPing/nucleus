use std::path::Path;
use std::process::Command;
use anyhow::{bail, Context};
use colored::Colorize;
use tracing::{warn};

pub fn scaffold(project_path: &Path) -> anyhow::Result<()> {
    create_next_app(&project_path)?;
    patch_next_config(project_path)?;
    Ok(())
}

fn create_next_app(project_path: &Path) -> anyhow::Result<()> {
    println!("  {} Next.js frontend", "scaffolding".cyan());

    let status = Command::new("npx")
        .args([
            "create-next-app@latest",
            "frontend",
            "--ts",
            "--tailwind",
            "--eslint",
            "--disable-git",
            "--app",
            "--no-src-dir",
            "--import-alias", "@/*",
            "--yes"
        ]).current_dir(&project_path).status().context("Failed to run create-next-app - is Node.js installed?")?;

    if !status.success() {
        bail!("create-next-app failed with status {}", status);
    }

    println!("  {} Next.js frontend", "scaffolded".green());
    Ok(())
}

fn patch_next_config(project_path: &Path) -> anyhow::Result<()> {
    let config_path = project_path.join("frontend/next.config.ts");

    let original = std::fs::read_to_string(&config_path)?;

    let patched = original.replace(r#"const nextConfig: NextConfig = {
  /* config options here */
};
"#, r#"
const nextConfig: NextConfig = {
    // Nucleus: static export required for nucleus
    output: "export",
};
    "#.trim());

    if patched == original {
        warn!("Could not patch next.config.ts automatically - \
        please add `output: 'export'` manually")
    }

    std::fs::write(&config_path, patched)?;

    println!("  {} next.config.ts (output: export)", "patched".green());
    Ok(())
}