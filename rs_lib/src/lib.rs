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
  println!("{:?}", segment);

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
    let multipart_data = r#"------WebKitFormBoundary7MA4YWxkTrZu0gW\r\n
       Content-Disposition: form-data; name="username"
       
       john_doe
       ------WebKitFormBoundary7MA4YWxkTrZu0gW
       Content-Disposition: form-data; name="file"; filename="example.txt"
       Content-Type: text/plain
       
       Hello, world!
       ------WebKitFormBoundary7MA4YWxkTrZu0gW--"#;
    let body = r#"trash1\r\n
       ------WebKitFormBoundaryvef1fLxmoUdYZWXp\r\n
       Content-Type: text/plain\r\n
       Content-Disposition: form-data; name="uploads[]"; filename="A.txt"\r\n
       \r\n
       @11X111Y\r\n
       111Z\rCCCC\nCCCC\r\nCCCCC@\r\n\r\n
       ------WebKitFormBoundaryvef1fLxmoUdYZWXp\r\n
       Content-Type: text/plain\r\n
       Content-Disposition: form-data; name="uploads[]"; filename="B.txt"\r\n
       \r\n
       @22X222Y\r\n
       222Z\r222W\n2220\r\n666@\r\n
       ------WebKitFormBoundaryvef1fLxmoUdYZWXp\r\n
       Content-Disposition: form-data; name="input1"\r\n
       \r\n
       value1\r\n
       ------WebKitFormBoundaryvef1fLxmoUdYZWXp--\r\n"#;

    // Convert the string to a vector of bytes
    let buffer: Vec<u8> = body.as_bytes().to_vec();

    let boundry: String =
      String::from("----WebKitFormBoundaryvef1fLxmoUdYZWXp");
    let result = parse(buffer, boundry);
    // Check if the result is as expected
    println!("{:?}", result);
    assert_eq!(result.len(), 3);
    // assert_eq!(result[0], );
  }
}
