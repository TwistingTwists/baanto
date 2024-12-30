use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MessageBody {
    #[serde(rename = "init")]
    Init(InitMessage),

    #[serde(rename = "topology")]
    Topology(TopologyMessage),

    #[serde(rename = "error")]
    Error(ErrorMessage),

    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitMessage {
    msg_id: u32,
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopologyMessage {
    msg_id: u32,
    topology: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMessage {
    in_reply_to: u32,
    // #[serde(flatten)]
    code: MaelstromError,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum MaelstromError {
    Timeout = 0,
    NodeNotFound = 1,
    NotSupported = 10,
    TemporarilyUnavailable = 11,
    MalformedRequest = 12,
    Crash = 13,
    Abort = 14,
    KeyDoesNotExist = 20,
    KeyAlreadyExists = 21,
    PreconditionFailed = 22,
    TxnConflict = 30,
}

/////////////////////// trait implementations /////////////////////

pub trait MessageResponse {
    type Response: Serialize + Deserialize<'static>;

    fn from_message_body(_body: &MessageBody) -> Option<Self::Response> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopologyOkResponse {
    #[serde(rename = "type")]
    type_field: String,
    msg_id: u32,
    in_reply_to: u32,
}

impl MessageResponse for TopologyMessage {
    type Response = TopologyOkResponse;

    fn from_message_body(body: &MessageBody) -> Option<Self::Response> {
        if let MessageBody::Topology(msg) = body {
            Some(TopologyOkResponse {
                type_field: "topology_ok".to_string(),
                // new message id - unique??
                msg_id: 7878,
                in_reply_to: msg.msg_id,
            })
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitOkResponse {
    #[serde(rename = "type")]
    type_field: String,
    in_reply_to: u32,
}

impl MessageResponse for InitMessage {
    type Response = InitOkResponse;

    fn from_message_body(body: &MessageBody) -> Option<Self::Response> {
        if let MessageBody::Init(msg) = body {
            Some(InitOkResponse {
                type_field: "init_ok".to_string(),
                in_reply_to: msg.msg_id,
            })
        } else {
            None
        }
    }
}

/////////////////////// tests /////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_init() {
        let init_json = r#"
        {
            "type": "init",
            "msg_id": 1,
            "node_id": "n3",
            "node_ids": ["n1", "n2", "n3"]
        }
        "#;

        // let init_message: MessageBody = serde_json::from_str(init_json).unwrap();

        let jd = &mut serde_json::Deserializer::from_str(init_json);
        let init_message: MessageBody = serde_path_to_error::deserialize(jd).unwrap();
        match init_message {
            MessageBody::Init(init) => {
                assert_eq!(init.msg_id, 1);
                assert_eq!(init.node_id, "n3");
                assert_eq!(init.node_ids, vec!["n1", "n2", "n3"]);
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_deserialize_topology() {
        let topology_json = r#"
        {
            "type": "topology",
            "msg_id": 2,
            "topology": ["n1", "n2", "n3", "n4"]
        }
        "#;

        // let topology_message: MessageBody = serde_json::from_str(topology_json).unwrap();

        let jd = &mut serde_json::Deserializer::from_str(topology_json);
        let topology_message: MessageBody = serde_path_to_error::deserialize(jd).unwrap();
        match topology_message {
            MessageBody::Topology(topology) => {
                assert_eq!(topology.msg_id, 2);
                assert_eq!(topology.topology, vec!["n1", "n2", "n3", "n4"]);
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_deserialize_error() {
        let error_json = r#"
        {
            "type": "error",
            "in_reply_to": 5,
            "code": 11,
            "text": "Node n5 is waiting for quorum and cannot service requests yet"
        }
        "#;

        // let error_message: MessageBody = serde_json::from_str(error_json).unwrap();

        let jd = &mut serde_json::Deserializer::from_str(error_json);
        let error_message: MessageBody = serde_path_to_error::deserialize(jd).unwrap();

        match error_message {
            MessageBody::Error(err) => {
                assert_eq!(err.in_reply_to, 5);
                println!("{err:#?}");
                assert!(matches!(err.code, MaelstromError::TemporarilyUnavailable));
                assert_eq!(
                    err.text,
                    "Node n5 is waiting for quorum and cannot service requests yet"
                );
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_deserialize_error_2() {
        let error_json = r#"
        {
            "type": "error",
            "in_reply_to": 3,
            "code": 20,
            "text": "The requested key does not exist"
        }
        "#;

        // let error_message: MessageBody = serde_json::from_str(error_json).unwrap();

        let jd = &mut serde_json::Deserializer::from_str(error_json);
        let error_message: MessageBody = serde_path_to_error::deserialize(jd).unwrap();

        match error_message {
            MessageBody::Error(err) => {
                assert_eq!(err.in_reply_to, 3);
                assert!(matches!(err.code, MaelstromError::KeyDoesNotExist));
                assert_eq!(err.text, "The requested key does not exist");
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_deserialize_unknown() {
        let unknown_json = r#"
        {
            "type": "unknown",
            "data": "some data"
        }
        "#;

        // let unknown_message: MessageBody = serde_json::from_str(unknown_json).unwrap();

        let jd = &mut serde_json::Deserializer::from_str(unknown_json);
        let unknown_message: MessageBody = serde_path_to_error::deserialize(jd).unwrap();

        match unknown_message {
            MessageBody::Unknown => (),
            _ => panic!("Unexpected message type"),
        }
    }
}
