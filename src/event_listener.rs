use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use tungstenite::ClientRequestBuilder;
use tungstenite::protocol::Message;
use tungstenite::protocol::WebSocket;

use crate::alarm::Alarm;
use crate::alarm_heap::AlarmHeap;
use crate::config;
use crate::json;
use crate::json::Value;
use crate::stoat_api;

struct WebSocketListener<T> {
    stream: WebSocket<T>,
    bot_id: Option<String>,
    alarm_heap: Arc<Mutex<AlarmHeap>>
}

impl<T: Read + Write> WebSocketListener<T> {
    fn handle_ready(&mut self, ready: HashMap<String, Value>) -> bool {
        let Some(Value::Array(users)) = ready.get("users") else {
            return false;
        };
        let Some(Value::Object(user)) = users.get(0) else {
            return false;
        };
        let Some(Value::String(id)) = user.get("_id") else {
            return false;
        };
        self.bot_id = Some(id.into());
        println!("ready!");
        true
    }

    pub fn listen(mut self) {
        loop {
            if let Ok(message) = self.stream.read() {
                println!("{:?}", message);
                let Ok((Value::Object(message), _)) = json::parse_value(&message.into_data(), 0) else {
                    println!("you suck at this, steve.");
                    continue;
                };
                let Some(Value::String(msg_type)) = message.get("type") else {
                    println!("no message type");
                    continue;
                };
                match msg_type.as_str() {
                    "Ready" => {
                        self.handle_ready(message);
                    },
                    "Message" => {
                        if
                            let Some(Value::Array(mentions)) = message.get("mentions")
                            && let Some(ref bot_id) = self.bot_id 
                            && mentions.contains(&Value::String(bot_id.to_string()))
                        {
                            if let Some(alarm) = Alarm::from_message(message) {
                                stoat_api::react(&alarm.channel_id, &alarm.message_id, "👌");
                                self.alarm_heap.lock().unwrap().push(alarm);
                            }
                        }
                    },
                    _ => {}
                }
            }
            thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

pub fn start_listening(alarm_heap: Arc<Mutex<AlarmHeap>>) -> JoinHandle<()> {
    let auth_request = Message::Text(format!(r#"{{"type":"Authenticate","token":"{}"}}"#, config::BOT_TOKEN).into());
    let request = ClientRequestBuilder::new(config::EVENT_ENDPOINT.parse().unwrap());
    let stream = TcpStream::connect(config::EVENT_SOCKET).unwrap();
    let (mut websocket_client, _response) = tungstenite::client_tls(request, stream).unwrap();
    websocket_client.write(auth_request).unwrap();
    websocket_client.flush().unwrap();
    thread::spawn(|| {
        WebSocketListener::listen(WebSocketListener{stream: websocket_client, bot_id: None, alarm_heap});
    })
}
