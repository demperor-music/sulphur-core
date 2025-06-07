use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    pub path: PathBuf,
    pub enabled: bool,
}

impl Asset {
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

macro_rules! impl_as_mut_asset {
    ($($t:ty),*) => {
        $(
            impl AsMut<Asset> for $t {
                fn as_mut(&mut self) -> &mut Asset {
                    &mut self.0
                }
            }
        )*
    };
}
macro_rules! impl_as_ref_asset {
    ($($t:ty),*) => {
        $(
            impl AsRef<Asset> for $t {
                fn as_ref(&self) -> &Asset {
                    &self.0
                }
            }
        )*
    };
}
macro_rules! create_asset_variant {
    ($($t:ident),*) => {
        $(
            #[derive(Serialize, Deserialize, Clone)]
            pub struct $t(pub Asset);
        )*
    };
}

create_asset_variant!(Mod, Iwad);
impl_as_mut_asset!(Mod, Iwad);
impl_as_ref_asset!(Mod, Iwad);

impl crate::Movable for Mod {
    fn get_dir_name() -> &'static str {
        "mods"
    }
}

impl crate::Movable for Iwad {
    fn get_dir_name() -> &'static str {
        "iwads"
    }
}

impl crate::Argument for Mod {
    fn get_prefix() -> &'static OsStr {
        OsStr::new("-file")
    }
}

impl crate::Argument for Iwad {
    fn get_prefix() -> &'static OsStr {
        OsStr::new("-iwad")
    }
}