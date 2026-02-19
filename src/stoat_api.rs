use crate::alarm::Alarm;

fn sanitize(input: String) -> String {
    input.replace("\\", "\\\\").replace("\"", "\\\"")
}

pub fn into_post_message(alarm: Alarm, bot_token: &str) -> String {
    let message = sanitize(alarm.what);
    let channel = sanitize(alarm.channel_id);
    let reply_to = sanitize(alarm.message_id);

    let body = format!(r#"{{"content":"{}","replies":[{{"id":"{}","mention":true}}]}}"#, message, reply_to);
    let request = format!("POST /channels/{}/messages HTTP/1.0\r\nHost: api.stoat.chat\r\nContent-Length: {}\r\nX-Bot-Token: {}\r\n\r\n{}", channel, body.len(), bot_token, body);

    request
}
