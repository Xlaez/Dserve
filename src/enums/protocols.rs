#[derive(Debug, PartialEq)]
pub enum ProtocolState {
    Idle,
    Connecting,
    Connected,
    Disconnecting,
}
