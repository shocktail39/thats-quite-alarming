pub const BOT_ID: &str = "put your bot's user id here";
pub const BOT_TOKEN: &str = "put your bot's secret token here";

// if you're self-hosting your own stoat server,
// then change the following to match your stoat server.
pub const EVENT_SOCKET: (&str, u16) = ("events.stoat.chat", 443);
pub const EVENT_ENDPOINT: &str = "wss://events.stoat.chat/";
pub const HTTP_SOCKET: (&str, u16) = ("api.stoat.chat", 443);
pub const HTTP_ENDPOINT: &str = "api.stoat.chat";
