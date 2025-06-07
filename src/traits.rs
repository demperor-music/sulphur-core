use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

use crate::SulphurConfig;
use crate::asset::Asset;

pub trait Argument {
    fn get_prefix() -> &'static OsStr;
}

pub trait Movable {
    fn get_dir_name() -> &'static str;

    fn get_full_dir() -> Result<PathBuf, std::io::Error> {
        let dest_dir = match SulphurConfig::get_dir().place_data_file(Self::get_dir_name()) {
            Ok(dir) => {
                if let Some(parent) = dir.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                dir.parent().unwrap().join(Self::get_dir_name())
            }
            Err(e) => return Err(e),
        };

        fs::create_dir_all(&dest_dir)?;
        Ok(dest_dir)
    }

    fn move_file(&mut self, keep_original: bool) -> Result<(), std::io::Error>
    where
        Self: AsMut<Asset>,
    {
        let asset = self.as_mut();
        let source_path = &asset.path;

        let dest_dir = Self::get_full_dir()?;

        let filename = match source_path.file_name() {
            Some(name) => name,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Cannot resolve file name",
                ));
            }
        };

        let dest_path = dest_dir.join(filename);

        let result = if keep_original {
            fs::copy(source_path, &dest_path).map(|_| ())
        } else {
            fs::rename(source_path, &dest_path)
        };
        let success = result.is_ok();

        if success {
            asset.path = Self::get_relative_path(filename);
        }

        result
    }

    fn get_relative_path(filename: &OsStr) -> PathBuf {
        PathBuf::from(Self::get_dir_name()).join(filename)
    }

    fn get_filename(&self) -> Option<&OsStr>
    where
        Self: AsRef<Asset>,
    {
        self.as_ref().path.file_name()
    }

    fn get_absolute_path(&self) -> Option<PathBuf>
    where
        Self: AsRef<Asset>,
    {
        let asset = self.as_ref();
        let path = &asset.path;

        if path.is_absolute() {
            return Some(path.clone());
        }

        if let Ok(base_dir) = SulphurConfig::get_dir().place_data_file("") {
            return Some(base_dir.join(path));
        }

        None
    }
}

pub trait Saveable: Serialize + for<'de> Deserialize<'de> {
    fn as_toml(&self) -> Result<String> {
        toml::to_string_pretty(&self).context("Failed to serialize to TOML")
    }

    fn from_toml(toml: String) -> Result<Self> {
        toml::from_str(&toml).context("Failed to deserialize from TOML")
    }

    fn save_as(&self, path: PathBuf) -> Result<()> {
        let toml_str = self.as_toml()?;

        fs::write(&path, toml_str)
            .with_context(|| format!("Failed to write to {}", path.display()))?;
        Ok(())
    }
    fn load_from(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read from {}", path.display()))?;

        Self::from_toml(content)
    }
}

pub trait SaveableDefaultPath: Saveable {
    fn get_dir() -> BaseDirectories;
    fn get_filename() -> &'static str;

    fn save(&self) -> Result<()> {
        let path = Self::get_dir()
            .place_config_file(Self::get_filename())
            .context("Failed to determine file path")?;

        self.save_as(path)
    }

    fn load() -> Result<Self> {
        let path = Self::get_dir()
            .find_config_file(Self::get_filename())
            .context("File not found")?;
        Self::load_from(path)
    }
}
