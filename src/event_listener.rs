use std::net::TcpStream;
use tungstenite::ClientRequestBuilder;
use tungstenite::protocol::Message;

pub fn learn_how_to_websocket(bot_token: &str) {
    const SOCKET: (&str, u16) = ("events.stoat.chat", 443);
    const EVENT_ENDPOINT: &str = "wss://events.stoat.chat/";
    let auth_request = Message::Text(format!(r#"{{"type":"Authenticate","token":"{}"}}"#, bot_token).into());
    let request = ClientRequestBuilder::new(EVENT_ENDPOINT.parse().unwrap());
    let stream = TcpStream::connect(SOCKET).unwrap();
    let (mut websocket_client, _response) = tungstenite::client_tls(request, stream).unwrap();
    websocket_client.write(auth_request).unwrap();
    websocket_client.flush().unwrap();
    while true {
        if let Ok(message) = websocket_client.read() {
            println!("{:?}", message);
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
