use std::{fs::File, io::BufWriter, path::{Path, PathBuf}};
use anyhow::{Result};
use bootloader::{BiosBoot, UefiBoot};
use serde::{Serialize, Deserialize};

trait Stored {
    fn path(&self) -> &Path;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    images: Vec<Image>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDeclaration {
    pub dep: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDefinition {
    pub declaration: ArtifactDeclaration,
    pub file_path: Box<Path>,
} 

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub kernel_artifact: ArtifactDefinition,
    pub image_file_path: Box<Path>,
    pub boot_kind: BootKind,
}

impl Build {
    pub fn new(dep: &str, artifacts_names: &[&str]) -> Self {
        Self { 
            images: artifacts_names.iter().map(|&bin_file_name| {
                let artifact = ArtifactDefinition::from_cargo_env(
                    ArtifactDeclaration { dep: dep.into(), name: bin_file_name.into() })
                    .expect("Expected to create artifact");
                [ 
                    artifact.build_image(BootKind::Bios).expect("Expected to build BIOS image"),
                    artifact.build_image(BootKind::Uefi).expect("Expected to build UEFI image")
                ]
            }).flatten().collect()
         }
    }

    pub fn write_info(&self) -> Result<()> {
        let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
        let file = File::create(out_dir.join("images.json"))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self)?;
        Ok(())
    }

    pub fn read_info() -> Result<Self> {
        let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
        let file = File::open(out_dir.join("images.json"))?;
        let build: Build = serde_json::from_reader(file)?;
        Ok(build)
    }

    pub fn images(&self) -> &[Image] {
        &self.images
    }
}

impl ArtifactDefinition {
    pub fn from_cargo_env(artifact_decl: ArtifactDeclaration) -> Result<Self> {
        let dep = artifact_decl.dep.replace("-", "_").to_uppercase();
        let name = artifact_decl.name.replace("-", "_");
        let file_path = PathBuf::from(std::env::var(
            format!("CARGO_BIN_FILE_{}_{}", dep, name))?);
        Ok(Self { declaration: artifact_decl, file_path: file_path.into() })
    }

    pub fn build_image(&self, build_kind: BootKind) -> Result<Image> {
        let out_image_path = PathBuf::from(std::env::var("OUT_DIR")?)
            .join(format!("{}-{}.img", self.declaration.name, match build_kind {
                BootKind::Uefi => "uefi",
                BootKind::Bios => "bios",
            }));
        Ok(match build_kind {
            BootKind::Uefi => {            
                UefiBoot::new(self.path()).create_disk_image(&out_image_path)?;
                Image { kernel_artifact: self.clone(), image_file_path: out_image_path.into(), boot_kind: BootKind::Uefi }
            },
            BootKind::Bios => {
                BiosBoot::new(self.path()).create_disk_image(&out_image_path)?;
                Image { kernel_artifact: self.clone(), image_file_path: out_image_path.into(), boot_kind: BootKind::Bios }
            }
        })
    }
}

impl Stored for ArtifactDefinition {
    fn path(&self) -> &Path {
        &self.file_path
    }       
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum, PartialEq, Eq)]
pub enum BootKind {
    Uefi,
    Bios,
}

impl Stored for Image {
    fn path(&self) -> &Path
    {
        &self.image_file_path
    }
}

#[cfg(test)]
mod tests {
    use envtestkit::lock::lock_test;
    use envtestkit::set_env;
    use std::ffi::OsString;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_build_artifact_from_cargo_envs() -> Result<()> {
        let _lock = lock_test();
        let file = NamedTempFile::new()?;
        let _test = set_env(
            OsString::from("CARGO_BIN_FILE_TEST_DEP_test_name"), file.path().to_str()
                    .expect("Expected to convert file path to string")
        );
        let artifact = ArtifactDefinition::from_cargo_env(
            ArtifactDeclaration { dep: "test-dep".into(), name: "test-name".into() })?;

        assert_eq!(artifact.declaration.dep, "test-dep");
        assert_eq!(artifact.declaration.name, "test-name");
        assert_eq!(artifact.path(), file.path());
        Ok(())
    }

    #[test]
    fn test_build_artifact_from_cargo_envs_missing_env() -> Result<()> {
        let _lock = lock_test();
        let result = ArtifactDefinition::from_cargo_env(
            ArtifactDeclaration { dep: "missing-dep".into(), name: "missing-name".into() });
        assert!(result.is_err());   
        Ok(())
    }
}