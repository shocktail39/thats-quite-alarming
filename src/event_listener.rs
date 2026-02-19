use std::net::TcpStream;
use tungstenite::ClientRequestBuilder;
use tungstenite::Message;

pub fn learn_how_to_websocket() {
    const DOMAIN: &str = "events.stoat.chat";
    const EVENT_ENDPOINT: &str = "wss://events.stoat.chat/";
    let request = ClientRequestBuilder::new(EVENT_ENDPOINT.parse().unwrap());
    let stream = TcpStream::connect((DOMAIN, 443)).unwrap();
    let (mut websocket_client, response) = tungstenite::client_tls(request, stream).unwrap();
    websocket_client.write(Message::Text(r#"{"type":"Authenticate","token":"ItWouldBeCrazyIfThisTokenExists"}"#.into()));
    websocket_client.flush();
    println!("{:?}", websocket_client.read());
}
