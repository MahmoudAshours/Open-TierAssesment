use embedded_recruitment_task::message::{client_message, ServerMessage};
use log::error;
use log::info;
use prost::Message;
use std::io::Read;
use std::io::Write;
use std::{
    io,
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

// TCP/IP Client
pub struct TestClient {
    ip: String,
    port: u16,
    timeout: Duration,
    stream: Option<TcpStream>,
}

impl TestClient {
    pub fn new(ip: &str, port: u16, timeout_ms: u64) -> Self {
        TestClient {
            ip: ip.to_string(),
            port,
            timeout: Duration::from_millis(timeout_ms),
            stream: None,
        }
    }

    // connect the client to the server
    pub fn connect(&mut self) -> io::Result<()> {
        println!("Connecting to {}:{}", self.ip, self.port);

        // Resolve the address
        let address = format!("{}:{}", self.ip, self.port);
        let socket_addrs: Vec<SocketAddr> = address.to_socket_addrs()?.collect();

        if socket_addrs.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid IP or port",
            ));
        }

        // Connect to the server with a timeout
        let stream = TcpStream::connect_timeout(&socket_addrs[0], self.timeout)?;
        self.stream = Some(stream);

        println!("Connected to the server!");
        Ok(())
    }

    // disconnect the client
    pub fn disconnect(&mut self) -> io::Result<()> {
        if let Some(stream) = self.stream.take() {
            stream.shutdown(std::net::Shutdown::Both)?;
        }

        println!("Disconnected from the server!");
        Ok(())
    }

    pub fn send(&mut self, message: client_message::Message) -> io::Result<()> {
        if let Some(ref mut stream) = self.stream {
            // Encode the message to a buffer
            let mut message_buf = Vec::new();
            message.encode(&mut message_buf);
            // Write length prefix (4 bytes, big-endian)
            let length = message_buf.len() as u32;

            stream.write_all(&length.to_be_bytes())?;
            // Write the message
            stream.write_all(&message_buf)?;
            stream.flush()?;

            println!("Sent message: {:?}", message);
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }

    pub fn receive(&mut self) -> io::Result<ServerMessage> {
        if let Some(ref mut stream) = self.stream {
            println!("Receiving message from the server");

            // Read message length (4 bytes)
            let mut length_buf = [0u8; 4];
            stream.read_exact(&mut length_buf)?;

            let message_length = u32::from_be_bytes(length_buf) as usize;

            println!("Length: {}", message_length.to_string());

            // Sanity check the message length
            if message_length == 0 || message_length > 1024 * 1024 {
                // 1MB limit
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid message length received",
                ));
            }

            // Read the exact message bytes
            let mut message_buf = vec![0u8; message_length];

            stream.read_exact(&mut message_buf)?;
            println!("Message buffer :{:?}", message_buf);

            info!("Received {} bytes from the server", message_length);

            // Decode the received message
            ServerMessage::decode(&message_buf[..]).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to decode ServerMessage: {}", e),
                )
            })
        } else {
            error!("No active connection");
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }
}
