mod aitalked;

use std::{
    ffi::CString,
    path::{Path, PathBuf},
};

use aitalked::ConfigFactory;
use anyhow::Result;
use clap::Parser;
use libloading::Library;

use crate::aitalked::Aitalked;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // #[arg(env, default_value = "C:\\Program Files (x86)\\Gynoid\\GynoidTalk")]
    #[arg(env, default_value = "C:\\Program Files (x86)\\AHS\\VOICEROID2")]
    installation_dir: PathBuf,
    #[arg(env, default_value = "aitalked.dll")]
    aitalked_dll: PathBuf,
    #[arg(env, default_value = "Voice")]
    voice_dir: PathBuf,
    #[arg(env, default_value = "aitalk.lic")]
    aitalk_lic: PathBuf,
    // #[arg(env, default_value = "Afzu154YOD9urEoHBsCF")]
    #[arg(env, default_value = "ORXJC6AIWAUKDpDbH2al")]
    code_auth_seed: String,
}

fn path_to_cstring(path: &Path) -> Result<CString> {
    Ok(CString::new(path.to_str().unwrap())?)
}

impl Args {
    fn config(&self) -> Result<ConfigFactory> {
        let dir_voice_dbs = path_to_cstring(&self.installation_dir.join(&self.voice_dir))?;
        let path_license = path_to_cstring(&self.installation_dir.join(&self.aitalk_lic))?;
        let code_auth_seed = CString::new(self.code_auth_seed.as_str())?;
        Ok(ConfigFactory {
            dir_voice_dbs,
            path_license,
            code_auth_seed,
        })
    }

    unsafe fn load_aitalked_dll(&self) -> Result<Library> {
        Ok(Library::new(
            self.installation_dir.join(&self.aitalked_dll),
        )?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config_factory = args.config()?;
    let config = config_factory.build();

    std::env::set_current_dir(&args.installation_dir)?;
    let dll = unsafe { args.load_aitalked_dll()? };
    let aitalked = unsafe { Aitalked::new(&dll)? };
    let code = aitalked.init(&config);
    println!("code: {:?}", code);
    let code = aitalked.load_language("Lang\\standard");
    println!("code: {:?}", code);
    // let code = aitalked.load_voice("hime_44");
    let code = aitalked.load_voice("akari_44");
    println!("code: {:?}", code);



    Ok(())
}
