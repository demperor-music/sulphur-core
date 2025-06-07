use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;

use crate::{SaveableDefaultPath, SulphurConfig};
use crate::asset::{Iwad, Mod};
use crate::utils::get_arguments;

#[derive(Serialize, Deserialize, Clone)]
pub struct GameData {
    pub iwads: Vec<Iwad>,
    pub mods: Vec<Mod>,
    pub savedir: PathBuf,
    pub additional_params: Vec<OsString>,
}

impl GameData {
    pub fn get_iwad_parameters(&self) -> OsString {
        get_arguments(&self.iwads)
            .collect::<Vec<_>>()
            .join(OsStr::new(" "))
            .into()
    }

    pub fn get_mods_parameters(&self) -> OsString {
        get_arguments(&self.mods)
            .collect::<Vec<_>>()
            .join(OsStr::new(" "))
            .into()
    }

    pub fn get_parameters(&self) -> OsString {
        vec![
            self.get_iwad_parameters(),
            self.get_mods_parameters(),
            [OsStr::new("-savedir"), &self.get_absolute_savedir().unwrap().into_os_string()].join(OsStr::new(" ")),
            self.additional_params.join(OsStr::new(" ")),
        ]
        .join(OsStr::new(" "))
    }

    pub fn get_savedir(&self) -> &Path {
        &self.savedir
    }

    pub fn get_absolute_savedir(&self) -> Option<PathBuf> {
        let path= self.get_savedir();

        if path.is_absolute() {
            return Some(path.to_path_buf());
        }

        if let Ok(base_dir) = SulphurConfig::get_dir().place_data_file("") {
            return Some(base_dir.join(path));
        }

        None
    }

    pub fn set_savedir(&mut self, new: PathBuf) {
        self.savedir = new;
    }
}
