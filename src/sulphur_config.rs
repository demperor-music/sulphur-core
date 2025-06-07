use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

use crate::instance::Instance;
use crate::traits::{Saveable, SaveableDefaultPath};

#[derive(Serialize, Deserialize, Clone)]
pub struct SulphurConfig {
    pub gzdoom_command: OsString,
    pub instances: Vec<Instance>,
}

impl SulphurConfig {
    pub fn new() -> Self {
        Self {
            gzdoom_command: OsString::from("gzdoom"),
            instances: Vec::new(),
        }
    }

    pub fn initialize_saves_structure(&self) -> Result<(), std::io::Error> {
        let saves_dir = crate::savedir::Savedir::get()?;

        if let Some(parent) = saves_dir.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&saves_dir)?;

        Ok(())
    }

    fn find_assets_in_subdir(subdir: &str) -> Result<Vec<PathBuf>, std::io::Error> {
        let dir_path = match SulphurConfig::get_dir().find_data_file(subdir) {
            Some(path) => path,
            None => return Ok(vec![]),
        };

        let read_dir = match fs::read_dir(dir_path) {
            Ok(rd) => rd,
            Err(e) => return Err(e),
        };

        Ok(read_dir
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.is_file() { Some(path) } else { None }
                })
            })
            .collect())
    }

    pub fn get_command(&self) -> OsString {
        self.gzdoom_command.clone()
    }

    pub fn set_command(&mut self, new: OsString) {
        self.gzdoom_command = new;
    }

    pub fn get_instances(&self) -> Vec<&Instance> {
        self.instances.iter().collect()
    }

    pub fn get_instances_mut(&mut self) -> Vec<&mut Instance> {
        self.instances.iter_mut().collect()
    }

    pub fn get_indices_by_playtime(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self.get_played_instances();
        indices.sort_by(|&a, &b| {
            self.instances[b]
                .metadata
                .playtime
                .cmp(&self.instances[a].metadata.playtime)
        });
        indices
    }

    pub fn get_indices_by_last_played(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self.get_played_instances();
        indices.sort_by(|&a, &b| {
            match (
                &self.instances[a].metadata.last_played,
                &self.instances[b].metadata.last_played,
            ) {
                (Some(a_time), Some(b_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        indices
    }

    pub fn get_unplayed_instances(&self) -> Vec<usize> {
        self.get_instances_with_played_state(false)
    }

    pub fn get_played_instances(&self) -> Vec<usize> {
        self.get_instances_with_played_state(true)
    }

    pub fn get_instances_with_played_state(&self, played: bool) -> Vec<usize> {
        self.instances
            .iter()
            .enumerate()
            .filter(|(_, inst)| played != inst.metadata.last_played.is_none())
            .map(|(idx, _)| idx)
            .collect()
    }

    pub fn add_instance(&mut self, new_inst: Instance) -> usize {
        self.instances.push(new_inst);
        self.instances.len() - 1
    }

    pub fn remove_instance(&mut self, index: usize) {
        self.instances.swap_remove(index);
    }
}

impl Saveable for SulphurConfig {}
impl SaveableDefaultPath for SulphurConfig {
    fn get_dir() -> BaseDirectories {
        BaseDirectories::with_prefix("sulphur").unwrap()
    }

    fn get_filename() -> &'static str {
        "config.toml"
    }
}
