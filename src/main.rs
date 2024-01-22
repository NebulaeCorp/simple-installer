use std::fs;
use std::fs::OpenOptions;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::process::exit;

use ini::Ini;
use steamlocate::{SteamApp, SteamDir};
use zip::ZipArchive;

const UE4SS: &str =
    "https://github.com/UE4SS-RE/RE-UE4SS/releases/download/v2.5.2/UE4SS_Xinput_v2.5.2.zip";
const LUA_CODE: &str = "function Register()
  return \"48 89 5C 24 08 57 48 83 EC 30 48 8B D9 48 89 54 24 20 33 C9\"
end

function OnMatchFound(MatchAddress)
  return MatchAddress
end";

type Error = anyhow::Error;

fn locate_game() -> Option<SteamApp> {
    let steamdir = SteamDir::locate();
    steamdir
        .map(|mut dir| dir.app(&1623730).map(|a| a.to_owned()))
        .flatten()
}

fn download_ue4ss(game_path: &PathBuf) -> Result<(), Error> {
    let client = reqwest::blocking::Client::new();
    let resp = client.get(UE4SS).send()?.bytes()?;

    let content = Cursor::new(resp.to_vec());
    ZipArchive::new(content)?.extract(game_path)?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let pw = locate_game();
    if pw.is_none() {
        eprintln!("Game not found!");
        exit(1);
    }
    let pw = pw.unwrap();
    let path = pw.path.join("Pal/Binaries/Win64");
    dbg!(&path);
    match download_ue4ss(&path) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{err}");
            exit(1)
        }
    }
    let ini_path = path.join("UE4SS-settings.ini");
    let mut ini_file = Ini::load_from_file(&ini_path)?;
    ini_file.set_to(Some("Debug"), "ConsoleEnabled".to_string(), "1".to_string());
    dbg!(1);
    ini_file.write_to_file(&ini_path)?;
    dbg!(2);
    let sigs_path = &path.join("UE4SS_Signatures");
    fs::create_dir_all(&sigs_path)?;
    let lua_path = &sigs_path.join("FName_Constructor.lua");
    dbg!(&lua_path);
    let mut lua = OpenOptions::new()
        .create(true)
        .append(false)
        .write(true)
        .open(lua_path)?;
    lua.write_all(&LUA_CODE.as_bytes())?;

    Ok(())
}
