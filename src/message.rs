use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MessageBody {
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: usize,
    },
    Echo {
        msg_id: usize,
        echo: String,
    },
    EchoOk {
        msg_id: usize,
        in_reply_to: usize,
        echo: String,
    },
    Generate {
        msg_id: usize,
    },
    GenerateOk {
        msg_id: usize,
        in_reply_to: usize,
        id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: MessageBody,
}

impl Message {
    pub fn get_node_id(&self) -> Option<&str> {
        match &self.body {
            MessageBody::Init { node_id, .. } => Some(node_id),
            _ => None,
        }
    }

    pub fn get_response(&self, node_id: &str, id: usize) -> Self {
        match &self.body {
            MessageBody::Init { msg_id, .. } => Message {
                src: node_id.to_string(),
                dst: self.src.to_string(),
                body: MessageBody::InitOk {
                    in_reply_to: *msg_id,
                },
            },
            MessageBody::Echo { msg_id, echo } => Message {
                src: node_id.to_string(),
                dst: self.src.to_string(),
                body: MessageBody::EchoOk {
                    msg_id: *msg_id,
                    in_reply_to: *msg_id,
                    echo: echo.to_string(),
                },
            },
            MessageBody::Generate { msg_id } => Message {
                src: node_id.to_string(),
                dst: self.src.to_string(),
                body: MessageBody::GenerateOk {
                    msg_id: *msg_id,
                    in_reply_to: *msg_id,
                    id: format!("{}-{}", node_id, id),
                },
            },
            _ => panic!("Received unexpected message type"),
        }
    }
}
