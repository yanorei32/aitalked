use std::ffi::{c_char, c_void, CStr};

use libloading::{Library, Symbol};

use crate::binding::*;
use crate::AITALKED_BINDS;

#[derive(Debug)]
pub(crate) struct Aitalked<'lib> {
    init: Symbol<'lib, unsafe extern "system" fn(*const AitalkedConfig) -> ResultCode>,
    lang_load: Symbol<'lib, unsafe extern "system" fn(*const c_char) -> ResultCode>,
    lang_clear: Symbol<'lib, unsafe extern "system" fn() -> ResultCode>,
    voice_load: Symbol<'lib, unsafe extern "system" fn(*const c_char) -> ResultCode>,
    voice_clear: Symbol<'lib, unsafe extern "system" fn() -> ResultCode>,
    set_param: Symbol<'lib, unsafe extern "system" fn(*const TtsParam) -> ResultCode>,
    get_param: Symbol<'lib, unsafe extern "system" fn(*mut TtsParam, *mut u32) -> ResultCode>,
    text_to_kana: Symbol<
        'lib,
        unsafe extern "system" fn(*mut i32, *const JobParam, *const c_char) -> ResultCode,
    >,
    get_kana: Symbol<
        'lib,
        unsafe extern "system" fn(i32, *mut u8, u32, *mut u32, *mut u32) -> ResultCode,
    >,
    close_kana: Symbol<'lib, unsafe extern "system" fn(i32, i32) -> ResultCode>,
    text_to_speech: Symbol<
        'lib,
        unsafe extern "system" fn(*mut i32, *const JobParam, *const c_char) -> ResultCode,
    >,
    get_data: Symbol<'lib, unsafe extern "system" fn(i32, *mut u8, u32, *mut u32) -> ResultCode>,
    close_speech: Symbol<'lib, unsafe extern "system" fn(i32, i32) -> ResultCode>,
    reload_phrase_dic: Symbol<'lib, unsafe extern "system" fn(*const c_char) -> ResultCode>,
    reload_word_dic: Symbol<'lib, unsafe extern "system" fn(*const c_char) -> ResultCode>,
    reload_symbol_dic: Symbol<'lib, unsafe extern "system" fn(*const c_char) -> ResultCode>,
}

impl<'lib> Aitalked<'lib> {
    pub(crate) unsafe fn new(lib: &'lib Library) -> Result<Self, libloading::Error> {
        let init = lib.get(b"_AITalkAPI_Init@4")?;
        let lang_load = lib.get(b"_AITalkAPI_LangLoad@4")?;
        let lang_clear = lib.get(b"_AITalkAPI_LangClear@0")?;
        let voice_load = lib.get(b"_AITalkAPI_VoiceLoad@4")?;
        let voice_clear = lib.get(b"_AITalkAPI_VoiceClear@0")?;
        let set_param = lib.get(b"_AITalkAPI_SetParam@4")?;
        let get_param = lib.get(b"_AITalkAPI_GetParam@8")?;
        let text_to_kana = lib.get(b"_AITalkAPI_TextToKana@12")?;
        let get_kana = lib.get(b"_AITalkAPI_GetKana@20")?;
        let close_kana = lib.get(b"_AITalkAPI_CloseKana@8")?;
        let text_to_speech = lib.get(b"_AITalkAPI_TextToSpeech@12")?;
        let close_speech = lib.get(b"_AITalkAPI_CloseSpeech@8")?;
        let get_data = lib.get(b"_AITalkAPI_GetData@16")?;
        let reload_phrase_dic = lib.get(b"_AITalkAPI_ReloadPhraseDic@4")?;
        let reload_word_dic = lib.get(b"_AITalkAPI_ReloadWordDic@4")?;
        let reload_symbol_dic = lib.get(b"_AITalkAPI_ReloadSymbolDic@4")?;

        Ok(Self {
            init,
            lang_load,
            lang_clear,
            voice_load,
            voice_clear,
            set_param,
            get_param,
            text_to_kana,
            get_kana,
            close_kana,
            text_to_speech,
            close_speech,
            get_data,
            reload_phrase_dic,
            reload_word_dic,
            reload_symbol_dic,
        })
    }
}

pub unsafe fn init(config: &AitalkedConfig) -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .init)(config)
}

/// NOTE: Install DirectoryがCurrent Working Directoryでないと正常に動作しない
pub unsafe fn lang_load(lang_name: &CStr) -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .lang_load)(lang_name.as_ptr())
}

pub unsafe fn lang_clear() -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .lang_clear)()
}

pub unsafe fn voice_load(voice_name: &CStr) -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .voice_load)(voice_name.as_ptr())
}

pub unsafe fn voice_clear() -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .voice_clear)()
}

pub unsafe fn get_param(tts_param: *mut TtsParam, size: *mut u32) -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .get_param)(tts_param, size)
}

pub unsafe fn set_param(tts_param: &TtsParam) -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .set_param)(tts_param)
}

pub unsafe fn text_to_kana(job_id: &mut i32, user_data: *mut c_void, text: &CStr) -> ResultCode {
    let job_param = JobParam {
        user_data,
        model_in_out: JobInOut::PLAIN_TO_AIKANA,
    };

    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .text_to_kana)(job_id, &job_param, text.as_ptr())
}

pub unsafe fn text_to_speech(job_id: &mut i32, user_data: *mut c_void, text: &CStr) -> ResultCode {
    let job_param = JobParam {
        user_data,
        model_in_out: JobInOut::AIKANA_TO_WAVE,
    };

    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .text_to_speech)(job_id, &job_param, text.as_ptr())
}

pub unsafe fn get_kana(
    job_id: i32,
    buffer: &mut [u8],
    bytes_read: &mut u32,
    position: &mut u32,
) -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .get_kana)(
        job_id,
        buffer.as_mut_ptr(),
        buffer.len() as u32,
        bytes_read,
        position,
    )
}

pub unsafe fn get_data(job_id: i32, buffer: &mut [u8], words_read: &mut u32) -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .get_data)(
        job_id,
        buffer.as_mut_ptr(),
        (buffer.len() / 2) as u32,
        words_read,
    )
}

/// unknownは0にしておくとよろしいらしい
/// REF: https://github.com/Nkyoku/pyvcroid2/blob/01d7e4b30e6b055f8cf1a3b0ad1c35d211754027/pyvcroid2/pyvcroid2.py#L396
pub unsafe fn close_kana(job_id: i32, unknown: i32) -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .close_kana)(job_id, unknown)
}

/// unknownは0にしておくとよろしいらしい
/// REF: https://github.com/Nkyoku/pyvcroid2/blob/01d7e4b30e6b055f8cf1a3b0ad1c35d211754027/pyvcroid2/pyvcroid2.py#L492
pub unsafe fn close_speech(job_id: i32, unknown: i32) -> ResultCode {
    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .close_speech)(job_id, unknown)
}

/// NOTE: Install DirectoryがCurrent Working Directoryでないと正常に動作しない
pub unsafe fn reload_phrase_dic(path: Option<&CStr>) -> ResultCode {
    let path = match path {
        Some(path) => path.as_ptr(),
        None => std::ptr::null(),
    };

    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .reload_phrase_dic)(path)
}

/// NOTE: Install DirectoryがCurrent Working Directoryでないと正常に動作しない
pub unsafe fn reload_word_dic(path: Option<&CStr>) -> ResultCode {
    let path = match path {
        Some(path) => path.as_ptr(),
        None => std::ptr::null(),
    };

    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .reload_word_dic)(path)
}

/// NOTE: Install DirectoryがCurrent Working Directoryでないと正常に動作しない
pub unsafe fn reload_symbol_dic(path: Option<&CStr>) -> ResultCode {
    let path = match path {
        Some(path) => path.as_ptr(),
        None => std::ptr::null(),
    };

    (AITALKED_BINDS
        .get()
        .expect("aitalked::load_dll() is not called.")
        .reload_symbol_dic)(path)
}
