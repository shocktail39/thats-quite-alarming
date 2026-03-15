use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

use native_tls::TlsConnector;

use crate::alarm::Alarm;
use crate::config;
use crate::json;
use crate::json::Value;

fn sanitize(input: &str) -> String {
    input.replace("\\", "\\\\").replace("\"", "\\\"")
}

fn send(request: &[u8]) {
    loop {
        let connector = TlsConnector::new().unwrap();
        let stream = TcpStream::connect(config::HTTP_SOCKET).unwrap();
        let mut stream = connector.connect(config::HTTP_ENDPOINT, stream).unwrap();

        stream.write_all(request).unwrap();
        stream.flush().unwrap();
        let mut response = vec![];
        stream.read_to_end(&mut response).unwrap();
        let response = String::from_utf8(response).unwrap();
        println!("{response}");

        if response.split_once("\r\n").is_some_and(|(first_line, _everything_after)|
            first_line.contains("429 Too Many Requests")
        ) {
            let time_to_sleep = if
                let Some((_headers, body)) = response.split_once("\r\n\r\n")
                && let Ok((Value::Object(response_json), _)) = json::parse_value(body.as_bytes(), 0)
                && let Some(Value::Number(milliseconds)) = response_json.get("retry_after")
                && let Ok(millis_as_u64) = TryInto::<u64>::try_into(milliseconds.as_int())
            {
                Duration::from_millis(millis_as_u64)
            } else {
                // if the endpoint fails to send the amount of time to wait,
                // we at least know it'll never be higher than 10 secs.
                Duration::from_secs(10)
            };
            std::thread::sleep(time_to_sleep);
            continue;
        }
        return;
    }
}

pub fn post_message(channel_id: &str, content: &str) {
    let channel = sanitize(channel_id);
    let content = sanitize(content);

    let body = format!(r#"{{"content":"{}","embeds":[]}}"#, content);
    let request = format!("POST /channels/{}/messages HTTP/1.0\r\nHost: {}\r\nContent-Length: {}\r\nX-Bot-Token: {}\r\n\r\n{}", channel, config::HTTP_ENDPOINT, body.len(), config::BOT_TOKEN, body);

    send(request.as_bytes());
}

pub fn post_alarm(alarm: Alarm) {
    let message = sanitize(&alarm.what);
    let channel = sanitize(&alarm.channel_id);
    let reply_to = sanitize(&alarm.message_id);

    let body = format!(r#"{{"content":"{}","replies":[{{"id":"{}","mention":true,"fail_if_not_exists":false}}]}}"#, message, reply_to);
    let request = format!("POST /channels/{}/messages HTTP/1.0\r\nHost: {}\r\nContent-Length: {}\r\nX-Bot-Token: {}\r\n\r\n{}", channel, config::HTTP_ENDPOINT, body.len(), config::BOT_TOKEN, body);

    send(request.as_bytes());
}

pub fn react(channel: &str, message: &str, emoji: &str) {
    let request = format!("PUT /channels/{}/messages/{}/reactions/{} HTTP/1.0\r\nHost: {}\r\nX-Bot-Token: {}\r\nContent-Length: 0\r\n\r\n", channel, message, emoji, config::HTTP_ENDPOINT, config::BOT_TOKEN);

    send(request.as_bytes());
}
