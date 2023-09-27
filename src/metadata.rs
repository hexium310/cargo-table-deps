use std::path::PathBuf;

use anyhow::Result;
use cargo_metadata::Package;

use crate::cli::{TableDepsOptionPackage, TableDepsOptionWorkspace};

pub(crate) struct Metadata(cargo_metadata::Metadata);

pub(crate) trait PackageFinder<Option>
where
    Option: TableDepsOptionPackage + TableDepsOptionWorkspace,
{
    fn get_packages(&self, option: &Option) -> Vec<&Package>;
}

impl Metadata {
    pub(crate) fn build(manifest_path: impl Into<PathBuf>) -> Result<Self> {
        let mut command = cargo_metadata::MetadataCommand::new();
        command.manifest_path(manifest_path);
        command.no_deps();

        let metadata = command.exec()?;
        Ok(Self(metadata))
    }
}

impl<Option> PackageFinder<Option> for Metadata
where
    Option: TableDepsOptionPackage + TableDepsOptionWorkspace,
{
    fn get_packages(&self, option: &Option) -> Vec<&Package> {
        let metadata = &self.0;

        if !option.workspace() && option.package().is_empty() {
            return metadata.workspace_default_packages();
        }

        if option.workspace() {
            return metadata
                .workspace_packages()
                .into_iter()
                .filter(|package| !option.exclude().contains(&package.name))
                .collect::<Vec<_>>();
        }

        metadata
            .workspace_packages()
            .into_iter()
            .filter(|package| option.package().contains(&package.name))
            .collect::<Vec<_>>()
    }
}
