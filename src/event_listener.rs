use std::collections::HashMap;
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
use crate::file;
use crate::json;
use crate::json::Value;
use crate::stoat_api;

fn authenticate(stream: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Result<(), String> {
    let auth_request = Message::Text(format!(r#"{{"type":"Authenticate","token":"{}"}}"#, config::BOT_TOKEN).into());
    let Ok(()) = stream.send(auth_request) else {
        return Err("failed to send auth request to event websocket".to_string());
    };

    let Ok(ws_response) = stream.read() else {
        return Err("failed to read auth response from event websocket".to_string());
    };
    let Ok((Value::Object(mut response), _)) = json::parse_value(&ws_response.into_data(), 0) else {
        return Err("auth response from event websocket is invalid".to_string());
    };
    let Some(Value::String(msg_type)) = response.remove("type") else {
        return Err("auth response from event websocket is invalid".to_string());
    };
    if msg_type.as_str() != "Authenticated" {
        return Err("authentication failed.  make sure BOT_TOKEN in config.rs is set correctly".to_string());
    }
    Ok(())
}

fn wait_for_ready(stream: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Result<(), String> {
    let Ok(response) = stream.read() else {
        return Err("failed to read ready response from event websocket".to_string());
    };
    let Ok((Value::Object(mut response), _)) = json::parse_value(&response.into_data(), 0) else {
        return Err("ready response from event websocket is invalid".to_string());
    };
    let Some(Value::String(msg_type)) = response.remove("type") else {
        return Err("ready response from event websocket is invalid".to_string());
    };
    if msg_type.as_str() != "Ready" {
        return Err("ready response from event websocket is invalid".to_string());
    }
    Ok(())
}

fn start_ws_stream() -> Result<WebSocket<MaybeTlsStream<TcpStream>>, String> {
    let mut stream = {
        let Ok(endpoint) = config::EVENT_ENDPOINT.parse() else {
            return Err("make sure EVENT_ENDPOINT in config.rs is a valid url.".to_string());
        };
        let tls_request = ClientRequestBuilder::new(endpoint);
        let Ok(tcp_stream) = TcpStream::connect(config::EVENT_SOCKET) else {
            return Err("failed to start tcp session with event websocket".to_string());
        };
        let Ok((tls_stream, _response)) = tungstenite::client_tls(tls_request, tcp_stream) else {
            return Err("failed to start tls session with event websocket".to_string());
        };
        tls_stream
    };

    authenticate(&mut stream)?;
    wait_for_ready(&mut stream)?;

    Ok(stream)
}

fn handle_message(message: &HashMap<String, Value>, alarm_heap: &Arc<Mutex<AlarmHeap>>) -> Result<(), String> {
    let Some(Value::Array(mentions)) = message.get("mentions") else {
        return Ok(());
    };
    if !mentions.contains(&Value::String(config::BOT_ID.to_string())) {
        return Ok(());
    }

    if let Some(alarm) = Alarm::from_message(message) {
        if let Err(what_happened) = file::save(&alarm) {
            println!("failed to save alarm: {}", what_happened);
        }
        const GREEN_CHECK_BOX: &str = "%E2%9C%85";
        if let Err(what_happened) = stoat_api::react(&alarm.channel_id, &alarm.message_id, GREEN_CHECK_BOX) {
            println!("event listener: {}\nfailed to react {:?}", what_happened, &alarm);
        }
        let Ok(mut heap_lock) = alarm_heap.lock() else {
            return Err("alarm heap mutex has been poisoned.  ending event listener.".to_string());
        };
        heap_lock.push(alarm);
        return Ok(());
    }

    let Some(Value::String(content)) = message.get("content") else {
        return Ok(());
    };
    if
        content.to_lowercase().contains("license")
        && let Some(Value::String(channel_id)) = message.get("channel")
    {
        const AGPL3_MESSAGE: &str = "that's quite alarming is licensed under the gnu affero general public license version 3.  source code can be found at <https://github.com/shocktail39/thats-quite-alarming/>";
        if let Err(what_happened) = stoat_api::post_message(channel_id, AGPL3_MESSAGE) {
            println!("event listener: {}\nfailed to post license to {}", what_happened, channel_id);
        }
    }
    Ok(())
}

fn listen(mut stream: WebSocket<MaybeTlsStream<TcpStream>>, alarm_heap: Arc<Mutex<AlarmHeap>>) -> Result<(), String> {
    loop {
        let Ok(response) = stream.read() else {
            println!("warning: unexpected response from event endpoint");
            continue;
        };
        if let Message::Close(_) = response {
            println!("stream closed by event endpoint");
            return Ok(());
        }
        let Ok((Value::Object(response), _)) = json::parse_value(&response.into_data(), 0) else {
            println!("warning: event endpoint response is unexpectedly not a json object.");
            continue;
        };
        let Some(Value::String(msg_type)) = response.get("type") else {
            println!("warning: no message type");
            continue;
        };
        match msg_type.as_str() {
            "Message" => {
                handle_message(&response, &alarm_heap)?;
            },
            _ => {}
        }
    }
}

pub fn start_listening(alarm_heap: Arc<Mutex<AlarmHeap>>) -> JoinHandle<Result<(), String>> {
    thread::spawn(|| {
        let ws = start_ws_stream()?;
        listen(ws, alarm_heap)?;
        Ok(())
    })
}
