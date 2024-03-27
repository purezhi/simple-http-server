use std::error::Error;
use std::fmt;
use std::io;
use std::ops::Deref;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local, TimeZone};
use iron::headers;
use iron::status;
use iron::{IronError, Response};
use percent_encoding::{utf8_percent_encode, AsciiSet};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT_ENCODE_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`');
/// https://url.spec.whatwg.org/#path-percent-encode-set
const PATH_ENCODE_SET: &AsciiSet = &FRAGMENT_ENCODE_SET.add(b'#').add(b'?').add(b'{').add(b'}');
const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &PATH_ENCODE_SET.add(b'/').add(b'%').add(b'[').add(b']');

// Site favicon image
pub const FAVICON_IMAGE: &str = r#"<link rel="shortcut icon" href="data:image/png;base64,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" />"#;

pub fn root_link(baseurl: &str) -> String {
    format!(
        r#"<a href="{baseurl}"><strong>[Root]</strong></a>"#,
        baseurl = baseurl,
    )
}

#[derive(Debug)]
pub struct StringError(pub String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl Deref for StringError {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn enable_string(value: bool) -> String {
    (if value { "enabled" } else { "disabled" }).to_owned()
}

pub fn encode_link_path(path: &[String]) -> String {
    path.iter()
        .map(|s| utf8_percent_encode(s, PATH_SEGMENT_ENCODE_SET).to_string())
        .collect::<Vec<String>>()
        .join("/")
}

pub fn error_io2iron(err: io::Error) -> IronError {
    let status = match err.kind() {
        io::ErrorKind::PermissionDenied => status::Forbidden,
        io::ErrorKind::NotFound => status::NotFound,
        _ => status::InternalServerError,
    };
    IronError::new(err, status)
}

/* TODO: may not used

use iron::headers::{Range, ByteRangeSpec};

#[allow(dead_code)]
pub fn parse_range(ranges: &Vec<ByteRangeSpec>, total: u64)
                   -> Result<Option<(u64, u64)>, IronError> {
    if let Some(range) = ranges.get(0) {
        let (offset, length) = match range {
            &ByteRangeSpec::FromTo(x, mut y) => { // "x-y"
                if x >= total || x > y {
                    return Err(IronError::new(
                        StringError(format!("Invalid range(x={}, y={})", x, y)),
                        status::RangeNotSatisfiable
                    ));
                }
                if y >= total {
                    y = total - 1;
                }
                (x, y - x + 1)
            }
            &ByteRangeSpec::AllFrom(x) => { // "x-"
                if x >= total {
                    return Err(IronError::new(
                        StringError(format!(
                            "Range::AllFrom to large (x={}), Content-Length: {})",
                            x, total)),
                        status::RangeNotSatisfiable
                    ));
                }
                (x, total - x)
            }
            &ByteRangeSpec::Last(mut x) => { // "-x"
                if x > total {
                    x = total;
                }
                (total - x, x)
            }
        };
        Ok(Some((offset, length)))
    } else {
        return Err(IronError::new(
            StringError("Empty range set".to_owned()),
            status::RangeNotSatisfiable
        ));
    }
}
*/

pub fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn system_time_to_date_time(t: SystemTime) -> DateTime<Local> {
    let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => {
            // unlikely but should be handled
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        }
    };
    Local.timestamp_opt(sec, nsec).unwrap()
}

pub fn error_resp(s: status::Status, msg: &str, baseurl: &str) -> Response {
    let mut resp = Response::with((
        s,
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  {favicon_image}
  <title>Simple HTTP(s) Server</title>
</head>
<body>
  {root_link}
  <hr />
  <div>[<strong style=color:red;>ERROR {code}</strong>]: {msg}</div>
</body>
</html>
"#,
            favicon_image = FAVICON_IMAGE,
            root_link = root_link(baseurl),
            code = s.to_u16(),
            msg = msg
        ),
    ));
    resp.headers.set(headers::ContentType::html());
    resp
}
