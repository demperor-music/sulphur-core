use std::fs;
use std::io::Error;
use std::path::PathBuf;
use crate::{SaveableDefaultPath, SulphurConfig};

pub struct Savedir;

impl Savedir {
    pub fn get_dir_name() -> &'static str {
        "saves"
    }
    pub fn get() -> Result<PathBuf, Error> {
        SulphurConfig::get_dir()
            .place_data_file(Savedir::get_dir_name())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
