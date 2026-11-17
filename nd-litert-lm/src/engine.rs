use crate::Error;
use nd_litert_lm_sys::LiteRtLmEngine;
use nd_litert_lm_sys::LiteRtLmEngineSettings;
use nd_litert_lm_sys::LiteRtLmStatusCode;
use nd_litert_lm_sys::litert_lm_engine_create;
use nd_litert_lm_sys::litert_lm_engine_delete;
use nd_litert_lm_sys::litert_lm_engine_settings_create;
use nd_litert_lm_sys::litert_lm_engine_settings_delete;
use std::ffi::CStr;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::path::Path;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EngineSettingsBackend {
    Cpu,
    Gpu,
}

impl EngineSettingsBackend {
    fn as_cstr(self) -> &'static CStr {
        match self {
            Self::Cpu => c"cpu",
            Self::Gpu => c"gpu",
        }
    }
}

#[derive(Debug)]
pub struct EngineSettings {
    ptr: *mut LiteRtLmEngineSettings,
}

impl EngineSettings {
    /// Create new engine settings.
    ///
    /// # Returns
    /// Returns new engine settings is successful.
    /// Returns an error on failure.
    /// Returns an error if the model path is not utf8.
    ///
    /// # Panics
    /// Panics if `model_path` has an interior NUL.
    pub fn new<P>(
        model_path: P,
        backend: EngineSettingsBackend,
        vision_backend: Option<EngineSettingsBackend>,
        audio_backend: Option<EngineSettingsBackend>,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let model_path = model_path.as_ref();
        let model_path = model_path
            .to_str()
            .ok_or_else(|| Error::custom("model_path is not utf8"))?;
        let model_path = CString::new(model_path).expect("model_path has an interior NUL");
        let backend = backend.as_cstr();
        let vision_backend = vision_backend
            .map(|backend| backend.as_cstr().as_ptr())
            .unwrap_or(std::ptr::null());
        let audio_backend = audio_backend
            .map(|backend| backend.as_cstr().as_ptr())
            .unwrap_or(std::ptr::null());
        let mut settings: MaybeUninit<*mut LiteRtLmEngineSettings> = MaybeUninit::zeroed();
        let code = unsafe {
            litert_lm_engine_settings_create(
                model_path.as_ptr(),
                backend.as_ptr(),
                vision_backend,
                audio_backend,
                settings.as_mut_ptr(),
            )
        };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        let ptr = unsafe { settings.assume_init() };

        Ok(Self { ptr })
    }
}

impl Drop for EngineSettings {
    fn drop(&mut self) {
        unsafe {
            litert_lm_engine_settings_delete(self.ptr);
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    ptr: *mut LiteRtLmEngine,
}

impl Engine {
    /// Create a new `Engine`.
    pub fn new(settings: &EngineSettings) -> Result<Self, Error> {
        let mut engine: MaybeUninit<*mut LiteRtLmEngine> = MaybeUninit::zeroed();
        let code = unsafe { litert_lm_engine_create(settings.ptr, engine.as_mut_ptr()) };
        if code != LiteRtLmStatusCode::kLiteRtLmStatusOk {
            return Err(Error::last_error_clear());
        }
        let ptr = unsafe { engine.assume_init() };
        Ok(Self { ptr })
    }

    /// Get the engine ptr.
    pub fn as_ptr(&self) -> *mut LiteRtLmEngine {
        self.ptr
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe { litert_lm_engine_delete(self.ptr) }
    }
}
