use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

use native_tls::TlsConnector;

use crate::alarm::Alarm;
use crate::config;

fn sanitize(input: String) -> String {
    input.replace("\\", "\\\\").replace("\"", "\\\"")
}

pub fn post_alarm(alarm: Alarm) {
    let message = sanitize(alarm.what);
    let channel = sanitize(alarm.channel_id);
    let reply_to = sanitize(alarm.message_id);

    let body = format!(r#"{{"content":"{}","replies":[{{"id":"{}","mention":true,"fail_if_not_exists":false}}]}}"#, message, reply_to);
    let request = format!("POST /channels/{}/messages HTTP/1.0\r\nHost: {}\r\nContent-Length: {}\r\nX-Bot-Token: {}\r\n\r\n{}", channel, config::HTTP_ENDPOINT, body.len(), config::BOT_TOKEN, body);

    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect(config::HTTP_SOCKET).unwrap();
    let mut stream = connector.connect(config::HTTP_ENDPOINT, stream).unwrap();

    stream.write_all(request.as_bytes()).unwrap();
    let mut buffer = vec![];
    stream.read_to_end(&mut buffer).unwrap();
}

pub fn react(channel: &str, message: &str, emoji: &str) {
    let request = format!("PUT /channels/{}/messages/{}/reactions/{} HTTP/1.0\r\nHost: {}\r\nX-Bot-Token: {}\r\n\r\n", channel, message, emoji, config::HTTP_ENDPOINT, config::BOT_TOKEN);

    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect(config::HTTP_SOCKET).unwrap();
    let mut stream = connector.connect(config::HTTP_ENDPOINT, stream).unwrap();

    stream.write_all(request.as_bytes()).unwrap();
    let mut buffer = vec![];
    stream.read_to_end(&mut buffer).unwrap();
}
