use std::path::Path;

use anyhow::{Ok, Result};
use chrono::prelude::*;
use clap::Parser;
use geo::Coords;
use loader::WallpaperLoader;
use properties::WallpaperProperties;

#[macro_use]
mod macros;
mod cache;
mod cli;
mod config;
mod constants;
mod geo;
mod heic;
mod loader;
mod metadata;
mod properties;
mod selection;
mod setter;
mod wallpaper;

use metadata::ImageInfo;

use crate::cache::LastWallpaper;
use crate::config::Config;
use crate::selection::select_image_h24;
use crate::selection::select_image_solar;
use crate::setter::set_wallpaper;

fn main() -> Result<()> {
    env_logger::init();

    let args = cli::Args::parse();

    match args.action {
        cli::Action::Info { file } => {
            println!("{}", ImageInfo::from_image(file)?);
            Ok(())
        }
        cli::Action::Unpack { file, output } => wallpaper::unpack_heic(file, output),
        cli::Action::Set { file } => set(file),
    }
}

pub fn set<P: AsRef<Path>>(path: P) -> Result<()> {
    let config = Config::find()?;
    println!("{config:?}");
    let mut loader = WallpaperLoader::new();
    let last_wallpaper = LastWallpaper::find();
    println!("{loader:?}");
    let wallpaper = loader.load(&path);
    last_wallpaper.save(&path);
    println!("{wallpaper:?}");

    let now = Local::now();
    let current_index = match wallpaper.properties {
        WallpaperProperties::H24(props) => select_image_h24(&props.time_info, &now.time()),
        WallpaperProperties::Solar(props) => {
            select_image_solar(&props.solar_info, &now, &config.coords)
        }
    }?;
    let current_image_path = wallpaper.images.get(current_index).unwrap();

    println!("image index: {}", current_index);

    set_wallpaper(current_image_path)?;

    Ok(())
}
