use std::{fs, io::{BufWriter, Write, stdout}, path::PathBuf};

use anyhow::Result;
use clap::{Args, Parser};

use crate::{
    manifest::{Manifest, ManifestDocument},
    metadata::{Metadata, PackageFinder},
};

#[derive(Debug, Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub enum Command {
    TableDeps(TableDepsOption),
}

/// Convert dependencies list from inline table to table in the Cargo.toml manifest file
#[derive(Debug, Args)]
#[command(version)]
pub struct TableDepsOption {
    /// Path to Cargo.toml
    #[arg(long, value_name = "PATH")]
    manifest_path: Option<PathBuf>,

    /// Package to convert dependencies list from inline table to table
    #[arg(long, short = 'p', value_name = "SPEC", conflicts_with = "workspace")]
    package: Vec<String>,

    /// Convert dependencies list from inline table to table in all packages in the workspace
    #[arg(long, conflicts_with = "package")]
    workspace: bool,

    /// Exclude packages from converting
    #[arg(long, value_name = "SPEC", requires = "workspace")]
    exclude: Vec<String>,

    /// Print manifest converted
    #[arg(long)]
    dry_run: bool,
}

pub(crate) trait TableDepsOptionManifest {
    fn manifest_path(&self) -> Option<&PathBuf>;
}

pub(crate) trait TableDepsOptionPackage {
    fn package(&self) -> &[String];
}

pub(crate) trait TableDepsOptionWorkspace {
    fn exclude(&self) -> &[String];
    fn workspace(&self) -> bool;
}

pub(crate) trait TableDepsOptionDryRun {
    fn dry_run(&self) -> bool;
}

impl TableDepsOptionManifest for TableDepsOption {
    fn manifest_path(&self) -> Option<&PathBuf> {
        self.manifest_path.as_ref()
    }
}

impl TableDepsOptionPackage for TableDepsOption {
    fn package(&self) -> &[String] {
        &self.package
    }
}

impl TableDepsOptionWorkspace for TableDepsOption {
    fn exclude(&self) -> &[String] {
        &self.exclude
    }

    fn workspace(&self) -> bool {
        self.workspace
    }
}

impl TableDepsOptionDryRun for TableDepsOption {
    fn dry_run(&self) -> bool {
        self.dry_run
    }
}

pub(crate) fn execute() -> Result<()> {
    let Command::TableDeps(ref option) = Command::parse();

    let metadata = Metadata::build(option.manifest_path().unwrap_or(&PathBuf::from("Cargo.toml")))?;
    let packages = metadata.get_packages(option);

    let stdout = stdout();
    let mut buffer = BufWriter::new(&stdout);

    for package in packages {
        let manifest_path = &package.manifest_path;
        let text = fs::read_to_string(manifest_path)?;
        let mut manifest = Manifest::build(manifest_path, &text)?;

        manifest.convert();

        if manifest.converted {
            writeln!(buffer, "Updated {:?}", PathBuf::from(manifest_path))?;
        }

        if option.dry_run() {
            manifest.print()?;
            continue;
        }

        manifest.write()?;
    }

    if buffer.buffer().is_empty() {
        writeln!(&stdout, "no updates")?;
    }

    buffer.flush()?;
    Ok(())
}

pub(crate) fn bufwrite(inner: impl Write, content: impl Into<String>) -> Result<()> {
    let mut buffer = BufWriter::new(inner);
    write!(buffer, "{}", content.into())?;
    buffer.flush()?;

    Ok(())
}
