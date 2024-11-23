use std::{io::Read, str::FromStr};

use anyhow::{anyhow, Error, Result};
use flate2::bufread::GzDecoder;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};

use crate::handler::download_with_progress;

use super::RepositoryHandler;

pub struct Debian {
    repositories: Vec<String>,
}

impl Default for Debian {
    fn default() -> Self {
        Self {
            repositories: vec![
                "http://deb.debian.org/debian/dists/stable/main/binary-amd64/Packages.gz"
                    .to_string(),
            ],
        }
    }
}

impl RepositoryHandler for Debian {
    async fn fetch_package_data(&self) -> Result<Vec<String>> {
        println!("Fetching package data from the Debian repository...");
        let repository_indexes: Vec<Vec<u8>> =
            download_with_progress(self.repositories.clone()).await?;

        println!("Decompressing package index files...");
        let mut out: Vec<String> = vec![];
        let indexes_iter: Vec<Vec<u8>> = repository_indexes;
        let pb: ProgressBar = ProgressBar::new(indexes_iter.len() as u64).with_finish(
            indicatif::ProgressFinish::WithMessage("Finished compresing!".into()),
        );
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

        for line in index_data.split('\n') {
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
    fn create_database(&self) {}
    fn store_packages(&self) {}
    fn sync_repository(&self) {}
}

#[derive(Debug, Default, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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
