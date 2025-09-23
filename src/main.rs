mod aitalked;
use std::ffi::{c_void, CString};

use std::path::{Path, PathBuf};

use aitalked::*;
use anyhow::Result;
use clap::Parser;
use libloading::Library;
use tokio::sync::mpsc;

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

struct ProcTextBufContext<'a> {
    buffer: &'a mut Vec<u8>,
    notify: mpsc::Sender<()>,
    aitalked: &'a Aitalked<'a>,
    len_text_buf_bytes: u32,
}

extern "system" fn proc_text_buf(
    reason_code: EventReasonCode,
    job_id: i32,
    user_data: *mut c_void,
) -> i32 {
    match reason_code {
        EventReasonCode::TEXTBUF_FULL
        | EventReasonCode::TEXTBUF_FLUSH
        | EventReasonCode::TEXTBUF_CLOSE => (),
        _ => return 0,
    }

    let context = unsafe { &mut *( user_data as *mut ProcTextBufContext<'static> ) };
    let buffer_length = context.len_text_buf_bytes.min(LEN_TEXT_BUF_MAX);

    let mut buffer = vec![0; buffer_length as usize];

    loop {
        let mut bytes_read = 0;
        let mut position = 0;

        context.aitalked.get_kana(job_id, &mut buffer, &mut bytes_read, &mut position);
        context.buffer.extend_from_slice(&buffer[0..bytes_read as usize]);

        if bytes_read < buffer_length - 1 {
            break;
        }
    }

    if reason_code == EventReasonCode::TEXTBUF_CLOSE {
        context.notify.blocking_send(()).unwrap();
    }

    0
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
    let code = aitalked.load_language(&CString::new("Lang\\standard").unwrap());
    println!("code: {:?}", code);
    // let code = aitalked.load_voice("hime_44");
    let code = aitalked.load_voice(&CString::new("akari_44").unwrap());
    println!("code: {:?}", code);

    let empty_tts_param_size = std::mem::size_of::<TtsParam>() as u32;
    println!("Empty TtsParamSize: {empty_tts_param_size}");

    let speaker_param_size = std::mem::size_of::<SpeakerParam>() as u32;
    println!("SpeakerParamSize: {speaker_param_size}");

    let mut actual_tts_param_size = 0;
    let code = aitalked.get_param(std::ptr::null_mut(), &mut actual_tts_param_size);

    println!("code: {:?}", code);
    println!("Actual TtsParamSize: {actual_tts_param_size}");

    let estimate_speaker_param_count =
        (actual_tts_param_size - empty_tts_param_size) / speaker_param_size;

    println!("Estimate Speaker Param Count: {estimate_speaker_param_count}");

    let mut boxed_tts_param = BoxedTtsParam::new(estimate_speaker_param_count as usize);
    let code = aitalked.get_param(boxed_tts_param.tts_param_mut(), &mut actual_tts_param_size);

    println!("Get code: {code:?}");
    println!("tts_param: {:#?}", boxed_tts_param.tts_param());
    println!("speakers: {:#?}", boxed_tts_param.speakers());

    boxed_tts_param.tts_param_mut().proc_text_buf = Some(proc_text_buf);
    let code = aitalked.set_param(boxed_tts_param.tts_param());
    println!("Set code: {code:?}");

    let mut job_id = 0;

    let mut buffer = vec![];
    let (tx, mut rx) = mpsc::channel(1);

    let mut context = ProcTextBufContext {
        buffer: &mut buffer,
        notify: tx.clone(),
        aitalked: &aitalked,
        len_text_buf_bytes: boxed_tts_param.tts_param().len_text_buf_bytes,
    };

    aitalked.text_to_kana(
        &mut job_id,
        &mut context as *mut ProcTextBufContext as *mut std::ffi::c_void,
        &CString::new("Hello World").unwrap(),
    );

    rx.recv().await.unwrap();

    println!("Received");

    drop(context);

    println!("{buffer:x?}{}", buffer.len());

    // boxed_tts_param.speakers_mut()[0].volume = 0.3;
    // let code = aitalked.set_param(boxed_tts_param.tts_param());
    // println!("Set code: {code:?}");
    //
    // let code = aitalked.get_param(boxed_tts_param.tts_param_mut(), &mut actual_tts_param_size);
    // println!("Get code: {code:?}");
    // println!("tts_param: {:#?}", boxed_tts_param.tts_param());
    // println!("speakers: {:#?}", boxed_tts_param.speakers());

    Ok(())
}
