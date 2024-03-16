use std::{
    collections::HashMap,
    io::{self, BufRead, StdoutLock, Write},
};

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageBody {
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    rest: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: MessageBody,
}

fn main() -> anyhow::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut buffer = String::new();
    let _ = stdin.read_line(&mut buffer);

    let init_req: Message =
        serde_json::from_str(&buffer).context("Error while deserializing init request")?;

    assert_eq!(init_req.body.kind, "init");

    let node_id = match extract_node_id(&init_req) {
        Some(string) => string,
        None => bail!("Failed to extract node_id from init request"),
    };

    let init_res = get_init_response(&init_req, node_id);
    let _ = reply(&mut stdout, &init_res)?;

    for count in 1.. {
        buffer.clear();
        let _ = stdin.read_line(&mut buffer);

        let req: Message = serde_json::from_str(&buffer)?;

        let res = match req.body.kind.as_str() {
            "echo" => get_echo_response(&req, node_id),
            "generate" => get_generate_response(&req, node_id, count),
            unknown => bail!(format!("Received unknown message type: {unknown}")),
        };

        let _ = reply(&mut stdout, &res)?;
    }

    Ok(())
}

fn extract_node_id(init_req: &Message) -> Option<&str> {
    let rest = &init_req.body.rest;

    match rest.get("node_id") {
        Some(node_id) => node_id.as_str(),
        None => None,
    }
}

fn get_init_response(request: &Message, node_id: &str) -> Message {
    let Message { src, dst: _, body } = request;
    Message {
        src: node_id.to_string(),
        dst: src.to_string(),
        body: MessageBody {
            kind: "init_ok".to_string(),
            id: None,
            in_reply_to: body.id,
            rest: HashMap::new(),
        },
    }
}

fn get_echo_response(request: &Message, node_id: &str) -> Message {
    let Message { src, dst: _, body } = request;
    Message {
        src: node_id.to_string(),
        dst: src.to_string(),
        body: MessageBody {
            kind: "echo_ok".to_string(),
            id: body.id,
            in_reply_to: body.id,
            rest: body.rest.clone(),
        },
    }
}

fn get_generate_response(request: &Message, node_id: &str, count: usize) -> Message {
    let Message { src, dst: _, body } = request;
    Message {
        src: node_id.to_string(),
        dst: src.to_string(),
        body: MessageBody {
            kind: "generate_ok".to_string(),
            id: body.id,
            in_reply_to: body.id,
            rest: HashMap::from([("id".to_string(), json!(format!("{node_id}-{count}")))]),
        },
    }
}

fn reply(stdout: &mut StdoutLock<'_>, response: &Message) -> anyhow::Result<()> {
    let _ = stdout.write_all(serde_json::to_string(response)?.as_bytes());
    let _ = stdout.write_all("\n".as_bytes());

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use crate::{get_init_response, Message, MessageBody};

    #[test]
    fn it_correctly_replies_to_init_request() -> anyhow::Result<()> {
        let req = Message {
            src: "c1".to_string(),
            dst: "n3".to_string(),
            body: MessageBody {
                kind: "init".to_string(),
                id: Some(1),
                in_reply_to: None,
                rest: HashMap::from([
                    ("node_id".to_string(), json!("n3")),
                    ("node_ids".to_string(), json!(["n1", "n2", "n3"])),
                ]),
            },
        };
        let res = get_init_response(&req, "n3");
        let reply = serde_json::to_string(&res)?;
        assert_eq!(
            reply,
            r#"{"src":"n3","dest":"c1","body":{"type":"init_ok","msg_id":null,"in_reply_to":1}}"#
        );
        Ok(())
    }
}
