use napi_derive::napi;
use graphqxl_parser::{parse_spec_vfs, VirtualFileSystem};
use graphqxl_transpiler::{transpile_spec, TranspileSpecOptions};
use graphqxl_synthesizer::{synth_spec, SynthConfig};
use std::path::{Path, PathBuf};
use std::error::Error as StdError;
use napi::{Error, JsObject, JsString};

struct JsVfs<'a> {
    obj: &'a JsObject,
}

impl<'a> VirtualFileSystem for JsVfs<'a> {
    fn read_to_string(&self, path: &Path) -> std::result::Result<String, Box<dyn StdError>> {
        let path_str = path.to_string_lossy();
        match self.obj.get::<_, JsString>(&path_str.as_ref()) {
            Ok(Some(js_val)) => match js_val.into_utf8() {
                Ok(utf8_val) => match utf8_val.as_str() {
                    Ok(s) => Ok(s.to_string()),
                    Err(e) => Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to convert to string: {}", e),
                    ))),
                },
                Err(e) => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to convert to UTF-8: {}", e),
                ))),
            },
            Ok(None) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Path not found in VFS: {}", path_str),
            ))),
            Err(e) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get from object: {}", e),
            ))),
        }
    }
    
    fn canonicalize(&self, path: &Path) -> std::result::Result<PathBuf, Box<dyn StdError>> {
        Ok(PathBuf::from(path))
    }
    
    fn exists(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.obj.has_own_property(&path_str.as_ref()).unwrap_or(false)
    }
}

#[napi]
pub fn graphqxl_to_sdl(
    vfs: JsObject,
    entry_path: String,
    indent_spaces: Option<u32>,
    private_prefix: Option<String>,
) -> napi::Result<String> {
    let js_vfs = JsVfs { obj: &vfs };
    let private_prefix_val = private_prefix.unwrap_or_else(|| "_".to_string());
    let spec = parse_spec_vfs(&entry_path, &js_vfs)
        .map_err(|e| Error::from_reason(format!("{:?}", e)))?;
    let transpiled = transpile_spec(
        &spec,
        &TranspileSpecOptions {
            private_prefix: private_prefix_val.clone(),
        },
    )
    .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

    let (result, _) = synth_spec(
        transpiled,
        SynthConfig {
            indent_spaces: indent_spaces.unwrap_or(2) as usize,
            private_prefix: private_prefix_val,
            ..Default::default()
        },
    );
    Ok(result)
}