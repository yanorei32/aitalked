use std::{
    ffi::{c_char, c_void, CString},
    marker::PhantomData,
};

use std::alloc::{alloc, dealloc, Layout};

use anyhow::Result;
use libloading::{Library, Symbol};

pub struct ConfigFactory {
    pub dir_voice_dbs: CString,
    pub path_license: CString,
    pub code_auth_seed: CString,
}

impl ConfigFactory {
    pub fn build(&self) -> Config<'_> {
        Config {
            hz_voice_db: 44100,
            dir_voice_dbs: self.dir_voice_dbs.as_ptr(),
            msec_timeout: 1000,
            path_license: self.path_license.as_ptr(),
            code_auth_seed: self.code_auth_seed.as_ptr(),
            len_auth_seed: 0,
            _marker: &PhantomData,
        }
    }
}

#[repr(i32)]
#[derive(Debug)]
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

pub const MAX_VOICE_NAME: usize = 80;
pub const MAX_JEITA_CONTROL: usize = 12;

#[repr(C)]
#[derive(Debug)]
pub struct JeitaParam {
    pub female_name: [c_char; MAX_VOICE_NAME],
    pub male_name: [c_char; MAX_VOICE_NAME],
    pub pause_middle: i32,
    pub pause_long: i32,
    pub pause_sentence: i32,
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
#[derive(Debug)]
pub struct SpeakerParam {
    pub voice_name: [c_char; MAX_VOICE_NAME],
    pub volume: f32,
    pub speed: f32,
    pub pitch: f32,
    pub range: f32,
    pub pause_middle: i32,
    pub pause_long: i32,
    pub pause_sentence: i32,
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
#[derive(Debug)]
pub struct TtsParam {
    pub size: u32,
    pub proc_text_buf: Option<unsafe extern "system" fn(i32, i32, *mut c_void) -> i32>,
    pub proc_raw_buf: Option<unsafe extern "system" fn(i32, i32, u64, *mut c_void) -> i32>,
    pub proc_event_tts: Option<unsafe extern "system" fn(i32, i32, u64, *const c_char, *mut c_void) -> i32>,
    pub len_text_buf_bytes: u32,
    pub len_raw_buf_bytes: u32,
    pub volume: f32,
    pub pause_begin: i32,
    pub pause_term: i32,
    pub extend_format: i32,
    pub voice_name: [c_char; MAX_VOICE_NAME],
    pub jeita: JeitaParam,
    pub num_speakers: u32,
    pub _reserved: i32,
    pub speakers: [SpeakerParam; 0],
}

#[derive(Debug)]
pub struct BoxedTtsParam {
    ptr: *mut TtsParam,
    layout: Layout,
}

impl BoxedTtsParam {
    pub fn new(len: usize) -> Self {
        let header_size = std::mem::size_of::<TtsParam>();
        let total_size = header_size + len * std::mem::size_of::<SpeakerParam>();
        let align = std::mem::align_of::<TtsParam>();

        let layout = Layout::from_size_align(total_size, align).unwrap();
        let ptr = unsafe { alloc(layout) as *mut TtsParam };
        if ptr.is_null() {
            panic!("Allocation failed");
        }

        unsafe {
            (*ptr).num_speakers = len as u32;
            (*ptr).size = total_size as u32;
        }

        Self { ptr, layout }
    }

    pub fn tts_param(&self) -> &TtsParam {
        unsafe { &*self.ptr }
    }

    pub fn tts_param_mut(&mut self) -> &mut TtsParam {
        unsafe { &mut *self.ptr }
    }

    pub fn speakers_mut(&mut self) -> &mut [SpeakerParam] {
        unsafe {
            std::slice::from_raw_parts_mut(
                (*self.ptr).speakers.as_mut_ptr(),
                (*self.ptr).num_speakers as usize
            )
        }
    }

    pub fn speakers(&self) -> &[SpeakerParam] {
        unsafe {
            std::slice::from_raw_parts(
                (*self.ptr).speakers.as_ptr(),
                (*self.ptr).num_speakers as usize
            )
        }
    }

    pub fn speakers_len(&self) -> usize {
        unsafe {
            (*self.ptr).num_speakers as usize
        }
    }
}

impl Drop for BoxedTtsParam {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr as *mut u8, self.layout)
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Config<'a> {
    hz_voice_db: u32,
    dir_voice_dbs: *const c_char,
    msec_timeout: u32,
    path_license: *const c_char,
    code_auth_seed: *const c_char,
    len_auth_seed: u32,
    _marker: &'a PhantomData<()>,
}

pub struct Aitalked<'lib> {
    init: Symbol<'lib, unsafe extern "system" fn(*const Config) -> ResultCode>,
    lang_load: Symbol<'lib, unsafe extern "system" fn(*const c_char) -> ResultCode>,
    voice_load: Symbol<'lib, unsafe extern "system" fn(*const c_char) -> ResultCode>,
    set_param: Symbol<'lib, unsafe extern "system" fn(*const TtsParam) -> ResultCode>,
    get_param: Symbol<'lib, unsafe extern "system" fn(*mut TtsParam, *mut u32) -> ResultCode>,
}

impl<'lib> Aitalked<'lib> {
    pub unsafe fn new(lib: &'lib Library) -> Result<Self> {
        let init = lib.get(b"_AITalkAPI_Init@4")?;
        let lang_load = lib.get(b"_AITalkAPI_LangLoad@4")?;
        let voice_load = lib.get(b"_AITalkAPI_VoiceLoad@4")?;
        let set_param = lib.get(b"_AITalkAPI_SetParam@4")?;
        let get_param = lib.get(b"_AITalkAPI_GetParam@8")?;
        Ok(Self {
            init,
            lang_load,
            voice_load,
            set_param,
            get_param,
        })
    }

    pub fn init(&self, config: &Config) -> ResultCode {
        unsafe { (self.init)(config) }
    }

    pub fn load_language(&self, lang_name: &str) -> ResultCode {
        let lang_name = CString::new(lang_name).unwrap();
        unsafe { (self.lang_load)(lang_name.as_ptr()) }
    }

    pub fn load_voice(&self, voice_name: &str) -> ResultCode {
        let voice_name = CString::new(voice_name).unwrap();
        unsafe { (self.voice_load)(voice_name.as_ptr()) }
    }

    pub fn get_param(&self, tts_param: *mut TtsParam, size: *mut u32) -> ResultCode {
        unsafe { (self.get_param)(tts_param, size) }
    }

    pub fn set_param(&self, tts_param: &TtsParam) -> ResultCode {
        unsafe { (self.set_param)(tts_param) }
    }
}
