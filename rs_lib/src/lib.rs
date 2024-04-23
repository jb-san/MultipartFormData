use bytes::Bytes;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
// Structure to represent form data parts
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormData {
  headers: Vec<(String, String)>,
  body: Vec<u8>,
}

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
  segments.push(buffer.slice(start..end));
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
      if segment[i] == b'\r' && segment[i + 1] == b'\n' {
        if start == 0 {
          break;
        }
        let header_str = String::from_utf8(segment.slice(start..end).to_vec())
          .map_err(|_| "Failed to parse header")?;
        let parts: Vec<&str> = header_str.split(": ").collect();
        if parts.len() != 2 {
          return Err("Invalid header".to_string());
        }
        headers.push((parts[0].to_string(), parts[1].to_string()));
        i += 2;
        start = i;
        end = i;
        header = false;
      }
    } else {
      if segment[i] == b'\r'
        && segment[i + 1] == b'\n'
        && segment[i + 2] == b'\r'
        && segment[i + 3] == b'\n'
      {
        body = segment.slice(i + 4..segment.len()).to_vec();
        break;
      }
    }
    end += 1;
    i += 1;
  }
  Ok(FormData { headers, body })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_multiple_entries() {
    // Define the multipart/form-data content
    let multipart_data = r#"------WebKitFormBoundary7MA4YWxkTrZu0gW
       Content-Disposition: form-data; name="username"
       
       john_doe
       ------WebKitFormBoundary7MA4YWxkTrZu0gW
       Content-Disposition: form-data; name="file"; filename="example.txt"
       Content-Type: text/plain
       
       Hello, world!
       ------WebKitFormBoundary7MA4YWxkTrZu0gW--"#;

    // Convert the string to a vector of bytes
    let buffer: Vec<u8> = multipart_data.as_bytes().to_vec();

    let boundry: String = String::from("----WebKitFormBoundary7MA4YWxkTrZu0gW");
    let result = parse(buffer, boundry);
    assert_eq!(result, 3);
  }
}
