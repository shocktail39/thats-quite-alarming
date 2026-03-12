use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

use tungstenite::ClientRequestBuilder;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::protocol::Message;
use tungstenite::protocol::WebSocket;

use crate::alarm::Alarm;
use crate::alarm_heap::AlarmHeap;
use crate::config;
use crate::json;
use crate::json::Value;
use crate::stoat_api;

struct WebSocketListener {
    stream: WebSocket<MaybeTlsStream<TcpStream>>,
    bot_id: String,
    alarm_heap: Arc<Mutex<AlarmHeap>>
}

impl WebSocketListener {
    fn authenticate(stream: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
        let auth_request = Message::Text(format!(r#"{{"type":"Authenticate","token":"{}"}}"#, config::BOT_TOKEN).into());
        stream.send(auth_request).expect("failed to send auth request to event websocket");

        let ws_response = stream.read().expect("failed to read auth response from event websocket");
        let Ok((Value::Object(mut response), _)) = json::parse_value(&ws_response.into_data(), 0) else {
            panic!("auth response from event websocket is invalid");
        };
        let Some(Value::String(msg_type)) = response.remove("type") else {
            panic!("auth response from event websocket is invalid");
        };
        if msg_type.as_str() != "Authenticated" {
            panic!("authentication failed.  make sure BOT_TOKEN in config.rs is set correctly");
        }
    }

    fn handle_ready(response: Message) -> String {
        let Ok((Value::Object(mut response), _)) = json::parse_value(&response.into_data(), 0) else {
            panic!("ready response from event websocket is invalid");
        };
        let Some(Value::String(msg_type)) = response.remove("type") else {
            panic!("ready response from event websocket is invalid");
        };
        if msg_type.as_str() != "Ready" {
            panic!("ready response from event websocket is invalid");
        }
        let Some(Value::Array(mut users)) = response.remove("users") else {
            panic!("invalid ready response from event endpoint");
        };
        let Value::Object(mut user) = users.remove(0) else {
            panic!("invalid ready response from event endpoint");
        };
        let Some(Value::String(id)) = user.remove("_id") else {
            panic!("invalid ready response from event endpoint");
        };
        id
    }

    fn new(alarm_heap: Arc<Mutex<AlarmHeap>>) -> Self {
        let mut stream = {
            let tls_request = ClientRequestBuilder::new(config::EVENT_ENDPOINT.parse().expect("make sure EVENT_ENDPOINT in config.rs is a valid url."));
            let tcp_stream = TcpStream::connect(config::EVENT_SOCKET).expect("failed to start tcp session with event websocket");
            let (tls_stream, _response) = tungstenite::client_tls(tls_request, tcp_stream).expect("failed to start tls session with event websocket");
            tls_stream
        };

        Self::authenticate(&mut stream);

        let ws_response = stream.read().expect("failed to read ready response from event websocket");
        let bot_id = Self::handle_ready(ws_response);

        Self {
            stream,
            bot_id,
            alarm_heap
        }
    }

    fn listen(mut self) {
        loop {
            let Ok(response) = self.stream.read() else {
                println!("unexpected response from event endpoint");
                continue;
            };
            println!("{:?}", response);
            if let Message::Close(_) = response {
                println!("stream closed by event endpoint");
                return;
            }
            let Ok((Value::Object(response), _)) = json::parse_value(&response.into_data(), 0) else {
                println!("event endpoint response is unexpectedly not a json object.");
                continue;
            };
            let Some(Value::String(msg_type)) = response.get("type") else {
                println!("no message type");
                continue;
            };
            match msg_type.as_str() {
                "Message" => {
                    if
                        let Some(Value::Array(mentions)) = response.get("mentions")
                        && mentions.contains(&Value::String(self.bot_id.to_string()))
                    {
                        if let Some(alarm) = Alarm::from_message(response) {
                            stoat_api::react(&alarm.channel_id, &alarm.message_id, "👌");
                            self.alarm_heap.lock().unwrap().push(alarm);
                        }
                    }
                },
                _ => {}
            }
        }
    }
}

pub fn start_listening(alarm_heap: Arc<Mutex<AlarmHeap>>) -> JoinHandle<()> {
    thread::spawn(|| {
        let ws = WebSocketListener::new(alarm_heap);
        ws.listen();
    })
}
