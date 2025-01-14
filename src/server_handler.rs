use crate::message::{
    client_message, server_message, AddRequest, AddResponse, ClientMessage, EchoMessage,
    ServerMessage,
};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::TcpStream,
    thread,
};

pub struct ServerHandler {
    stream: TcpStream,
}

impl ServerHandler {
    pub fn new(stream: TcpStream) -> Self {
        ServerHandler { stream }
    }
    pub fn handle(&mut self, id: usize) -> io::Result<()> {
        println!("Client {} connected", id);

        loop {
            let message = match self.read_message() {
                Ok(msg) => msg,
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    println!("Client {} disconnected.", id);
                    return Ok(());
                }
                Err(e) => return Err(e),
            };

            let response = self.process_message(message)?;
            self.send_response(response)?;
        }
    }
    fn read_message(&mut self) -> io::Result<ClientMessage> {
        // Read message length
        let mut length_buf = [0u8; 4];
        self.stream.read_exact(&mut length_buf)?;
        let message_length = u32::from_be_bytes(length_buf) as usize;

        // Validate message length
        if message_length == 0 || message_length > 1024 * 1024 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid message length",
            ));
        }

        // Read message content
        let mut message_buf = vec![0u8; message_length];
        self.stream.read_exact(&mut message_buf)?;

        // Decode protobuf message
        ClientMessage::decode(&message_buf[..]).map_err(|e| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("Failed to decode message: {}", e),
            )
        })
    }

    fn process_message(&self, message: ClientMessage) -> io::Result<ServerMessage> {
        let mut server_message = ServerMessage::default();

        match message.message {
            Some(client_message::Message::AddRequest(add_request)) => {
                let mut response = AddResponse::default();
                response.result = add_request.a + add_request.b;

                let mut server_message = ServerMessage::default();
                server_message.message = Some(server_message::Message::AddResponse(response));
                Ok(server_message)
            }
            Some(client_message::Message::EchoMessage(echo_request)) => {
                let mut response = EchoMessage::default();
                response.content = echo_request.content;
                server_message.message = Some(server_message::Message::EchoMessage(response));
                Ok(server_message)
            }
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Unsupported message type",
            )),
        }
    }

    fn send_response(&mut self, response: ServerMessage) -> io::Result<()> {
        let mut response_buf = Vec::new();
        response.encode(&mut response_buf).map_err(|e| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("Failed to encode response: {}", e),
            )
        })?;

        // Write length prefix
        let length = response_buf.len() as u32;
        self.stream.write_all(&length.to_be_bytes())?;

        // Write message
        self.stream.write_all(&response_buf)?;
        self.stream.flush()?;

        Ok(())
    }
    // Handle AddRequest and respond with AddResponse
    pub fn handle_add_request(&self, add_request: AddRequest) -> AddResponse {
        // Calculate the sum of a and b from the AddRequest
        let result = add_request.a + add_request.b;

        // Create and return the AddResponse
        AddResponse { result }
    }
}
