use crate::{Error, Result};
use iocore::Path;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Package {
    #[serde(rename(deserialize = "default-run"))]
    default_run: Option<String>,
    #[serde(flatten)]
    pub meta: toml::Value,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ExecutableAsset {
    name: String,
    path: Option<String>,
    #[serde(flatten)]
    pub meta: toml::Value,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ManifestData {
    pub package: Package,

    pub bin: Option<Vec<ExecutableAsset>>,
    pub example: Option<Vec<ExecutableAsset>>,

    #[serde(flatten)]
    pub meta: toml::Value,
}
impl ManifestData {
    pub fn bin(&self) -> Vec<ExecutableAsset> {
        self.bin.clone().unwrap_or_default()
    }
    pub fn example(&self) -> Vec<ExecutableAsset> {
        self.example.clone().unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Manifest {
    pub data: ManifestData,
    pub path: Path,
}

impl Manifest {
    pub fn project_dir(&self) -> Path {
        let path = &self.path;
        self.path
            .parent()
            .expect(&format!("could not get parent directory of {path}"))
    }
    pub fn bin_names(&self) -> Option<Vec<String>> {
        let names = self
            .data
            .bin()
            .iter()
            .map(|asset| asset.name.to_string())
            .collect::<Vec<String>>();
        let src = self.project_dir().join("src");
        let main = src.join("main.rs");
        if names.is_empty() && (!src.is_dir() || !main.is_file()) {
            None
        } else {
            Some(names)
        }
    }
    pub fn example_names(&self) -> Option<Vec<String>> {
        let mut names = self
            .data
            .example()
            .iter()
            .map(|asset| asset.name.to_string())
            .collect::<Vec<String>>();
        let examples = self.project_dir().join("examples");
        if names.is_empty() {
            names.extend(
                examples
                    .list()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|path| path.name())
                    .filter(|name| name.ends_with(".rs"))
                    .collect::<Vec<_>>(),
            );
        }
        (names.len() > 0).then_some(names)
    }
    pub fn from_path(path: &Path) -> Result<Manifest> {
        let data = toml::from_str::<ManifestData>(&path.read()?)?;
        let path = path.clone();
        Ok(Manifest { path, data })
    }
    pub fn default() -> Result<Manifest> {
        let path = Path::new("Cargo.toml").try_canonicalize();
        if !path.is_file() {
            Err(Error::IOError(format!("{path} is not a readable file")))
        } else {
            Ok(Manifest::from_path(&path)?)
        }
    }
}
