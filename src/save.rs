use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use chrono::offset::Local;
use crate::{Stock, Player};
use directories::ProjectDirs;
use serde::{Serialize, Deserialize};
use serde_json::error;

#[derive(Debug)]
pub enum Error {
    NotFound(PathBuf),
    PlatformNotSupported,
    IoError(io::Error),
    SerdeJsonError(error::Error),
    AlreadyExists,
    EmptyFileName,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<error::Error> for Error {
    fn from(error: error::Error) -> Self {
        Error::SerdeJsonError(error)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub stocks: Vec<Stock>,
    pub player: Player,
    pub goal: i64,
    pub add_stock_cost: i64,
    pub initial_income: i64,
    pub income_upgrade_cost: i64,
}

#[derive(Hash)]
pub struct Save {
    pub path: PathBuf,
    pub name: String,
}

impl fmt::Display for Save {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Turns a `&Path` into a `Game`. Will return an error if there was an issue reading
/// the file at the Path or if there's an issue parsing the JSON.
pub fn from_path(path: &Path) -> Result<Game, Error> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn project_save_dir() -> Result<PathBuf, Error> {
    let pd = ProjectDirs::from("xyz", "Rainbow Asteroids", "Millionaire");
    let pd = match pd {
        Some(pd) => pd,
        None => return Err(Error::PlatformNotSupported),
    };

    Ok(pd.data_dir().to_path_buf())
}

/// Finds all the potential save files and returns them. Will error if there was some
/// issue reading the directory.
pub fn saves_in_folder(dir: Option<&Path>) -> Result<Vec<Save>, Error> {
    let mut result = Vec::new();
        
    let dir = match dir {
        Some(p) => p.to_path_buf(),
        None => project_save_dir()?,
    };

    if !dir.is_dir() {
        return Err(Error::NotFound(dir));
    }

    for f in dir.read_dir()? {
        let f = match f {
            Ok(de) => de,
            Err(_) => continue,
        };

        if f.file_name().to_string_lossy().ends_with(".save.json") {
            let mut name = f.file_name().to_string_lossy().into_owned();
            name.replace_range(name.len()-10.., ""); // Remove the extension

            result.push(Save {
                path: f.path(),
                name
            });
        }
    }

    Ok(result)
}

/// Get a path to a save file.
pub fn make_path(dir: Option<&Path>) -> Result<PathBuf, Error> {
    let mut dir = match dir {
        Some(p) => p.to_path_buf(),
        None => project_save_dir()?,
    };

    dir.push(Local::now().format("%Y-%m-%d %H:%M:%S.save.json").to_string());
    Ok(dir)
}

/// Saves a game at path
pub fn save(path: &Path, game: &Game) -> Result<(), Error> {
    fs::write(path, serde_json::to_string(game)?)?;
    
    Ok(())
}

/// Copies a save in the same folder as the specified save.
pub fn copy(path: &Path) -> Result<(), Error> {
    let copy_name = format!("{} {}", "Copy of", path.file_name().unwrap().to_string_lossy());
    let mut copy_path = path.to_path_buf();
    copy_path.set_file_name(copy_name);

    fs::copy(path, &copy_path)?;

    Ok(())
}

/// Deletes a save. Pretty much the same as `std::fs::remove_file`.
pub fn delete(path: &Path) -> Result<(), Error> {
    fs::remove_file(path)?;
    Ok(())
}

/// Renames save file.
pub fn rename(path: &Path, name: &str) -> Result<(), Error> {
    let name = name.trim();
    if name == "" { return Err(Error::EmptyFileName); }

    let mut new_path = path.to_path_buf();
    new_path.set_file_name(format!("{}.save.json", name));
    if new_path.exists() { return Err(Error::AlreadyExists); }
    fs::rename(path, &new_path)?;

    Ok(())
}
