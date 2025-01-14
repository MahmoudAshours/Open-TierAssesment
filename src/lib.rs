pub mod server;
pub mod server_handler; 

pub mod message {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}
