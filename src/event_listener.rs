use std::net::TcpStream;
use tungstenite::ClientRequestBuilder;

pub fn learn_how_to_websocket() {
    const SOCKET: (&str, u16) = ("events.stoat.chat", 443);
    const EVENT_ENDPOINT: &str = "wss://events.stoat.chat/";
    const TEST_MESSAGE: &[u8] = br#"{"type":"Authenticate","token":"ItWouldBeCrazyIfThisTokenExists"}"#;
    let request = ClientRequestBuilder::new(EVENT_ENDPOINT.parse().unwrap());
    let stream = TcpStream::connect(SOCKET).unwrap();
    let (mut websocket_client, _response) = tungstenite::client_tls(request, stream).unwrap();
    websocket_client.write(TEST_MESSAGE.into()).unwrap();
    websocket_client.flush().unwrap();
    println!("{:?}", websocket_client.read().unwrap());
}
