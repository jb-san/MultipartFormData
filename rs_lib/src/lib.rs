use nom::{
  branch::alt,
  bytes::complete::{tag, take_until},
  combinator::opt,
  multi::{many0, many_till},
  sequence::delimited,
  IResult,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormData {
  segments: Vec<Segment>,
}
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Segment {
  headers: Vec<Header>,
  body: Vec<u8>,
}
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Header {
  name: String,
  value: String,
}
#[wasm_bindgen]
pub fn parse_multipart(input: &[u8], boundary: &str) -> JsValue {
  let (_, segments) = many0(parse_segment(&boundary))(&input).unwrap();
  let form_data = FormData { segments };
  return serde_wasm_bindgen::to_value(&form_data).unwrap();
}
/**
 * A multipart/form-data payload is a series of segments, each of which is separated by a boundary.
 */
fn parse_segment<'a>(
  boundary: &'a str,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Segment> + '_ {
  let moved = boundary;
  move |input| {
    let (input, _) = parse_boundary(moved)(input)?;
    let (input, (headers, _)) =
      many_till(parse_header, tag(b"\r\n\r\n"))(input)?; // Extract headers from the tuple
    println!("{:?}", headers);
    let (input, body) = parse_body(moved)(input)?;

    Ok((input, Segment { headers, body })) // Pass headers and body separately to the Segment struct
  }
}

/**
 * A boundry looks like this:
 * --boundary\r\n
 */
fn parse_boundary<'a>(
  boundary: &'a str,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
  let moved = boundary;

  move |input| {
    let res = alt((
      delimited(tag("\r\n--"), tag(moved), tag("\r\n")),
      delimited(tag("--"), tag(moved), tag("\r\n")),
    ))(input);
    res
  }
}

/**
 * A header looks like this:
 * Content-Disposition: form-data; name="file"; filename="example.txt"\r\n
 */
fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
  let (input, _) = opt(tag("\r\n"))(input)?;
  let (input, name) = take_until(":")(input)?;
  let (input, _) = tag(": ")(input)?;
  let (input, value) = take_until("\r\n")(input)?;

  let value = String::from_utf8_lossy(value).into_owned();
  let name = String::from_utf8_lossy(name).into_owned();

  Ok((input, Header { name, value }))
}

fn parse_body(boundary: &str) -> impl Fn(&[u8]) -> IResult<&[u8], Vec<u8>> {
  let boundary_marker = format!("\r\n--{}", boundary);
  move |input: &[u8]| {
    let (remaining, body) = take_until(boundary_marker.as_str())(input)?;
    Ok((remaining, body.to_vec()))
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::{fs::File, io::Read};
  fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    // let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("moc");
    // env::set_current_dir(&test_dir).expect("Failed to set current directory");

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
  }
  #[test]
  fn test_example() {
    let file_path = "./mocks/c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7.response";
    let file_data = read_file(file_path).unwrap();

    let input = file_data.as_ref();
    let result = parse_multipart(
      input,
      "c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7",
    );
  }
}
