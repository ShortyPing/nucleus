pub mod server_bootstrap;
pub mod proxy;

use std::path::PathBuf;
use anyhow::bail;
use crate::project::config::{TlsConfig, TlsStrategy};
use crate::project::NucleusProject;
pub enum AuthMode {
    Disabled,
    Required,
}

pub enum FrontendMode {
    Proxy(PathBuf),
    Static(PathBuf),
}


pub struct NucleusServer {
    pub project: NucleusProject,
    pub content_api: bool,
    pub admin: Option<AuthMode>,
    pub frontend: Option<FrontendMode>,
    pub tls: Option<TlsConfig>
}

impl NucleusServer {
    pub fn new(project: NucleusProject) -> Self {
        Self {
            project,
            content_api: false,
            admin: None,
            frontend: None,
            tls: None
        }
    }

    pub fn with_content_api(mut self) -> Self {
        self.content_api = true;
        self
    }

    pub fn with_admin(mut self, auth: AuthMode) -> Self {
        self.admin = Some(auth);
        self
    }

    pub fn with_proxy(mut self, frontend_dir: PathBuf) -> Self {
        self.frontend = Some(FrontendMode::Proxy(frontend_dir));
        self
    }

    pub fn with_static_files(mut self, out_dir: PathBuf) -> Self {
        self.frontend = Some(FrontendMode::Static(out_dir));
        self
    }

    pub fn with_tls(mut self, config: &TlsConfig) -> anyhow::Result<Self> {
        match config.strategy {
            TlsStrategy::Letsencrypt => {
                if config.domain.is_none() {
                    bail!("TLS strategy 'letsencrypt' requires a domain");
                }
            }
            TlsStrategy::File => {
                if config.certificate_file.is_none() || config.private_key_file.is_none() {
                    bail!("TLS strategy 'file' requires certificate_file and private_key_file");
                }
            }
        }
        self.tls = Some(config.clone());
        Ok(self)
    }
}