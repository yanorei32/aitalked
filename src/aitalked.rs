use std::{
    ffi::{c_char, CString},
    marker::PhantomData,
};

use anyhow::Result;
use libloading::{Library, Symbol};

pub struct ConfigFactory {
    pub dir_voice_dbs: CString,
    pub path_license: CString,
    pub code_auth_seed: CString,
}

impl ConfigFactory {
    pub fn build(&self) -> Config {
        Config {
            hz_voice_db: 0xAC44,
            dir_voice_dbs: self.dir_voice_dbs.as_ptr(),
            msec_timeout: 1000,
            path_license: self.path_license.as_ptr(),
            code_auth_seed: self.code_auth_seed.as_ptr(),
            len_auth_seed: 0,
            _marker: &PhantomData,
        }
    }
}

#[repr(packed)]
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
    init: Symbol<'lib, unsafe extern "system" fn(*const Config) -> i32>,
    lang_load: Symbol<'lib, unsafe extern "system" fn(*const c_char) -> i32>,
    voice_load: Symbol<'lib, unsafe extern "system" fn(*const c_char) -> i32>,
}

impl<'lib> Aitalked<'lib> {
    pub unsafe fn new(lib: &'lib Library) -> Result<Self> {
        let init = lib.get(b"_AITalkAPI_Init@4")?;
        let lang_load = lib.get(b"_AITalkAPI_LangLoad@4")?;
        let voice_load = lib.get(b"_AITalkAPI_VoiceLoad@4")?;
        Ok(Self {
            init,
            lang_load,
            voice_load,
        })
    }

    pub fn init(&self, config: &Config) -> i32 {
        unsafe { (self.init)(config) }
    }

    pub fn load_language(&self, lang_name: &str) -> i32 {
        let lang_name = CString::new(lang_name).unwrap();
        unsafe { (self.lang_load)(lang_name.as_ptr()) }
    }

    pub fn load_voice(&self, voice_name: &str) -> i32 {
        let voice_name = CString::new(voice_name).unwrap();
        unsafe { (self.voice_load)(voice_name.as_ptr()) }
    }
}
