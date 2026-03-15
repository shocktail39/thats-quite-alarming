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

fn send(request: &[u8]) -> Result<(), String> {
    loop {
        let mut stream = {
            let Ok(connector) = TlsConnector::new() else {
                return Err("https: failed to create tls connector".to_string());
            };
            let Ok(tcp_stream) = TcpStream::connect(config::HTTP_SOCKET) else {
                return Err("https: failed to create tcp stream".to_string());
            };
            let Ok(tls_stream) = connector.connect(config::HTTP_ENDPOINT, tcp_stream) else {
                return Err("https: failed to create tls stream".to_string());
            };
            tls_stream
        };

        let Ok(()) = stream.write_all(request) else {
            return Err("https: failed to write request".to_string());
        };
        let Ok(()) = stream.flush() else {
            return Err("https: failed to flush stream".to_string());
        };

        let mut response = vec![];
        let Ok(_response_length) = stream.read_to_end(&mut response) else {
            return Err("https: failed to read response".to_string());
        };
        std::mem::drop(stream);

        let Ok(response) = String::from_utf8(response) else {
            return Err("https: response is not valid utf8".to_string());
        };

        // wait and retry if the rate limit has been hit.
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
        return Ok(());
    }
}

pub fn post_message(channel_id: &str, content: &str) -> Result<(), String> {
    let channel = sanitize(channel_id);
    let content = sanitize(content);

    let body = format!(r#"{{"content":"{}","embeds":[]}}"#, content);
    let request = format!("POST /channels/{}/messages HTTP/1.0\r\nHost: {}\r\nContent-Length: {}\r\nX-Bot-Token: {}\r\n\r\n{}", channel, config::HTTP_ENDPOINT, body.len(), config::BOT_TOKEN, body);

    send(request.as_bytes())
}

pub fn post_alarm(alarm: &Alarm) -> Result<(), String> {
    let message = sanitize(&alarm.what);
    let channel = sanitize(&alarm.channel_id);
    let reply_to = sanitize(&alarm.message_id);

    let body = format!(r#"{{"content":"{}","replies":[{{"id":"{}","mention":true,"fail_if_not_exists":false}}]}}"#, message, reply_to);
    let request = format!("POST /channels/{}/messages HTTP/1.0\r\nHost: {}\r\nContent-Length: {}\r\nX-Bot-Token: {}\r\n\r\n{}", channel, config::HTTP_ENDPOINT, body.len(), config::BOT_TOKEN, body);

    send(request.as_bytes())
}

pub fn react(channel: &str, message: &str, emoji: &str) -> Result<(), String> {
    let request = format!("PUT /channels/{}/messages/{}/reactions/{} HTTP/1.0\r\nHost: {}\r\nX-Bot-Token: {}\r\nContent-Length: 0\r\n\r\n", channel, message, emoji, config::HTTP_ENDPOINT, config::BOT_TOKEN);

    send(request.as_bytes())
}
