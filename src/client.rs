use dserve::definitions::def;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut client = def::NetworkProtocol::new("127.0.0.1:3801")?;

    println!("Client started on 127.0.0.1:3801");

    client.connect("127.0.0.1:3801")?;

    println!("Attempting to connect to server...");

    loop {
        client.update()?;
        // Using 60fps update rate
        std::thread::sleep(Duration::from_millis(16));
    }
}
