use std::{io::Read, str::FromStr};

use anyhow::{anyhow, Error, Result};
use flate2::bufread::GzDecoder;
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use rusqlite::{Connection, Statement};
use serde::{Deserialize, Serialize};

use crate::handler::download_with_progress;

use super::RepositoryHandler;

pub struct Debian {
    repositories: Vec<String>,
    connection: Option<Connection>,
}

impl Default for Debian {
    fn default() -> Self {
        Self {
            repositories: vec![
                "http://deb.debian.org/debian/dists/stable/main/binary-amd64/Packages.gz"
                    .to_string(),
            ],
            connection: None,
        }
    }
}

impl RepositoryHandler for Debian {
    type Package = DebianPackage;

    async fn fetch_package_data(&self) -> Result<Vec<String>> {
        let repository_indexes: Vec<Vec<u8>> =
            download_with_progress(self.repositories.clone()).await?;

        println!("Decompressing package index files...");
        let mut out: Vec<String> = vec![];
        let indexes_iter: Vec<Vec<u8>> = repository_indexes;
        let pb: ProgressBar = ProgressBar::new(indexes_iter.len() as u64)
            .with_style(
                ProgressStyle::default_bar()
                    .template("{msg} {bar:60} {percent}% {pos}/{len}")?
                    .progress_chars("█▓▒░"),
            )
            .with_finish(ProgressFinish::WithMessage(
                "Uncompressed package indexes!".into(),
            ))
            .with_message("Uncompressing package indexes...");
        for data in pb.wrap_iter(indexes_iter.iter()) {
            let mut d: GzDecoder<&[u8]> = GzDecoder::new(data.as_slice());
            let mut s: String = String::new();
            d.read_to_string(&mut s).unwrap();
            out.push(s);
        }
        Ok(out)
    }
    fn parse_packages(&self, index_data: String) -> Vec<DebianPackage> {
        let mut packages: Vec<DebianPackage> = vec![];
        let mut current_package: Option<DebianPackage> = None;

        let unparsed_packages: Vec<&str> = index_data.split('\n').collect();
        let pb: ProgressBar = ProgressBar::new(unparsed_packages.len() as u64)
            .with_style(
                ProgressStyle::default_bar()
                    .template("{msg} {bar:60} {percent}% {pos}/{len}")
                    .unwrap()
                    .progress_chars("█▓▒░"),
            )
            .with_finish(ProgressFinish::WithMessage(
                "Parsed Debian packages!".into(),
            ))
            .with_message("Parsing Debian packages...");
        for line in pb.wrap_iter(unparsed_packages.iter()) {
            if line.starts_with("Package: ") {
                if let Some(package) = current_package {
                    packages.push(package)
                }
                current_package = Some(DebianPackage::default());
                current_package.as_mut().unwrap().package =
                    line.split(':').nth(1).unwrap().trim().to_string();
            } else if line.starts_with("Version: ") {
                current_package.as_mut().unwrap().version =
                    line.split(':').nth(1).unwrap().trim().to_string();
            } else if line.starts_with("Architecture: ") {
                current_package.as_mut().unwrap().architecture = line
                    .split(':')
                    .nth(1)
                    .unwrap()
                    .trim()
                    .to_string()
                    .parse()
                    .unwrap();
            } else if line.starts_with("Description: ") {
                current_package.as_mut().unwrap().description =
                    Some(line.split(':').nth(1).unwrap().trim().to_string());
            }
        }

        if let Some(package) = current_package {
            packages.push(package)
        }

        packages
    }
    fn create_database(&mut self) -> Result<()> {
        self.connection = Some(Connection::open("trek_debian.db")?);
        self.connection.as_ref().unwrap().execute(
            r#"
        CREATE TABLE IF NOT EXISTS packages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            architecture TEXT NOT NULL,
            description TEXT
        );
        "#,
            (),
        )?;

        Ok(())
    }
    fn store_packages(&self, packages: Vec<DebianPackage>) -> Result<()> {
        let mut statement: Statement<'_> = self.connection.as_ref().expect("`create_database` was not called before `store_packages`!").prepare("INSERT INTO packages (name, version, architecture, description) VALUES (?1, ?2, ?3, ?4)")?;

        let pb: ProgressBar = ProgressBar::new(packages.len() as u64)
            .with_style(
                ProgressStyle::default_bar()
                    .template("{bar:60} {percent}% {pos}/{len}")?
                    .progress_chars("█▓▒░"),
            )
            .with_finish(ProgressFinish::Abandon);
        for package in pb.wrap_iter(packages.iter()) {
            statement.insert((
                package.package.clone(),
                package.version.clone(),
                package.architecture.to_string(),
                package.description.clone(),
            ))?;
        }
        Ok(())
    }
    async fn sync_repository(&mut self) -> Result<()> {
        self.create_database()?;
        let index_data: Vec<String> = self.fetch_package_data().await?;
        let repositories: Vec<Vec<DebianPackage>> = {
            let mut packages: Vec<Vec<DebianPackage>> = vec![];
            for repository in index_data {
                packages.push(self.parse_packages(repository));
            }
            packages
        };
        let pb: ProgressBar = ProgressBar::new(repositories.len() as u64)
            .with_style(
                ProgressStyle::default_bar()
                    .template("{msg} {bar:60} {percent}% {pos}/{len}")?
                    .progress_chars("█▓▒░"),
            )
            .with_finish(ProgressFinish::WithMessage("Stored packages!".into()))
            .with_message("Storing packages from repositories...");
        for repository in pb.wrap_iter(repositories.iter()) {
            self.store_packages(repository.to_vec())?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub enum DebianArchitecture {
    #[default]
    All,
    Amd64,
    Arm64,
    ArmEl,
    ArmHf,
    I386,
    Mips64El,
    MipsEl,
    Ppc64El,
    S390X,
    Source,
}

impl FromStr for DebianArchitecture {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "all" => Ok(DebianArchitecture::All),
            "amd64" => Ok(DebianArchitecture::Amd64),
            "arm64" => Ok(DebianArchitecture::Arm64),
            "armel" => Ok(DebianArchitecture::ArmEl),
            "armhf" => Ok(DebianArchitecture::ArmHf),
            "i386" => Ok(DebianArchitecture::I386),
            "mips64el" => Ok(DebianArchitecture::Mips64El),
            "mipsel" => Ok(DebianArchitecture::MipsEl),
            "ppc64el" => Ok(DebianArchitecture::Ppc64El),
            "s390x" => Ok(DebianArchitecture::S390X),
            "source" => Ok(DebianArchitecture::Source),
            _ => Err(anyhow!("Could not match into DebianArchitecture!")),
        }
    }
}

impl ToString for DebianArchitecture {
    fn to_string(&self) -> String {
        match self {
            DebianArchitecture::All => "all".to_string(),
            DebianArchitecture::Amd64 => "amd64".to_string(),
            DebianArchitecture::Arm64 => "arm64".to_string(),
            DebianArchitecture::ArmEl => "armel".to_string(),
            DebianArchitecture::ArmHf => "armhf".to_string(),
            DebianArchitecture::I386 => "i386".to_string(),
            DebianArchitecture::Mips64El => "mips64el".to_string(),
            DebianArchitecture::MipsEl => "mipsel".to_string(),
            DebianArchitecture::Ppc64El => "ppc64el".to_string(),
            DebianArchitecture::S390X => "s390x".to_string(),
            DebianArchitecture::Source => "source".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DebianPackage {
    package: String,
    architecture: DebianArchitecture,
    version: String,
    description: Option<String>,
}

impl Default for DebianPackage {
    fn default() -> Self {
        Self {
            package: "".to_string(),
            architecture: DebianArchitecture::default(),
            version: "".to_string(),
            description: None,
        }
    }
}
