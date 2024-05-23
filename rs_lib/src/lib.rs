use nom::{
  branch::alt,
  bytes::complete::{tag, take_until},
  combinator::{map, not, opt, peek},
  multi::{many0, many_till},
  sequence::{delimited, preceded, terminated},
  IResult,
};

#[derive(Debug)]
pub struct FormData {
  segments: Vec<Segment>,
}

#[derive(Debug)]
struct Segment {
  headers: Vec<Header>,
  body: Vec<u8>,
}

#[derive(Debug)]
struct Header {
  name: String,
  value: String,
}

pub fn parse_multipart<'a>(
  input: &'a [u8],
  boundary: &'a str,
) -> IResult<&'a [u8], FormData> {
  let (input, segments) = many0(parse_segment(boundary))(input)?;

  Ok((input, FormData { segments }))
}

fn parse_boundary<'a>(
  boundary: &'a str,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
  let moved = boundary;

  move |input| {
    // let res = tag(boundary_marker.as_str())(input);
    let res = alt((
      delimited(tag("\r\n--"), tag(moved), tag("\r\n")),
      delimited(tag("--"), tag(moved), tag("\r\n")),
    ))(input);
    res
  }
}

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

fn parse_headers(input: &[u8]) -> IResult<&[u8], Vec<Header>> {
  let endOfHeaders = tag(b"\r\n\r\n");
  let (input, headers) = many0(terminated(parse_header, endOfHeaders))(input)?;
  // println!("{:?}", headers);
  Ok((input, headers))
}
fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
  let (input, _) = opt(tag("\r\n"))(input)?;
  let (input, name) = take_until(":")(input)?;
  let (input, _) = tag(": ")(input)?;
  let (input, value) = take_until("\r\n")(input)?;
  // let (input, _) = peek(not(tag("\r\n\r\n")))(input)?;

  // let (input, _) = not(tag("\r\n\r\n"))(input)?;
  let value = String::from_utf8_lossy(value).into_owned();
  let name = String::from_utf8_lossy(name).into_owned();
  println!("{:?}", name);
  println!("{:?}", value);
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
mod tests {

  use std::{fs::File, io::Read};

  use super::*;
  fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    // let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("moc");
    // env::set_current_dir(&test_dir).expect("Failed to set current directory");

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
  }
  // Add your test functions here
  #[test]
  fn test_example() {
    let file_path = "./mocks/e80da7018c0ad932ca2d0e9ccf4186055af4614c9f4699fe2e371ca8c613.response";
    let file_data = read_file(file_path).unwrap();

    let input = file_data.as_ref();
    let result = parse_multipart(
      input,
      "e80da7018c0ad932ca2d0e9ccf4186055af4614c9f4699fe2e371ca8c613",
    )
    .unwrap();
    println!("{:?}", result.1.segments[0].headers);
    // assert_eq!(result.1, ("", "", "", "", ""));
  }
  #[test]
  fn test_parse_header() {
    let data = b"Content-Disposition: form-data; name=\"field1\"\r\nContent-Disposition: form-data; name=\"field2\"\r\n\r\ntest2--boundary--\r\n";

    let result = many0(parse_header)(data);

    match result {
      Ok((rest, header)) => println!("{:#?}", header),
      Err(e) => eprintln!("Error: {:?}", e),
    }
  }
  #[test]
  fn test_parse_body() {
    let data = b"\r\n";

    let result = many0(parse_header)(data);

    match result {
      Ok((_, header)) => println!("{:#?}", header),
      Err(e) => eprintln!("Error: {:?}", e),
    }
  }
  #[test]
  fn simple_test() {
    let boundary = "boundary";
    // let data = b"--boundary\r\nContent-Disposition: form-data; name=\"field2\"; filename=\"example.txt\"\r\nContent-Type: text/plain\r\n\r\nvalue2\r\n--boundary--";
    let data = b"--boundary\r\nContent-Disposition: form-data; name=\"field1\"\r\n\r\nvalue1\r\n--boundary\r\nContent-Disposition: form-data; name=\"field2\"; filename=\"example.txt\"\r\nContent-Type: text/plain\r\n\r\nvalue2\r\n--boundary--";

    let result = parse_multipart(data, boundary);

    match result {
      Ok((_, form_data)) => println!(
        "{:#?}",
        form_data //
                  // String::from_utf8_lossy(&form_data.segments[1].body) // Convert Vec<u8> to &[u8]
      ),
      Err(e) => eprintln!("Error: {:?}", e),
    }
  }
}
