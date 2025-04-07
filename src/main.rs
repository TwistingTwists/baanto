mod message;

use message::{EchoMessage, InitMessage, MessageBody, MessageResponse, TopologyMessage};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new node with a default ID (will be updated during init)
    let node = Node::new("default".to_string());

    // Process messages continuously, one line at a time
    loop {
        match read_line_from_stdin()? {
            Some(line) => {
                // Parse the JSON message
                let message: Result<Message, _> = serde_json::from_str(&line);
                match message {
                    Ok(msg) => {
                        // Handle the message using our node
                        if let Err(e) = node.handle_message(msg) {
                            eprintln!("Error handling message: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error parsing message: {}", e);
                    }
                }
            }
            None => {
                // End of input (stdin closed)
                break;
            }
        }
    }

    Ok(())
}

/// Represents a response to a message
#[derive(Debug, Serialize)]
struct ResponseMessage {
    src: String,
    dest: String,
    // #[serde(flatten)]
    body: serde_json::Value,
    // body: MessageBody,
}

#[derive(Debug)]
pub struct Node {
    node_id: String,
}

impl Node {
    pub fn new(node_id: String) -> Self {
        Node { node_id }
    }

    pub fn handle_message(&self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        let response_body = match &msg.body {
            MessageBody::Init(init_msg) => {
                let response = InitMessage::from_message_body(&msg.body, init_msg.msg_id)
                    .ok_or_else(|| "Failed to create init response")?;
                serde_json::to_value(response)?
            }
            MessageBody::Echo(echo_msg) => {
                let response = EchoMessage::from_message_body(&msg.body, echo_msg.msg_id)
                    .ok_or_else(|| "Failed to create echo response")?;
                serde_json::to_value(response)?
            }
            MessageBody::Topology(topology_msg) => {
                let response = TopologyMessage::from_message_body(&msg.body, topology_msg.msg_id)
                    .ok_or_else(|| "Failed to create topology response")?;
                serde_json::to_value(response)?
            }
            MessageBody::Error(_) => {
                // Handle error messages if needed
                return Ok(());
            }
            MessageBody::Unknown => {
                eprintln!("Unknown message type: {:?}", msg);
                return Ok(());
            }
        };

        // Create the full response message
        let response = ResponseMessage {
            src: msg.dest,
            dest: msg.src,
            body: response_body,
        };

        // let body = msg.body.clone();
        // let response = Message {
        //     src: msg.dest,
        //     dest: msg.src,
        //     body,
        // };

        // Serialize and print the response
        let json = serde_json::to_string(&response)?;
        println!("{}", json);

        Ok(())
    }
}

/// Reads a single line from stdin
fn read_line_from_stdin() -> Result<Option<String>, io::Error> {
    let stdin = io::stdin();
    let mut line = String::new();
    let bytes_read = stdin.lock().read_line(&mut line)?;

    if bytes_read == 0 {
        // End of input
        Ok(None)
    } else {
        // Trim newline characters and return
        Ok(Some(line.trim().to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: MessageBody,
}
