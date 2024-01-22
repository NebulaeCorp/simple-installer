use anyhow::Error;
use argh::FromArgs;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use ini::Ini;
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
    let mut ini_file = Ini::load_from_file(ini_path)?;
    ini_file.set_to(Some("Debug"), "ConsoleEnabled".to_string(), "1".to_string());
    ini_file.write_to_file(ini_path)?;

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

fn main() -> Result<(), Error> {
    let installer: Installer = argh::from_env();
    let game_path = match installer.path {
        Some(d) => {
            if Path::new(&d).exists() {
                d
            } else {
                eprintln!("Directory doesn't exist!");
                exit(1)
            }
        }
        None => {
            let pw = locate_game();
            if pw.is_none() {
                eprintln!("Game not found!");
                exit(1);
            }
            pw.unwrap().path
        }
    };

    let bin_path = game_path.join("Pal/Binaries/Win64");
    fs::create_dir_all(&bin_path)?;

    match download_ue4ss(&bin_path) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to download UE4SS: {err}");
            exit(1)
        }
    }

    if installer.enable_console {
        let ini_path = bin_path.join("UE4SS-settings.ini");
        enable_console(&ini_path)?;
    }

    write_lua(&bin_path)?;

    Ok(())
}
