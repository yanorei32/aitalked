#![allow(clippy::missing_safety_doc)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub mod api;
pub mod binding;
pub mod model;

pub use libloading;

use libloading::Library;
use once_cell::sync::OnceCell;

static AITALKED_BINDS: OnceCell<Mutex<HashMap<PathBuf, api::Aitalked>>> = OnceCell::new();

pub unsafe fn load_dll(dll_path: &Path) -> Result<api::Aitalked, libloading::Error> {
    let mut binds = AITALKED_BINDS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    if let Some(aitalked) = binds.get(dll_path) {
        return Ok(*aitalked);
    }

    let library = unsafe { Library::new(dll_path)? };

    let library = Box::leak(Box::new(library));

    let inner = Box::leak(Box::new(api::AitalkedInner::new(library)?));

    binds.insert(dll_path.to_owned(), api::Aitalked { inner });

    Ok(*binds.get(dll_path).unwrap())
}
