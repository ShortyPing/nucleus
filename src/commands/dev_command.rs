use crate::project::NucleusProject;
use crate::server::{AuthMode, NucleusServer};

pub async fn handle_dev_command() -> anyhow::Result<()> {
    let project = NucleusProject::load()?;
    let project_root = &project.root;


    let server = NucleusServer::new(project.clone())
        .with_content_api()
        .with_admin(AuthMode::Disabled)
        .with_proxy(project_root.join("frontend"));

    server.run().await?;
    
    Ok(())
}