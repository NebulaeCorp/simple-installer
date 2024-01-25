mod error;

use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use crate::error::Error;
use argh::FromArgs;
use steamlocate::{SteamApp, SteamDir};
use zip::ZipArchive;

const UE4SS_URL: &str =
    "https://github.com/UE4SS-RE/RE-UE4SS/releases/download/v2.5.2/UE4SS_Xinput_v2.5.2.zip";
const LUA_CODE: &str = "function Register()
  return \"48 89 5C 24 08 57 48 83 EC 30 48 8B D9 48 89 54 24 20 33 C9\"
end

function OnMatchFound(MatchAddress)
  return MatchAddress
end";

fn locate_game() -> Option<SteamApp> {
    let steamdir = SteamDir::locate();
    steamdir.and_then(|mut dir| dir.app(&1623730).map(|a| a.to_owned()))
}

fn download_ue4ss(game_path: &PathBuf) -> Result<(), Error> {
    let client = reqwest::blocking::Client::new();
    let resp = client.get(UE4SS_URL).send()?.bytes()?;

    let content = Cursor::new(resp.to_vec());
    ZipArchive::new(content)?.extract(game_path)?;

    Ok(())
}

fn enable_console(ini_path: &PathBuf) -> Result<(), Error> {
    let mut file_content = String::new();
    let mut file = File::open(ini_path)?;
    file.read_to_string(&mut file_content)?;

    file_content = file_content.replace("ConsoleEnabled = 0", "ConsoleEnabled = 1");

    let mut modified_file = fs::File::create(ini_path)?;
    modified_file.write_all(file_content.as_bytes())?;

    Ok(())
}

fn enable_logicmods(cfg_path: &PathBuf) -> Result<(), Error> {
    let mut file_content = String::new();
    let mut file = File::open(cfg_path)?;
    file.read_to_string(&mut file_content)?;

    file_content = file_content.replace("BPModLoaderMod : 0", "BPModLoaderMod : 1");

    let mut modified_file = File::create(cfg_path)?;
    modified_file.write_all(file_content.as_bytes())?;

    Ok(())
}

fn write_lua(bin_path: &Path) -> Result<(), Error> {
    let sigs_path = &bin_path.join("UE4SS_Signatures");
    fs::create_dir_all(sigs_path)?;

    let lua_path = &sigs_path.join("FName_Constructor.lua");
    let mut lua = OpenOptions::new()
        .create(true)
        .append(false)
        .write(true)
        .open(lua_path)?;

    lua.write_all(LUA_CODE.as_bytes())?;

    Ok(())
}

#[derive(FromArgs, Debug)]
/// Install necessary components to start using mods.
struct Installer {
    /// game binary path
    #[argh(option, short = 'd')]
    path: Option<PathBuf>,
    /// enable debugging console
    #[argh(option, short = 'c', default = "true")]
    enable_console: bool,
}

fn run(installer: Installer) -> Result<(), Error> {
    let game_path = match installer.path {
        Some(d) => {
            if Path::new(&d).exists() {
                d
            } else {
                return Err(Error::MissingDirectory);
            }
        }
        None => {
            let pw = locate_game();
            if pw.is_none() {
                return Err(Error::GameNotFound);
            }
            pw.unwrap().path
        }
    };

    let bin_path = game_path.join("Pal/Binaries/Win64");
    let content_path = game_path.join("Pal/Content/Paks/LogicMods");
    fs::create_dir_all(&bin_path)?;
    fs::create_dir_all(&content_path)?;

    download_ue4ss(&bin_path)?;

    enable_logicmods(&bin_path.join("Mods/mods.txt"))?;

    if installer.enable_console {
        let ini_path = bin_path.join("UE4SS-settings.ini");
        enable_console(&ini_path)?;
    }

    write_lua(&bin_path)?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let installer: Installer = argh::from_env();
    match run(installer) {
        Ok(_) => println!("Installed successfully!"),
        Err(error) => eprintln!("{error}"),
    }

    press_btn_continue::wait("Press any key to continue...")?;

    Ok(())
}
