use std::ffi::{c_char, c_void};

use derivative::Derivative;
use encoding_rs::*;

fn format_sjis_cchar_slice(
    s: &[c_char],
    fmt: &mut std::fmt::Formatter,
) -> Result<(), std::fmt::Error> {
    use std::fmt::Debug;
    let s = unsafe { std::slice::from_raw_parts(s.as_ptr() as *const u8, s.len()) };
    let (s, _encoding, _errors) = SHIFT_JIS.decode(s);
    let s = s.trim_matches(char::from(0));
    s.fmt(fmt)
}

pub const LEN_TEXT_BUF_MAX: u32 = 64 * 1024;
pub const LEN_RAW_BUF_MAX_BYTES: u32 = 1024 * 1024;
pub const JEITA_RUBY: i32 = 1;
pub const AUTO_BOOKMARK: i32 = 16;

#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum EventReasonCode {
    TEXTBUF_FULL = 101,
    TEXTBUF_FLUSH = 102,
    TEXTBUF_CLOSE = 103,
    RAWBUF_FULL = 201,
    RAWBUF_FLUSH = 202,
    RAWBUF_CLOSE = 203,
    PH_LABEL = 301,
    BOOKMARK = 302,
    AUTO_BOOKMARK = 303,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum ResultCode {
    SUCCESS = 0,
    INTERNAL_ERROR = -1,
    UNSUPPORTED = -2,
    INVALID_ARGUMENT = -3,
    WAIT_TIMEOUT = -4,
    NOT_INITIALIZED = -10,
    ALREADY_INITIALIZED = 10,
    NOT_LOADED = -11,
    ALREADY_LOADED = 11,
    INSUFFICIENT = -20,
    PARTIALLY_REGISTERED = 21,
    LICENSE_ABSENT = -100,
    LICENSE_EXPIRED = -101,
    LICENSE_REJECTED = -102,
    TOO_MANY_JOBS = -201,
    INVALID_JOBID = -202,
    JOB_BUSY = -203,
    NOMORE_DATA = 204,
    OUT_OF_MEMORY = -206,
    FILE_NOT_FOUND = -1001,
    PATH_NOT_FOUND = -1002,
    READ_FAULT = -1003,
    COUNT_LIMIT = -1004,
    USERDIC_LOCKED = -1011,
    USERDIC_NOENTRY = -1012,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum JobInOut {
    PLAIN_TO_WAVE = 11,
    AIKANA_TO_WAVE = 12,
    JEITA_TO_WAVE = 13,
    PLAIN_TO_AIKANA = 21,
    AIKANA_TO_JEITA = 32,
}

pub const MAX_VOICE_NAME: usize = 80;
pub const MAX_JEITA_CONTROL: usize = 12;

#[repr(C)]
#[derive(Debug)]
pub struct JobParam {
    pub model_in_out: JobInOut,
    pub user_data: *mut c_void,
}

#[repr(C)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct JeitaParam {
    #[derivative(Debug(format_with = "format_sjis_cchar_slice"))]
    pub female_name: [c_char; MAX_VOICE_NAME],
    #[derivative(Debug(format_with = "format_sjis_cchar_slice"))]
    pub male_name: [c_char; MAX_VOICE_NAME],
    pub pause_middle: i32,
    pub pause_long: i32,
    pub pause_sentence: i32,
    #[derivative(Debug(format_with = "format_sjis_cchar_slice"))]
    pub control: [c_char; MAX_JEITA_CONTROL],
}

impl Default for JeitaParam {
    fn default() -> Self {
        Self {
            female_name: [0 as c_char; MAX_VOICE_NAME],
            male_name: [0 as c_char; MAX_VOICE_NAME],
            pause_middle: 0,
            pause_long: 0,
            pause_sentence: 0,
            control: [0 as c_char; MAX_JEITA_CONTROL],
        }
    }
}

#[repr(C)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct SpeakerParam {
    #[derivative(Debug(format_with = "format_sjis_cchar_slice"))]
    pub voice_name: [c_char; MAX_VOICE_NAME],
    pub volume: f32,
    pub speed: f32,
    pub pitch: f32,
    pub range: f32,
    pub pause_middle: i32,
    pub pause_long: i32,
    pub pause_sentence: i32,
    #[derivative(Debug(format_with = "format_sjis_cchar_slice"))]
    pub style_rate: [c_char; MAX_VOICE_NAME],
}

impl Default for SpeakerParam {
    fn default() -> Self {
        Self {
            voice_name: [0 as c_char; MAX_VOICE_NAME],
            volume: 0.0,
            speed: 0.0,
            pitch: 0.0,
            range: 0.0,
            pause_middle: 0,
            pause_long: 0,
            pause_sentence: 0,
            style_rate: [0 as c_char; MAX_VOICE_NAME],
        }
    }
}

#[repr(C)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct TtsParam {
    pub size: u32,
    pub proc_text_buf: Option<unsafe extern "system" fn(EventReasonCode, i32, *mut c_void) -> i32>,
    pub proc_raw_buf:
        Option<unsafe extern "system" fn(EventReasonCode, i32, u64, *mut c_void) -> i32>,
    pub proc_event_tts: Option<
        unsafe extern "system" fn(EventReasonCode, i32, u64, *const c_char, *mut c_void) -> i32,
    >,
    pub len_text_buf_bytes: u32,
    pub len_raw_buf_words: u32,
    pub volume: f32,
    pub pause_begin: i32,
    pub pause_term: i32,
    pub extend_format: i32,
    #[derivative(Debug(format_with = "format_sjis_cchar_slice"))]
    pub voice_name: [c_char; MAX_VOICE_NAME],
    pub jeita: JeitaParam,
    pub num_speakers: u32,
    pub _reserved: i32,
    pub speakers: [SpeakerParam; 0],
}

#[repr(C)]
#[derive(Debug)]
pub struct AitalkedConfig {
    pub hz_voice_db: u32,
    pub dir_voice_dbs: *const c_char,
    pub msec_timeout: u32,
    pub path_license: *const c_char,
    pub code_auth_seed: *const c_char,
    pub len_auth_seed: u32,
}

