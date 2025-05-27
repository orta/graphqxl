use napi::bindgen_prelude::*;
use napi_derive::napi;
use graphqxl_parser::{parse_spec_vfs, VirtualFileSystem};
use graphqxl_transpiler::{transpile_spec, TranspileSpecOptions};
use graphqxl_synthesizer::{synth_spec, SynthConfig};
use std::path::{Path, PathBuf};
use std::error::Error;
use napi::*;

struct JsVfs<'a> {
    obj: &'a JsObject,
}

impl<'a> VirtualFileSystem for JsVfs<'a> {
    fn read_to_string(&self, path: &Path) -> Result<String, Box<dyn Error>> {
        let path_str = path.to_string_lossy();
        let js_val: JsString = self.obj.get(&path_str)?;
        Ok(js_val.into_utf8()?.as_str()?.to_string())
    }
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, Box<dyn Error>> {
        Ok(PathBuf::from(path))
    }
    fn exists(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.obj.has_own_property(path_str).unwrap_or(false)
    }
}

#[napi]
pub fn graphqxl_to_sdl(
    vfs: JsObject,
    entry_path: String,
    indent_spaces: Option<u32>,
    private_prefix: Option<String>,
) -> Result<String> {
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