use std::path::Path;

pub mod api;
pub mod binding;
pub mod model;

pub use libloading;

use libloading::Library;
use once_cell::sync::OnceCell;

static AITALKED_LIBRARY: OnceCell<Library> = OnceCell::new();
static AITALKED_BINDS: OnceCell<api::Aitalked<'static>> = OnceCell::new();

pub unsafe fn load_dll(dll_path: &Path) -> Result<(), libloading::Error> {
    AITALKED_LIBRARY
        .set(unsafe { Library::new(dll_path)? })
        .expect("Failed to init aitalked library (already initialized)");

    let aitalked_lib = AITALKED_LIBRARY.get().unwrap();

    AITALKED_BINDS
        .set(api::Aitalked::new(aitalked_lib)?)
        .unwrap();

    Ok(())
}
