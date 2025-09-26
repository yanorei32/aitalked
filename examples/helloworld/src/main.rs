use std::ffi::{c_char, c_void, CStr, CString};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use aitalked::{api as aitalked_api, binding::*, model::*};
use anyhow::Result;
use clap::Parser;
use directories::UserDirs;
use encoding_rs::*;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // #[arg(env, default_value = "C:\\Program Files (x86)\\Gynoid\\GynoidTalk")]
    #[arg(long, env, default_value = "C:\\Program Files (x86)\\AHS\\VOICEROID2")]
    installation_dir: PathBuf,
    #[arg(long, env, default_value = "aitalked.dll")]
    aitalked_dll: PathBuf,
    #[arg(long, env, default_value = "Voice")]
    voice_dir: PathBuf,
    #[arg(long, env, short)]
    character: String,
    #[arg(long, env, default_value = "aitalk.lic")]
    aitalk_lic: PathBuf,
    // #[arg(env, default_value = "Afzu154YOD9urEoHBsCF")]
    #[arg(long, env, default_value = "ORXJC6AIWAUKDpDbH2al")]
    code_auth_seed: String,
    #[arg(short, long, env, default_value = "こんにちは、世界")]
    text: String,
}

fn path_to_cstring(path: &Path) -> CString {
    CString::new(SHIFT_JIS.encode(path.to_str().unwrap()).0).unwrap()
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
        let code = unsafe { aitalked_api::get_data(job_id, &mut buffer, &mut samples_read) };

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

        let code =
            unsafe { aitalked_api::get_kana(job_id, &mut buffer, &mut bytes_read, &mut position) };

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
    /*\
    |*| Load DLL
    \*/
    {
        let original_working_dir = std::env::current_dir()?;
        std::env::set_current_dir(&args.installation_dir)?;
        unsafe { aitalked::load_dll(&args.installation_dir.join(&args.aitalked_dll)) }.unwrap();
        std::env::set_current_dir(&original_working_dir)?;
    }

    let code = unsafe {
        aitalked_api::init(&AitalkedConfig {
            hz_voice_db: 44100,
            dir_voice_dbs: path_to_cstring(&args.installation_dir.join(&args.voice_dir)).as_ptr(),
            msec_timeout: 1000,
            path_license: path_to_cstring(&args.installation_dir.join(&args.aitalk_lic)).as_ptr(),
            code_auth_seed: CString::new(args.code_auth_seed).unwrap().as_ptr(),
            len_auth_seed: 0,
        })
    };

    println!("aitalked_api::init code: {:?}", code);

    {
        let original_working_dir = std::env::current_dir()?;
        std::env::set_current_dir(&args.installation_dir)?;

        let code = unsafe { aitalked_api::lang_load(&CString::new("Lang\\standard").unwrap()) };
        println!("aitalked_api::lang_load code: {:?}", code);

        let user_dir = UserDirs::new().unwrap();
        let document = user_dir.document_dir().unwrap();

        let code = unsafe {
            aitalked_api::reload_word_dic(Some(&path_to_cstring(
                &document.join("VOICEROID2\\単語辞書\\user.wdic"),
            )))
        };
        println!("aitalked_api::reload_word_dic code: {:?}", code);

        let code = unsafe {
            aitalked_api::reload_phrase_dic(Some(&path_to_cstring(
                &document.join("VOICEROID2\\フレーズ辞書\\user.pdic"),
            )))
        };
        println!("aitalked_api::reload_phrase_dic code: {:?}", code);

        let code = unsafe {
            aitalked_api::reload_symbol_dic(Some(&path_to_cstring(
                &document.join("VOICEROID2\\記号ポーズ辞書\\user.sdic"),
            )))
        };
        println!("aitalked_api::reload_symbol_dic code: {:?}", code);

        std::env::set_current_dir(&original_working_dir)?;
    }

    // let code = unsafe { aitalked_api::lang_clear() };
    // println!("aitalked_api::lang_clear code: {:?}", code);
    // let code = unsafe { aitalked_api::voice_clear() };
    // println!("aitalked_api::voice_clear code: {:?}", code);

    let code = unsafe {
        aitalked_api::voice_load(&CString::new(SHIFT_JIS.encode(&args.character).0).unwrap())
    };
    println!("aitalked_api::voice_load code: {:?}", code);

    /*\
    |*| Param Initialization
    \*/
    let empty_tts_param_size = std::mem::size_of::<TtsParam>() as u32;
    println!("Empty TtsParamSize: {empty_tts_param_size}");

    let speaker_param_size = std::mem::size_of::<SpeakerParam>() as u32;
    println!("SpeakerParamSize: {speaker_param_size}");

    let mut actual_tts_param_size = 0;

    let code = unsafe { aitalked_api::get_param(std::ptr::null_mut(), &mut actual_tts_param_size) };
    println!(
        "aitalked_api::get_param: {:?} (expects: INSUFFICIENT)",
        code
    );

    println!("Actual TtsParamSize: {actual_tts_param_size}");

    let estimate_speaker_param_count =
        (actual_tts_param_size - empty_tts_param_size) / speaker_param_size;

    println!("Estimate Speaker Param Count: {estimate_speaker_param_count}");

    let mut boxed_tts_param = BoxedTtsParam::new(estimate_speaker_param_count as usize);
    let code = unsafe {
        aitalked_api::get_param(boxed_tts_param.tts_param_mut(), &mut actual_tts_param_size)
    };
    println!("aitalked_api::get_param: {code:?}");

    /*\
    |*| Set Params
    \*/
    boxed_tts_param.tts_param_mut().pause_begin = 0;
    boxed_tts_param.tts_param_mut().pause_term = 0;
    boxed_tts_param.tts_param_mut().extend_format =
        ExtendFormat::JEITA_RUBY | ExtendFormat::AUTO_BOOKMARK;

    let code = unsafe { aitalked_api::set_param(boxed_tts_param.tts_param()) };
    println!("aitalked_api::set_param: {code:?}");

    println!("tts_param: {:#?}", boxed_tts_param.tts_param());
    println!("speakers: {:#?}", boxed_tts_param.speakers());

    /*\
    |*| Start Text2Kana
    \*/
    boxed_tts_param.tts_param_mut().proc_text_buf = Some(text_buffer_callback);
    let code = unsafe { aitalked_api::set_param(boxed_tts_param.tts_param()) };
    println!("aitalked_api::set_param: {code:?}");

    let mut job_id = 0;

    let mut buffer = vec![];
    let (tx, mut rx) = mpsc::channel(1);

    let mut context = ProcTextBufContext {
        buffer: &mut buffer,
        notify: tx.clone(),
        len_text_buf_bytes: boxed_tts_param.tts_param().len_text_buf_bytes,
    };

    let code = unsafe {
        aitalked_api::text_to_kana(
            &mut job_id,
            &mut context as *mut ProcTextBufContext as *mut std::ffi::c_void,
            &CString::new(SHIFT_JIS.encode(&args.text).0).unwrap(),
        )
    };
    println!("aitalked_api::text_to_kana: {code:?}");

    // await EOF received
    rx.recv().await.unwrap();

    drop(context);

    println!("Kana: {}", SHIFT_JIS.decode(&buffer).0);
    let code = unsafe { aitalked_api::close_kana(job_id, 0) };
    println!("aitalked_api::close_kana: {code:?}");

    // Unload proc_text_buf
    boxed_tts_param.tts_param_mut().proc_text_buf = None;
    let code = unsafe { aitalked_api::set_param(boxed_tts_param.tts_param()) };
    println!("aitalked_api::set_param: {code:?}");

    // Add '\0'
    buffer.push(0);

    let kana = CStr::from_bytes_with_nul(&buffer).unwrap();

    /*\
    |*| Start Kana2Speech
    \*/
    boxed_tts_param.tts_param_mut().proc_raw_buf = Some(raw_buf_callback);
    boxed_tts_param.tts_param_mut().proc_event_tts = Some(tts_event_callback);
    let code = unsafe { aitalked_api::set_param(boxed_tts_param.tts_param()) };
    println!("aitalked_api::set_param: {code:?}");

    let mut job_id = 0;
    let (tx, mut rx) = mpsc::channel(1);

    let mut buffer = vec![];
    let mut events = vec![];

    let mut context = TextToSpeechContext {
        events: &mut events,
        buffer: &mut buffer,
        notify: tx.clone(),
        len_raw_buf_words: boxed_tts_param.tts_param().len_raw_buf_words,
    };

    let code = unsafe {
        aitalked_api::text_to_speech(
            &mut job_id,
            &mut context as *mut TextToSpeechContext as *mut std::ffi::c_void,
            &kana,
        )
    };
    println!("aitalked_api::text_to_speech: {code:?}");

    // await EOF received
    rx.recv().await.unwrap();

    drop(context);

    println!("AudioBufferLength: {}", buffer.len());
    println!("Events:");
    for event in events {
        println!(" - {event:?}");
    }

    // Unload
    boxed_tts_param.tts_param_mut().proc_raw_buf = None;
    boxed_tts_param.tts_param_mut().proc_event_tts = None;
    let code = unsafe { aitalked_api::set_param(boxed_tts_param.tts_param()) };
    println!("aitalked_api::set_param: {code:?}");

    let code = unsafe { aitalked_api::close_speech(job_id, 0) };
    println!("aitalked_api::close_speech: {code:?}");

    /*\
    |*| Write to WAVE file
    \*/
    const WAV_HEADER_SIZE: usize = 44;
    let mut file = BufWriter::new(std::fs::File::create("output.wav").unwrap());
    file.write_all(b"RIFF")?;
    file.write_all(&((buffer.len() + WAV_HEADER_SIZE) as u32).to_le_bytes())?;
    file.write_all(b"WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00")?;
    file.write_all(&44100u32.to_le_bytes())?;
    file.write_all(&(44100u32 * 2).to_le_bytes())?;
    file.write_all(b"\x02\x00\x10\x00data")?;
    file.write_all(&buffer)?;

    println!("Output file created");

    Ok(())
}
