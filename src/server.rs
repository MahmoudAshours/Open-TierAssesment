use log::{info, warn};
use std::{
    io::{self},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::server_handler::ServerHandler;

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(AtomicBool::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);

        let client_id = Arc::new(Mutex::new(0));

        // Set the listener to non-blocking mode
        self.listener.set_nonblocking(true)?;

        for stream in self.listener.incoming() {
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }
            match stream {
                Ok(stream) => {
                    let id = {
                        let mut id_lock = client_id.lock().unwrap();
                        *id_lock += 1;
                        *id_lock
                    };
                    let is_running = self.is_running.clone(); // Pass running state to thread

                    // Spawn a thread for each client
                    thread::spawn(move || {
                        let mut server_handler: ServerHandler = ServerHandler::new(stream);
                        if let Err(e) = server_handler.handle(id) {
                            eprintln!("Error handling client {}: {}", id, e);
                        }

                        // Check if the server is still running after handling the client
                        if !is_running.load(Ordering::SeqCst) {
                            println!("Server shutting down gracefully.");
                        }
                    });
                }

                Err(_e) => {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
        Ok(())
    }

    /// Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
