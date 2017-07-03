use super::*;

#[test]
fn test_default_request() {
    let req = Request::new().build();
    assert_eq!(req, "GET / HTTP/1.1\r\n\
                     Host: \r\n\
                     Connection: Close\r\n\
                     \r\n")
}

#[test]
fn test_request_header() {
    let req = Request::new().with_header("Content-Length: 45").build();
    assert_eq!(req,
    "GET / HTTP/1.1\r\n\
     Host: \r\n\
     Connection: Close\r\n\
     Content-Length: 45\r\n\
     \r\n")
}

#[test]
fn test_request_multiple_same_header() {
    let req = Request::new()
        .with_header("Content-Length: 45")
        .with_header("Content-Length: 20")
        .build();
    assert_eq!(req,
    "GET / HTTP/1.1\r\n\
     Host: \r\n\
     Connection: Close\r\n\
     Content-Length: 45\r\n\
     Content-Length: 20\r\n\
     \r\n")
}
