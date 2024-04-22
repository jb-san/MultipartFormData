use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

enum ParsingState {
  Init,
  ReadingHeaders,
  ReadingData,
  ReadingPartSeparator,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileEntry {
  pub name: String,
  pub data: Vec<u8>,
  pub meta: HashMap<String, String>,
}

#[wasm_bindgen]
pub fn parse(array_buffer: &[u8], boundry: String) -> JsValue {
  let mut last_line = String::new();
  let mut iter = array_buffer.iter();
  let mut state = ParsingState::Init;
  while let Some(&byte) = iter.next() {
    let prev_byte = iter.as_slice().get(0).copied().unwrap_or(0);
    let new_line_detected = byte == b'\n' && prev_byte == b'\r';
    let new_line_char = byte == b'\n' || byte == b'\r';

    if !new_line_char {
      last_line.push(byte as char);
    }
  }
  let example = FileEntry {
    name: "field1".to_string(),
    data: vec![1, 2, 3],
    meta: HashMap::new(),
  };
  return serde_wasm_bindgen::to_value(&example).unwrap();
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = parse(1, 2);
    assert_eq!(result, 3);
  }
}
