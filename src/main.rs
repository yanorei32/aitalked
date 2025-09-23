mod aitalked;
use std::ffi::{c_char, c_void, CStr, CString};
use std::io::Write;
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

#[derive(Debug, PartialEq, Eq)]
pub enum TtsEvent {
    Phonetic(CString),
    Position(u32),
    Bookmark(CString),
}

struct TextToSpeechContext<'a> {
    events: &'a mut Vec<(u64, TtsEvent)>,
    buffer: &'a mut Vec<u8>,
    notify: mpsc::Sender<()>,
    aitalked: &'a Aitalked<'a>,
    len_raw_buf_words: u32,
}

extern "system" fn tts_event_callback(
    reason_code: EventReasonCode,
    _job_id: i32,
    tick: u64,
    name: *const c_char,
    user_data: *mut c_void,
) -> i32 {
    let context = unsafe { &mut *(user_data as *mut TextToSpeechContext<'static>) };

    let name = unsafe { CStr::from_ptr(name as *const i8) };

    match reason_code {
        EventReasonCode::PH_LABEL => {
            context
                .events
                .push((tick, TtsEvent::Phonetic(name.to_owned())));
        }
        EventReasonCode::AUTO_BOOKMARK => {
            if let Ok(value) = name.to_string_lossy().parse() {
                context.events.push((tick, TtsEvent::Position(value)));
            }
        }
        EventReasonCode::BOOKMARK => {
            context
                .events
                .push((tick, TtsEvent::Bookmark(name.to_owned())));
        }
        _ => {}
    }

    0
}

extern "system" fn raw_buf_callback(
    reason_code: EventReasonCode,
    job_id: i32,
    _tick: u64,
    user_data: *mut c_void,
) -> i32 {
    match reason_code {
        EventReasonCode::RAWBUF_FULL
        | EventReasonCode::RAWBUF_FLUSH
        | EventReasonCode::RAWBUF_CLOSE => (),
        _ => return 0,
    }

    let context = unsafe { &mut *(user_data as *mut TextToSpeechContext<'static>) };
    let buffer_bytes = (context.len_raw_buf_words * 2).min(LEN_RAW_BUF_MAX_BYTES);

    let mut buffer = vec![0; buffer_bytes as usize];

    loop {
        let mut samples_read = 0;
        let code = context
            .aitalked
            .get_data(job_id, &mut buffer, &mut samples_read);

        if code != ResultCode::SUCCESS {
            break;
        }

        context
            .buffer
            .extend_from_slice(&buffer[0..(samples_read * 2) as usize]);

        if samples_read * 2 < buffer_bytes {
            break;
        }
    }

    if reason_code == EventReasonCode::RAWBUF_CLOSE {
        context.notify.blocking_send(()).unwrap();
    }

    0
}

struct ProcTextBufContext<'a> {
    buffer: &'a mut Vec<u8>,
    notify: mpsc::Sender<()>,
    aitalked: &'a Aitalked<'a>,
    len_text_buf_bytes: u32,
}

extern "system" fn text_buffer_callback(
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

    let context = unsafe { &mut *(user_data as *mut ProcTextBufContext<'static>) };
    let buffer_length = context.len_text_buf_bytes.min(LEN_TEXT_BUF_MAX);

    let mut buffer = vec![0; buffer_length as usize];

    loop {
        let mut bytes_read = 0;
        let mut position = 0;

        let code = context
            .aitalked
            .get_kana(job_id, &mut buffer, &mut bytes_read, &mut position);

        if code != ResultCode::SUCCESS {
            break;
        }

        context
            .buffer
            .extend_from_slice(&buffer[0..bytes_read as usize]);

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

    let original_working_dir = std::env::current_dir()?;

    /*\
    |*| Load DLL
    \*/
    std::env::set_current_dir(&args.installation_dir)?;
    let dll = unsafe { args.load_aitalked_dll()? };
    let aitalked = unsafe { Aitalked::new(&dll)? };

    /*\
    |*| Talk Library Initialization
    \*/
    let code = aitalked.init(&config);
    println!("code: {:?}", code);
    let code = aitalked.load_language(&CString::new("Lang\\standard").unwrap());
    println!("code: {:?}", code);
    let code = aitalked.load_voice(&CString::new("akari_44").unwrap());
    println!("code: {:?}", code);

    /*\
    |*| Param Initialization
    \*/
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

    /*\
    |*| Set Params
    \*/
    boxed_tts_param.tts_param_mut().pause_begin = 0;
    boxed_tts_param.tts_param_mut().pause_term = 0;
    boxed_tts_param.tts_param_mut().extend_format = JEITA_RUBY | AUTO_BOOKMARK;


    /*\
    |*| Start Text2Kana
    \*/
    boxed_tts_param.tts_param_mut().proc_text_buf = Some(text_buffer_callback);
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

    // await EOF received
    rx.recv().await.unwrap();

    println!("Received");

    drop(context);

    println!("{buffer:x?} (len: {})", buffer.len());
    let code = aitalked.close_kana(job_id, 0);
    println!("Close code: {code:?}");

    // Unload proc_text_buf
    boxed_tts_param.tts_param_mut().proc_text_buf = None;
    let code = aitalked.set_param(boxed_tts_param.tts_param());
    println!("Set code: {code:?}");

    // Add '\0'
    buffer.push(0);

    let kana = CStr::from_bytes_with_nul(&buffer).unwrap();

    /*\
    |*| Start Kana2Speech
    \*/
    boxed_tts_param.tts_param_mut().proc_raw_buf = Some(raw_buf_callback);
    boxed_tts_param.tts_param_mut().proc_event_tts = Some(tts_event_callback);
    let code = aitalked.set_param(boxed_tts_param.tts_param());
    println!("Set code: {code:?}");

    let mut job_id = 0;
    let (tx, mut rx) = mpsc::channel(1);

    let mut buffer = vec![];
    let mut events = vec![];

    let mut context = TextToSpeechContext {
        events: &mut events,
        buffer: &mut buffer,
        notify: tx.clone(),
        aitalked: &aitalked,
        len_raw_buf_words: boxed_tts_param.tts_param().len_raw_buf_words,
    };

    let code = aitalked.text_to_speech(
        &mut job_id,
        &mut context as *mut TextToSpeechContext as *mut std::ffi::c_void,
        &kana,
    );

    println!("TTS code: {code:?}");

    // await EOF received
    rx.recv().await.unwrap();

    println!("Received");

    drop(context);

    println!("Buffer: {}", buffer.len());
    println!("Events: {:#?}", events);

    // Unload
    boxed_tts_param.tts_param_mut().proc_raw_buf = None;
    boxed_tts_param.tts_param_mut().proc_event_tts = None;
    let code = aitalked.set_param(boxed_tts_param.tts_param());
    println!("Set code: {code:?}");

    let code = aitalked.close_speech(job_id, 0);
    println!("Close code: {code:?}");

    /*\
    |*| Write to file
    \*/
    std::env::set_current_dir(&original_working_dir)?;
    let mut file = std::fs::File::create("output.bin").unwrap();
    file.write_all(&buffer).unwrap();

    println!("Output file created");

    Ok(())
}
