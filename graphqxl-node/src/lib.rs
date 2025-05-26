use napi::bindgen_prelude::*;
use napi_derive::napi;
use graphqxl_parser::parse_spec;
use graphqxl_transpiler::{transpile_spec, TranspileSpecOptions};
use graphqxl_synthesizer::{synth_spec, SynthConfig};

#[napi]
pub fn graphqxl_to_sdl(
    input: String,
    indent_spaces: Option<u32>,
    private_prefix: Option<String>,
) -> Result<String> {
    let private_prefix_val = private_prefix.unwrap_or_else(|| "_".to_string());
    let spec = parse_spec(&input).map_err(|e| Error::from_reason(format!("{:?}", e)))?;
    let transpiled = transpile_spec(
        &spec,
        &TranspileSpecOptions {
            private_prefix: private_prefix_val.clone(),
        },
    ).map_err(|e| Error::from_reason(format!("{:?}", e)))?;

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