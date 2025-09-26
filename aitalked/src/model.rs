use std::alloc::{alloc, dealloc, Layout};

use crate::binding::*;

#[derive(Debug)]
pub struct BoxedTtsParam {
    ptr: *mut TtsParam,
    layout: Layout,
}

unsafe impl Send for BoxedTtsParam {
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
                (*self.ptr).num_speakers as usize,
            )
        }
    }

    pub fn speakers(&self) -> &[SpeakerParam] {
        unsafe {
            std::slice::from_raw_parts(
                (*self.ptr).speakers.as_ptr(),
                (*self.ptr).num_speakers as usize,
            )
        }
    }

    pub fn speakers_len(&self) -> usize {
        unsafe { (*self.ptr).num_speakers as usize }
    }
}

impl Drop for BoxedTtsParam {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr as *mut u8, self.layout) }
    }
}
