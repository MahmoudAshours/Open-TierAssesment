use embedded_recruitment_task::{
    message::{client_message, server_message, AddRequest, EchoMessage},
    server::Server,
};
use serial_test::serial;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

mod test_client;

fn setup_server_thread(server: Arc<Server>) -> JoinHandle<()> {
    thread::spawn(move || {
        server.run().expect("Server encountered an error");
    })
}

fn create_server() -> Arc<Server> {
    Arc::new(Server::new("localhost:5000").expect("Failed to start server"))
}

#[test]
#[serial]
fn test_client_connection() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = test_client::TestClient::new("localhost", 5000, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
#[serial]
fn test_client_echo_message() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = test_client::TestClient::new("localhost", 5000, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the echoed message
    let response = client.receive();
    println!("Received response: {:?}", response);

    assert!(
        response.is_ok(),
        "Failed to receive response for EchoMessage"
    );

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(
                echo.content, echo_message.content,
                "Echoed message content does not match"
            );
        }
        _ => panic!("Expected EchoMessage, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
#[serial]
fn test_multiple_echo_messages() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = test_client::TestClient::new("localhost", 5000, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message);

        // Send the message to the server
        assert!(client.send(message).is_ok(), "Failed to send message");

        // Receive the echoed message
        let response = client.receive();
        assert!(
            response.is_ok(),
            "Failed to receive response for EchoMessage"
        );

        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(
                    echo.content, message_content,
                    "Echoed message content does not match"
                );
            }
            _ => panic!("Expected EchoMessage, but received a different message"),
        }
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
#[serial]
fn test_multiple_clients() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect multiple clients
    let mut clients = vec![
        test_client::TestClient::new("localhost", 5000, 1000),
        test_client::TestClient::new("localhost", 5000, 1000),
        test_client::TestClient::new("localhost", 5000, 1000),
    ];

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages for each client
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        for client in clients.iter_mut() {
            // Send the message to the server
            assert!(
                client.send(message.clone()).is_ok(),
                "Failed to send message"
            );

            // Receive the echoed message
            let response = client.receive();
            assert!(
                response.is_ok(),
                "Failed to receive response for EchoMessage"
            );

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content, message_content,
                        "Echoed message content does not match"
                    );
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            }
        }
    }

    // Disconnect the clients
    for client in clients.iter_mut() {
        assert!(
            client.disconnect().is_ok(),
            "Failed to disconnect from the server"
        );
    }

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
#[serial]
fn test_client_add_request() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = test_client::TestClient::new("localhost", 5000, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 20;
    let message = client_message::Message::AddRequest(add_request.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the response
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for AddRequest"
    );

    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(
                add_response.result,
                add_request.a + add_request.b,
                "AddResponse result does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

// new Test cases

#[test]
#[serial]
fn test_add_request_zero_values() {
    let server = create_server();
    let handle = setup_server_thread(server.clone());
    let mut client = test_client::TestClient::new("localhost", 5000, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the AddRequest message with zero values
    let mut add_request = AddRequest::default();
    add_request.a = 0;
    add_request.b = 0;
    let message = client_message::Message::AddRequest(add_request);

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the response
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for AddRequest"
    );

    // Check the AddResponse
    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(add_response.result, 0, "AddResponse result does not match");
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
#[serial]
fn test_simultaneous_requests() {
    let server = create_server(); // Assuming this initializes the server
    let handle = setup_server_thread(server.clone()); // Assuming this sets up the server thread

    let mut client1 = test_client::TestClient::new("localhost", 5000, 1000);
    let mut client2 = test_client::TestClient::new("localhost", 5000, 1001); // Different client

    assert!(client1.connect().is_ok(), "Failed to connect to the server");
    assert!(client2.connect().is_ok(), "Failed to connect to the server");

    // Prepare AddRequest for both clients
    let mut add_request1 = AddRequest::default();
    add_request1.a = 10;
    add_request1.b = 20;
    let message1 = client_message::Message::AddRequest(add_request1);

    let mut add_request2 = AddRequest::default();
    add_request2.a = 30;
    add_request2.b = 40;
    let message2 = client_message::Message::AddRequest(add_request2);

    // Send the messages to the server
    assert!(
        client1.send(message1).is_ok(),
        "Failed to send message from client1"
    );
    assert!(
        client2.send(message2).is_ok(),
        "Failed to send message from client2"
    );

    // Receive the responses
    let response1 = client1.receive();
    let response2 = client2.receive();

    assert!(
        response1.is_ok() && response2.is_ok(),
        "Failed to receive response for one or both clients"
    );

    // Check the AddResponse for both clients
    match response1.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(
                add_response.result,
                10 + 20,
                "AddResponse result for client1 does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }

    match response2.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(
                add_response.result,
                30 + 40,
                "AddResponse result for client2 does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
#[serial]
fn test_large_echo_message() {
    let server = create_server();
    let handle = setup_server_thread(server.clone());
    let mut client = test_client::TestClient::new("localhost", 5000, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut echo_message = EchoMessage::default();
    echo_message.content = "a".repeat(100_000); // Large but within limits
    let message = client_message::Message::EchoMessage(echo_message.clone());

    assert!(client.send(message).is_ok());
    let response = client.receive();
    assert!(response.is_ok());

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(echo.content.len(), 100_000);
        }
        _ => panic!("Expected EchoMessage"),
    }
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}
