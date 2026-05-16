use crate::{Error, Result};
use iocore::Path;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Workspace {
    resolver: String,
    members: Vec<String>,

    #[serde(flatten)]
    pub meta: toml::Value,
}
impl Workspace {
    pub fn members(&self, cargo_toml_path: &Path) -> Result<Vec<Manifest>> {
        let mut manifests = Vec::<Manifest>::new();
        for manifest_path in self
            .members
            .iter()
            .map(|subdir| cargo_toml_path.join(subdir).join("Cargo.toml"))
        {
            match Manifest::from_path(&manifest_path) {
                Ok(manifest) => {
                    manifests.push(manifest);
                }
                Err(error) => {
                    eprintln!("WARNING: error reading manifest from {manifest_path}: {error}");
                }
            }
        }
        Ok(manifests)
    }
}

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
    pub package: Option<Package>,
    pub workspace: Option<Workspace>,

    pub bin: Option<Vec<ExecutableAsset>>,
    pub example: Option<Vec<ExecutableAsset>>,

    #[serde(flatten)]
    pub meta: toml::Value,
}
impl ManifestData {
    pub fn bin(&self, path: &Path) -> Vec<ExecutableAsset> {
        let mut bins = self.bin.clone().unwrap_or_default();
        if let Some(assets) = self
            .workspace
            .clone()
            .map(|workspace| {
                workspace
                    .manifests(path)
                    .map(|manifest| manifest.data.bin(path))
                    .unwrap_or_default()
            })
            .unwrap_or_default()
        {
            bins.extend(&assets);
        }
        bins
    }
    pub fn example(&self, path: &Path) -> Vec<ExecutableAsset> {
        let mut examples = self.example.clone().unwrap_or_default();
        if let Some(assets) = self
            .workspace
            .clone()
            .map(|workspace| {
                workspace
                    .members(path)
                    .map(|manifest| manifest.data.example(path))
                    .unwrap_or_default()
            })
            .unwrap_or_default()
        {
            examples.extend(&assets);
        }
        examples
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
            .bin(&self.path)
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
            .example(&self.path)
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
