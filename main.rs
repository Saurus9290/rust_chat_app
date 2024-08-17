use std::io::{self, ErrorKind, Read, Write}; // Import I/O-related functionality
use std::net::TcpStream; // Import the TcpStream for TCP communication
use std::sync::mpsc::{self, TryRecvError}; // Import channels for multi-threaded communication
use std::thread; // Import thread management
use std::time::Duration; // Import time-related functionalities

// Define constants for the local server address and message size
const LOCAL: &str = "127.0.0.1:6000"; // Local server address
const MSG_SIZE: usize = 32; // Fixed message size in bytes

fn main() {
    // Attempt to connect to the server
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    // Set the stream to non-blocking mode
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    // Create a channel for communication between threads
    let (tx, rx) = mpsc::channel::<String>();

    // Spawn a new thread to handle incoming and outgoing messages
    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE]; // Buffer for receiving messages

        // Attempt to read from the server
        match client.read_exact(&mut buff) {
            Ok(_) => {
                // Remove trailing null bytes and print the received message
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("message recv {:?}", msg);
            },
            // If the socket is not ready for reading, continue the loop
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            // If there is another error, assume the connection was severed
            Err(_) => {
                println!("connection with server was severed");
                break;
            }
        }

        // Try to receive a message from the main thread to send to the server
        match rx.try_recv() {
            Ok(msg) => {
                // Convert the message to bytes and ensure it's of fixed size
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0); // Pad the message with null bytes if necessary
                // Send the message to the server
                client.write_all(&buff).expect("writing to socket failed");
                println!("message sent {:?}", msg);
            }, 
            // If there are no messages to send, continue the loop
            Err(TryRecvError::Empty) => (),
            // If the channel is disconnected, exit the loop
            Err(TryRecvError::Disconnected) => break
        }

        // Sleep for a short duration to prevent busy-waiting
        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a Message:");
    // Main thread loop to capture user input
    loop {
        let mut buff = String::new(); // Buffer to store user input
        // Read input from the standard input
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        // Trim the input and convert it to a string
        let msg = buff.trim().to_string();
        // If the user types ":quit" or if sending fails, break the loop
        if msg == ":quit" || tx.send(msg).is_err() {break}
    }
    println!("bye bye!");
}
