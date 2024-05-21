use bytes::Bytes;
use serde::{Deserialize, Serialize};
// use std::env;
use std::fs::File;
use std::io::Read;
// use std::path::Path;
// use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsValue;
// Structure to represent form data parts
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormData {
  headers: Vec<(String, String)>,
  body: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn parse(array_buffer: Vec<u8>, boundry: String) -> JsValue {
  let buffer = Bytes::from(array_buffer);

  // Placeholder for storing parsed form data
  let mut parts: Vec<FormData> = Vec::new();
  // Split the buffer by the boundary
  let segments = split_by_boundary(&buffer, &boundry).unwrap();
  // Parse each segment into FormData
  for segment in segments {
    let form_data = parse_segment(segment).unwrap();
    parts.push(form_data);
  }
  serde_wasm_bindgen::to_value(&parts).unwrap()
}
#[cfg(not(target_arch = "wasm32"))]
pub fn parse(array_buffer: Vec<u8>, boundry: String) -> Vec<FormData> {
  let buffer = Bytes::from(array_buffer);

  // Placeholder for storing parsed form data
  let mut parts: Vec<FormData> = Vec::new();
  // Split the buffer by the boundary
  let segments = split_by_boundary(&buffer, &boundry).unwrap();
  // Parse each segment into FormData
  for segment in segments {
    let form_data = parse_segment(segment).unwrap();
    parts.push(form_data);
  }
  return parts;
}

pub fn split_by_boundary(
  buffer: &Bytes,
  boundry: &str,
) -> Result<Vec<Bytes>, String> {
  let mut segments: Vec<Bytes> = Vec::new();
  let mut start = 0;
  let mut end = 0;
  let mut i = 0;
  while i < buffer.len() {
    if buffer[i] == boundry.chars().nth(0).unwrap() as u8 {
      if i + boundry.len() < buffer.len() {
        if &buffer[i..i + boundry.len()] == boundry.as_bytes() {
          if start != 0 {
            segments.push(buffer.slice(start..end));
          }
          start = i + boundry.len();
          end = start;
          i += boundry.len();
          continue;
        }
      }
    }
    end += 1;
    i += 1;
  }
  // Check if the last segment is the end boundary
  if segments
    .last()
    .map(|s| s == boundry.as_bytes())
    .unwrap_or(false)
  {
    segments.pop();
  }
  Ok(segments)
}

pub fn parse_segment(segment: Bytes) -> Result<FormData, String> {
  let mut headers: Vec<(String, String)> = Vec::new();
  let mut body: Vec<u8> = Vec::new();
  let mut i = 0;
  let mut start = 0;
  let mut end = 0;
  let mut header = true;
  while i < segment.len() {
    if header {
      //

      // Check if the header is complete
      if segment[i] == (b'\r' as u8)
        && segment[i + 1] == (b'\n' as u8)
        && end > start
      {
        let header_str = String::from_utf8(segment.slice(start..end).to_vec())
          .map_err(|_| "Failed to parse header")?;

        let parts: Vec<&str> = header_str.split(": ").collect();
        if parts.len() == 2 {
          // return Err("Invalid header".to_string());
          headers.push((parts[0].to_string(), parts[1].to_string()));
        } else {
          header = false
        }
      }
    }
    if segment[i] == b'\r' as u8
      && segment[i + 1] == b'\n' as u8
      && segment[i + 2] == b'\r' as u8
      && segment[i + 3] == b'\n' as u8
    {
      body = segment.slice(i + 4..segment.len()).to_vec();
      break;
    }

    // set the start of the header
    if segment[i] == (b'\r' as u8) && segment[i + 1] == (b'\n' as u8) {
      start = i + 2;
    }
    end += 1;
    i += 1;
  }
  // println!("headers {:?}", headers);
  // println!("body {:?}", body);

  Ok(FormData { headers, body })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    // let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("moc");
    // env::set_current_dir(&test_dir).expect("Failed to set current directory");

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
  }
  #[test]
  fn parse_multiple_entries() {
    // Define the multipart/form-data content

    let file_path = "./mocks/c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7.response";
    let file_data = read_file(file_path).unwrap();

    let boundry: String = String::from(
      "c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7",
    );
    let result = parse(file_data, boundry);
    // Check if the result is as expected
    // println!("{:?}", result[0]);
    assert_eq!(result.len(), 10);
    // assert_eq!(result[0], );
  }
}
