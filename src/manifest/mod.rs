use std::{
    fmt::{Debug, Display},
    fs::File,
    io::stdout,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Context, Error, Result};

use self::document::{CfgGetter, DependenciesGetter, ExcludeTable, Table};
use crate::cli::bufwrite;

pub(crate) mod document;

pub(crate) struct Manifest<Document> {
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

impl<Document> Manifest<Document>
where
    Document: self::document::Document + FromStr,
    <Document as FromStr>::Err: Debug + Display + Send + Sync + 'static,
{
    pub(crate) fn build(path: impl Into<PathBuf>, text: &str) -> Result<Self> {
        let path = path.into();
        let document = text
            .parse::<Document>()
            .map_err(Error::msg)
            .with_context(|| format!("failed to parse {:?} as manifest", path))?;

        Ok(Self {
            path,
            document,
            converted: false,
        })
    }
}

impl<'a, Document> ManifestDocument<'a> for Manifest<Document>
where
    Document: CfgGetter + DependenciesGetter + Display,
{
    const DEPENDENCY_TABLE_KEYS: [&'a str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    fn convert(&mut self) {
        let target_cfgs = self.document.get_cfgs();

        for key in Self::DEPENDENCY_TABLE_KEYS {
            let Some(dependencies) = self.document.get_dependencies(key) else {
                continue;
            };

            for (name, item) in dependencies.exclude_table() {
                dependencies.insert(&name, item);
                self.converted = true;
            }

            dependencies.sort_values();
            dependencies.set_implicit(true);

            for cfg in &target_cfgs {
                let Some(dependencies) = self.document.get_target_dependencies(cfg, key) else {
                    continue;
                };

                for (name, item) in dependencies.exclude_table() {
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
