use ftaql::{analyze_module, parse::parse_module_from_string};
use serde_json::to_string;
use wasm_bindgen::prelude::*;

#[cfg(test)]
mod lib_tests;

#[wasm_bindgen]
pub fn analyze_file_ftaql(source_code: &str, use_tsx: bool, include_comments: bool) -> String {
    let (parsed_module_result, line_count) =
        parse_module_from_string(source_code, use_tsx, include_comments);

    let module = match parsed_module_result {
        Ok(result) => result,
        Err(err) => {
            wasm_bindgen::throw_str(&format!("Failed to parse module: {:?}", err));
        }
    };

    let file_data = analyze_module(&module, "source.ts", line_count);

    to_string(&file_data).unwrap()
}
