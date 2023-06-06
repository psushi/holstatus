use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
};

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: MessageBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageBody {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Payload {
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

#[derive(Default)]
struct EchoNode {
    id: usize,
    node_id: String,
}

impl EchoNode {
    pub fn handle(&mut self, input: Message, mut output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Init { node_id, .. } => {
                if self.node_id.is_empty() {
                    self.node_id = node_id
                }

                let reply = Message {
                    src: self.node_id.to_owned(),
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::InitOk,
                    },
                };

                self.id += 1;

                serde_json::to_writer(&mut output, &reply).context("serialize response to init")?;
                output.write_all(b"\n")?;
            }
            Payload::Echo { echo } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: MessageBody {
                        msg_id: Some(self.id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::EchoOk { echo },
                    },
                };

                self.id += 1;
                serde_json::to_writer(&mut output, &reply).context("serialize response to echo")?;
                output.write_all(b"\n")?;
            }

            Payload::EchoOk { .. } => {}
            Payload::InitOk => bail!("received init_ok"),
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();
    let mut state = EchoNode::default();
    for input in inputs {
        let input = input.context("Maelstrom input fron STDIN cannot be deserialized")?;

        state
            .handle(input, &mut stdout)
            .context("Step function failed")?;
    }
    Ok(())
}
