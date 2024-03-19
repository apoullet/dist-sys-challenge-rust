use std::io::{self, BufRead, StdoutLock, Write};

use anyhow::{bail, Context};

pub mod message;

fn main() -> anyhow::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut buffer = String::new();
    let _ = stdin.read_line(&mut buffer);

    let init_req: message::Message =
        serde_json::from_str(&buffer).context("Error while deserializing init request")?;

    let node_id = match init_req.get_node_id() {
        Some(string) => string,
        None => bail!("Failed to extract node_id from init request"),
    };

    let init_res = init_req.get_response(node_id, 0);
    let _ = reply(&mut stdout, &init_res)?;

    for count in 1.. {
        buffer.clear();
        let _ = stdin.read_line(&mut buffer);

        let req: message::Message = serde_json::from_str(&buffer)?;

        let res = req.get_response(node_id, count);
        let _ = reply(&mut stdout, &res)?;
    }

    Ok(())
}

fn reply(stdout: &mut StdoutLock<'_>, response: &message::Message) -> anyhow::Result<()> {
    let _ = stdout.write_all(serde_json::to_string(response)?.as_bytes());
    let _ = stdout.write_all("\n".as_bytes());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::message;

    #[test]
    fn it_correctly_replies_to_init_request() -> anyhow::Result<()> {
        let req = message::Message {
            src: "c1".to_string(),
            dst: "n3".to_string(),
            body: message::MessageBody::Init {
                msg_id: 1,
                node_id: "n3".to_string(),
                node_ids: Vec::from(["n1".to_string(), "n2".to_string(), "n3".to_string()]),
            },
        };
        let res = req.get_response("n3", 0);
        let reply = serde_json::to_string(&res)?;
        assert_eq!(
            reply,
            r#"{"src":"n3","dest":"c1","body":{"type":"init_ok","in_reply_to":1}}"#
        );
        Ok(())
    }
    #[test]
    fn it_correctly_replies_to_echo_request() -> anyhow::Result<()> {
        let req = message::Message {
            src: "c1".to_string(),
            dst: "n3".to_string(),
            body: message::MessageBody::Echo {
                msg_id: 1,
                echo: "Please echo 98".to_string(),
            },
        };
        let res = req.get_response("n3", 0);
        let reply = serde_json::to_string(&res)?;
        assert_eq!(
            reply,
            r#"{"src":"n3","dest":"c1","body":{"type":"echo_ok","msg_id":1,"in_reply_to":1,"echo":"Please echo 98"}}"#
        );
        Ok(())
    }
    #[test]
    fn it_correctly_replies_to_generate_request() -> anyhow::Result<()> {
        let req = message::Message {
            src: "c1".to_string(),
            dst: "n3".to_string(),
            body: message::MessageBody::Generate { msg_id: 1 },
        };
        let res = req.get_response("n3", 0);
        let reply = serde_json::to_string(&res)?;
        assert_eq!(
            reply,
            r#"{"src":"n3","dest":"c1","body":{"type":"generate_ok","msg_id":1,"in_reply_to":1,"id":"n3-0"}}"#
        );
        Ok(())
    }
}
