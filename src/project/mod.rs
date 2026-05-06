pub mod collection;
pub mod config;

use crate::project::collection::Collection;
use crate::project::config::NucleusConfig;
use anyhow::Context;
use std::path::PathBuf;

#[derive(Clone)]
pub struct NucleusProject {
    pub root: PathBuf,
    pub config: NucleusConfig,
    pub collections: Vec<Collection>,
}

impl NucleusProject {
    pub fn load() -> anyhow::Result<Self> {
        let root =
            find_project_root().context("not inside a Nucleus project = run nucleus init first")?;

        Self::load_from(root)
    }

    pub fn load_from(root: PathBuf) -> anyhow::Result<Self> {
        let config = NucleusConfig::load(&root.join("nucleus/nucleus.toml"))?;
        let collections = Collection::load_all(&root.join("nucleus/collections"))?;

        Ok(Self {
            root,
            config,
            collections,
        })
    }

    pub fn nucleus_dir(&self) -> PathBuf {
        self.root.join("nucleus")
    }

    pub fn collections_dir(&self) -> PathBuf {
        self.root.join("collections")
    }

    pub fn content_dir(&self) -> PathBuf {
        self.root.join("content")
    }

    pub fn content_dir_for(&self, collection: &str) -> PathBuf {
        self.content_dir().join(collection)
    }

    pub fn media_dir(&self) -> PathBuf {
        self.content_dir().join("media")
    }

    pub fn frontend_dir(&self) -> PathBuf {
        self.root.join("frontend")
    }

    pub fn db_path(&self) -> PathBuf {
        self.nucleus_dir().join("nucleus.db")
    }

    pub fn out_dir(&self) -> PathBuf {
        self.root.join(&self.config.build.out_dir)
    }
}

fn find_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join("nucleus/nucleus.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}
