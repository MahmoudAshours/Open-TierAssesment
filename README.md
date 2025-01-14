# Solution

## New Server architecture

The server works in a multi-threaded environment, where each client connection is handled in its own thread. The server reads the incoming data in a non-blocking manner, validates the message length, decodes the message, and sends back the appropriate response. In case of inactivity or unexpected disconnection, the server handles the scenario gracefully by disconnecting the client and releasing resources.

**Some issues were fixed and other features were added (as multithreading)**

- Non-Blocking I/O: The server uses ErrorKind::WouldBlock to handle cases when the socket stream is temporarily unavailable, retrying after a short delay.
- Message Validation: Ensures that the message length is within acceptable bounds before attempting to process it.
- Message Decoding and Response: Decodes incoming messages using protobuf and sends back a response after processing.
- Graceful Disconnect: Detects client disconnections and closes the connection without leaving resources open.

## Issues

### Address Already in Use (Error 48)

Multiple test cases were failing with the error:
**Failed to start server: Os { code: 48, kind: AddrInUse, message: "Address already in use" }**
Root Cause : Tests were running concurrently,Each test was trying to use the same port (5000)
Previous test instances weren't properly releasing the port before the next test started

**Solution:**
Added serial test execution using the serial_test crate and added dependency in Cargo.toml, now Test Independence is acheived Each test runs in isolation, No shared state between tests and Proper cleanup after each test.

### Message Decoding and Response Handling

Bug: The server did not handle the message decoding and response properly. If the server received an invalid or malformed protobuf message, it would not respond appropriately.

**Solution:**

Implemented proper message decoding using EchoMessage::decode.If decoding fails, the server logs the error and returns an error response. On successful decoding, the server processes the message and sends an appropriate response back to the client.

### Graceful Client Disconnect

Bug: The server did not handle unexpected client disconnections properly, leading to resource wastage or improper cleanup.

**Solution:**
Used ErrorKind::UnexpectedEof to detect when the client has disconnected. The server now logs the disconnection and exits the connection handling loop gracefully, releasing resources properly.

## Port datastructure representation

Ports in this example is represented as u32 which might be not very optimal

```rust
pub struct TestClient {
    ip: String,
    port: u32,
    timeout: Duration,
    stream: Option<TcpStream>,
}
```

Ports are represented as **u16** (unsigned 16-bit integers) in networking. This means the valid port range is from 0 to 65535.

### Significant changes of code structure

The handle function processes client requests in a loop:

#### Reading Message:

- It starts by reading a message from the client using read_message.
- If the message length is invalid (either 0 or too large), it returns an error.
- The message content is read, and the function tries to decode it using protobuf.

#### Processing Message:

- Depending on the message type (AddRequest or EchoMessage), the server computes the result or echoes the content.
- If the message is unsupported, it returns an error.

#### Sending Response:

- The response is encoded and sent back to the client with a length prefix.

#### Error Handling:

If any error occurs during reading, processing, or sending a response, the function handles it appropriately, logging disconnections and retrying where necessary.

## Best Practices Implemented :

### Error Handling

- Explicit error types
- Proper error propagation
- Meaningful error messages

### Resource Management

- Proper connection cleanup
- Server shutdown between tests
- Port management

### Message Processing

- Validation of message lengths
- Proper message type handling
- Complete response cycle

## New Test cases added

- test_simultaneous_requests : In order to test simultaniously using the Add_request.
- test_add_request_zero_values : Simple test to check zeros addition.
- test_large_echo_message : Simulates how the server handles a large message in the form of an EchoMessage checking if the server can properly handle and echo back a large message (with a content length of 100,000 characters).

### Issues in current setup

The issue in current setup that it fires a thread each time a client connection is initiated, which could be CPU consuming.
In the current setup, if each client connection spawns a new thread to handle communication, it could indeed lead to high CPU usage and resource contention, especially with a large number of clients.

### Possible enhancements:

Instead of spawning a new thread for each client connection, you can consider using more efficient concurrency models like Thread Pooling using a limited number of threads or Event-Driven Frameworks (like tokio) that operates on a single thread for I/O tasks can also reduce the need for multiple threads and enhance scalability.

# Testing

In order to test you could run:

```shell
cargo test
```

### Thank you message

I would like to extend my gratitude for the opportunity to work on this assessment. It has been a challenging and insightful experience, pushing me to improve both my problem-solving and software engineering skills. The complexity of the task allowed me to dive deeper into concepts like client-server communication, concurrency, and error handling, which were both educational and rewarding to tackle.I believe that the current setup could be further enhanced to make it more efficient and scalable, particularly in terms of thread management and error handling.
