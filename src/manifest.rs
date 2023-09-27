use std::{fs::File, io::stdout, path::PathBuf};

use anyhow::{Context, Result};
use toml_edit::{Document, Item, Table};

use crate::cli::bufwrite;

pub(crate) struct Manifest {
    pub(crate) converted: bool,
    document: Document,
    path: PathBuf,
}

pub(crate) trait ManifestDocument<'a> {
    const DEPENDENCY_TABLE_KEYS: [&'a str; 3];

    fn convert(&mut self);
    fn print(&self) -> Result<()>;
    fn write(&self) -> Result<()>;
}

impl Manifest {
    pub(crate) fn build(path: impl Into<PathBuf>, text: &str) -> Result<Self> {
        let path = path.into();
        let document = text
            .parse::<Document>()
            .with_context(|| format!("failed to parse {:?} as manifest", path))?;
        Ok(Self {
            path,
            document,
            converted: false,
        })
    }
}

impl<'a> ManifestDocument<'a> for Manifest {
    const DEPENDENCY_TABLE_KEYS: [&'a str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    fn convert(&mut self) {
        let target_cfgs = self
            .document
            .get("target")
            .and_then(|item| item.as_table())
            .map(|table| table.iter().map(|(key, _)| key.to_string()).collect())
            .unwrap_or(vec![]);

        for key in Self::DEPENDENCY_TABLE_KEYS {
            let Some(dependencies) = self
                .document
                .get_mut(key)
                .and_then(|item| item.as_table_mut())
            else {
                continue;
            };

            for (name, item) in exclude_table(dependencies) {
                dependencies.insert(&name, item);
                self.converted = true;
            }

            dependencies.sort_values();
            dependencies.set_implicit(true);

            for cfg in &target_cfgs {
                let Some(dependencies) = self
                    .document
                    .get_mut("target")
                    .and_then(|item| item.get_mut(cfg))
                    .and_then(|item| item.get_mut(key))
                    .and_then(|item| item.as_table_mut())
                else {
                    continue;
                };

                for (name, item) in exclude_table(dependencies) {
                    dependencies.insert(&name, item);
                    self.converted = true;
                }

                dependencies.sort_values();
                dependencies.set_implicit(true);
            }
        }
    }

    fn print(&self) -> Result<()> {
        let stdout = stdout();
        bufwrite(stdout, self.document.to_string())?;
        Ok(())
    }

    fn write(&self) -> Result<()> {
        let file = File::create(&self.path)?;
        bufwrite(file, self.document.to_string())?;
        Ok(())
    }
}

fn exclude_table(table: &mut Table) -> Vec<(String, Item)> {
    table
        .iter()
        .filter_map(|(key, item)| {
            let Item::Value(value) = item else {
                return None;
            };

            match value {
                toml_edit::Value::String(version) => {
                    let mut table = toml_edit::Table::new();
                    table.insert("version", toml_edit::value(version.value()));

                    Some((key.to_string(), toml_edit::Item::Table(table)))
                },
                toml_edit::Value::InlineTable(inline_table) => Some((
                    key.to_string(),
                    toml_edit::Item::Table(inline_table.clone().into_table()),
                )),
                _ => None,
            }
        })
        .collect()
}
