// src/collection.rs

use std::path::Path;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct Collection {
    pub collection: CollectionMeta,
    #[serde(rename = "field")]
    pub fields: HashMap<String, Field>,
}

#[derive(Deserialize, Clone)]
pub struct CollectionMeta {
    pub label: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Field {
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub editable: bool,
    pub default: Option<String>,
    pub from: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Text,
    Richtext,
    Slug,
    Date,
    Datetime,
    Number,
    Boolean,
    Select,
    Tags,
    Image,
    Relation,
    User,
}

impl Collection {
    pub fn load_all(collections_dir: &Path) -> Result<Vec<Self>> {
        let mut collections = Vec::new();

        for entry in std::fs::read_dir(collections_dir)
            .context("Failed to read collections directory")?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                let text = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {:?}", path))?;
                let collection: Collection = toml::from_str(&text)
                    .with_context(|| format!("Failed to parse {:?}", path))?;
                collections.push(collection);
            }
        }

        Ok(collections)
    }
}