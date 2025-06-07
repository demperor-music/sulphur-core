use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use zip::{ZipArchive, ZipWriter, write::FileOptions};

use crate::{SaveableDefaultPath, SulphurConfig};
use crate::game_data::GameData;
use crate::metadata::Metadata;
use crate::traits::{Movable, Saveable};
use crate::asset::{Iwad, Mod};
use crate::savedir::Savedir;

#[derive(Serialize, Deserialize, Clone)]
pub struct Instance {
    pub metadata: Metadata,
    pub gamedata: GameData,
}

impl Instance {
    pub const FILENAME: &'static str = "instance.toml";
    pub fn get_full_command(&self, gzdoom: &OsStr) -> OsString {
        let mut full_command = OsString::new();
        full_command.push(gzdoom);
        full_command.push(" ");
        full_command.push(self.gamedata.get_parameters());
        full_command
    }

    pub fn run(&mut self, full_command: OsString) {
        self.metadata.last_played = Some(time::SystemTime::now());

        {
            #[cfg(target_family = "windows")]
            {
                let mut cmd = Command::new("cmd");
                cmd.arg("/C");
                cmd
            }
            #[cfg(not(target_family = "windows"))]
            {
                let mut cmd = Command::new("sh");
                cmd.arg("-c");
                cmd
            }
        }
        .arg(full_command)
        .output()
        .expect("failed to execute process");

        if let Some(t) = self.metadata.last_played {
            self.metadata.last_session_duration = t.elapsed().ok();
        }

        if let Some(t) = self.metadata.last_session_duration {
            self.metadata.playtime += t;
        }
    }

    pub fn save_brimpkg(&self, path: &Path, transfer_saves: bool, transfer_playtime: bool) -> Result<File> {
        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);
        let mut new_instance = self.clone();

        for dirname in [Mod::get_dir_name(), Iwad::get_dir_name()] {
            zip.add_directory(dirname, FileOptions::default())?;
        }

        new_instance.initialize_savedir()?;
        if transfer_saves {
            let saves_dir = self.gamedata.savedir.clone();
            let new_saves_dir = new_instance.gamedata.get_savedir().to_string_lossy();
            zip.add_directory(new_saves_dir.to_string(), FileOptions::default())?;
            for entry in fs::read_dir(&saves_dir)? {
                let entry = entry?;
                let path = entry.path();
                let filename = path.file_name().unwrap();
                let filename = filename.to_string_lossy();
                let dest_path = format!("{}/{}", &new_saves_dir, filename);
                zip.start_file(&dest_path, FileOptions::default())?;
                let mut buffer = Vec::new();
                fs::File::open(&path)?.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
        }

        if !transfer_playtime {
            new_instance.metadata.playtime = std::time::Duration::new(0, 0);
            new_instance.metadata.last_played = None;
            new_instance.metadata.last_session_duration = None;
        }

        write_assets_to_zip(&mut zip, new_instance.gamedata.mods.as_mut_slice())?;
        write_assets_to_zip(&mut zip, new_instance.gamedata.iwads.as_mut_slice())?;

        zip.start_file(Self::FILENAME, FileOptions::default())?;
        zip.write_all(new_instance.as_toml()?.as_bytes())?;

        Ok(zip.finish()?)
    }

    pub fn load_brimpkg(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut instance_toml_content = String::new();
        {
            let mut instance_file = archive
                .by_name(Self::FILENAME)
                .context("instance.toml not found in brimpkg")?;
            instance_file.read_to_string(&mut instance_toml_content)?;
        }

        let mut instance: Instance = Self::from_toml(instance_toml_content)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = PathBuf::from(file.name());

            if file.is_dir() || file.name() == Self::FILENAME {
                continue;
            }
            println!("loading {} from zip...", &file_path.display());

            if let Some(parent) = file_path.parent() {
                if let Some(filename) = file_path.file_name() {
                    let dest_dir = SulphurConfig::get_dir().place_data_file(parent)?;
                    let dest_path = dest_dir.join(filename);
                    fs::create_dir_all(dest_dir)?;

                    // Skip if file already exists
                    if !dest_path.exists() {
                        let mut dest_file = File::create(&dest_path)?;
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer)?;
                        std::io::Write::write_all(&mut dest_file, &buffer)?;
                    } else {
                        println!("{} found in Sulphur directory, skipping...", &dest_path.display());
                    }
                }
            }
        }
        // Update asset paths to point to the extracted files
        for iwad in &mut instance.gamedata.iwads {
            if let Some(filename) = iwad.get_filename() {
                iwad.as_mut().path = Iwad::get_relative_path(filename);
            }
        }
        for mod_asset in &mut instance.gamedata.mods {
            if let Some(filename) = mod_asset.get_filename() {
                mod_asset.as_mut().path = Mod::get_relative_path(filename);
            }
        }

        Ok(instance)
    }

    pub fn create_savedir(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.gamedata.savedir.clone())
    }

    pub fn initialize_savedir(&mut self) -> Result<()> {
        let instance_save_dir = format!("{}/{}", Savedir::get_dir_name(), &self.metadata.name);
        self.gamedata.savedir = PathBuf::from(instance_save_dir);
        Ok(())
    }
}

fn write_assets_to_zip<T>(zip: &mut ZipWriter<File>, assets: &mut [T]) -> Result<()>
where
    T: Movable + AsMut<crate::asset::Asset> + AsRef<crate::asset::Asset>,
{
    for asset in assets {
        let filename = asset
            .get_filename()
            .ok_or_else(|| anyhow::anyhow!("Asset filename is missing"))?;
        let absolute_path = asset
            .get_absolute_path()
            .ok_or_else(|| anyhow::anyhow!("Asset path is invalid"))?;

        {
            let zip_path = format!("{}", T::get_relative_path(filename).to_string_lossy());
            println!("Writing {} to zip...", &zip_path);
            zip.start_file(zip_path, FileOptions::default())?;
        }
        asset.as_mut().path = T::get_relative_path(filename);

        let content = fs::read(&absolute_path)?;
        zip.write_all(&content)?;
    }
    Ok(())
}

impl Movable for Instance {
    fn get_dir_name() -> &'static str {
        "instances"
    }
}
impl Saveable for Instance {}
