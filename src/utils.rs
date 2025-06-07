use std::ffi::OsString;

use crate::traits::{Argument, Movable};
use crate::asset::Asset;

pub fn get_argument<T: Argument + Movable + AsRef<Asset>>(element: &T) -> OsString {
    let mut argument = OsString::new();
    if let Some(p) = element.get_absolute_path() {
        argument.push(T::get_prefix());
        argument.push(" \"");
        argument.push(p.as_os_str());
        argument.push("\"");
    }
    argument
}

pub fn get_arguments<T>(assets: &[T]) -> impl Iterator<Item = OsString>
where
    T: Argument + Movable + AsRef<Asset>,
{
    get_enabled(assets).map(|asset| get_argument(asset))
}

pub fn get_enabled<T>(assets: &[T]) -> impl Iterator<Item = &T>
where
    T: Argument + Movable + AsRef<Asset>,
{
    assets.iter().filter(|asset| asset.as_ref().enabled)
}
