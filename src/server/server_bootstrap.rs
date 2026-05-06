use std::process::{Child, Command};
use std::sync::Arc;
use anyhow::Context;
use axum::extract::State;
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use colored::Colorize;
use hyper::header;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::set_header::SetResponseHeaderLayer;
use crate::project::NucleusProject;
use crate::server::{FrontendMode, NucleusServer};
use crate::server::proxy::{proxy_handler, ProxyClient};

#[derive(Clone)]
pub struct ServerState {
    pub project: NucleusProject,
    pub proxy_client: Option<ProxyClient>,
    pub dev_proxy_port: Option<u16>
}

impl NucleusServer {
    pub async fn run(self) -> anyhow::Result<()> {
        let port = self.project.config.serve.port;
        let site_name = self.project.config.site.name.clone();
        let admin_path = self.project.config.admin.path.clone();

        let mut dev_proxy_port = None;
        let mut proxy_client: Option<ProxyClient> = None;

        if let Some(frontend_mode) = self.frontend {
            match frontend_mode {
                FrontendMode::Proxy(dir) => {
                    dev_proxy_port = Some(9889);
                    proxy_client = Some(Client::builder(hyper_util::rt::TokioExecutor::new())
                        .build(HttpConnector::new()));
                }
                FrontendMode::Static(_) => {}
            }
        }

        if dev_proxy_port.is_some() {
            let mut child = spawn_framework_server(&self.project, dev_proxy_port.unwrap())?;

            tokio::spawn(async move {
                signal::ctrl_c().await.ok();
                child.kill().ok();
                std::process::exit(0);
            });
        }

        let state = Arc::new(ServerState {
            project: self.project,
            proxy_client,
            dev_proxy_port,
        });



        let app = Router::new()
            .route("/health", get(handler))
            .fallback(proxy_handler)
            .layer(SetResponseHeaderLayer::overriding(
                header::SERVER,
                HeaderValue::from_static("nucleus-cms/1.0")
            ))
            .with_state(state);

        let listener = TcpListener::bind(("0.0.0.0", port)).await?;

        println!();
        println!("  {} {}", "◆ Nucleus".cyan().bold(), "is ready".dimmed());
        println!();
        println!("  {}  http://localhost:{}", "site ".dimmed(), port);
        println!("  {}  http://localhost:{}{}", "admin".dimmed(), port, admin_path);
        println!();

        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handler(State(state): State<Arc<ServerState>>) -> String {
    format!("Hello {}", state.project.config.site.name)
}



pub fn spawn_framework_server(project: &NucleusProject, port: u16) -> anyhow::Result<Child> {
    let child = Command::new("npm")
        .args(["run", "dev", "--", "--port", &port.to_string()])
        .current_dir(project.frontend_dir())
        .spawn()
        .context("Failed to spawn framework dev server - is Node.js installed?")?;

    Ok(child)
}