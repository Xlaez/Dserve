// mod client;
// mod definitions;
// mod enums;
// mod implementations;

use std::time::Duration;

use dserve::definitions::def;

fn main() -> std::io::Result<()> {
    let mut protocol = def::NetworkProtocol::new("127.0.0.1:3800")?;

    println!("Server started on 127.0.0.1:3800");

    loop {
        protocol.update()?;
        // Using 60fps update rate
        std::thread::sleep(Duration::from_millis(16));
    }
}
