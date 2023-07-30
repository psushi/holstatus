use std::io::StdoutLock;

use holstatus::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
struct EchoNode {
    id: usize,
    node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

impl Node<Payload> for EchoNode {
    fn process(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match &input.body.payload {
            Payload::Echo { echo } => {
                let payload = Payload::EchoOk {
                    echo: echo.to_owned(),
                };
                self.reply(payload, &input, output)?
            }
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }

    fn set_node_id(&mut self, node_id: String) {
        self.node_id = node_id;
    }

    fn update_msg_id(&mut self) -> usize {
        self.id += 1;
        self.id
    }
}

fn main() -> anyhow::Result<()> {
    let state = EchoNode::default();
    main_loop(state)?;
    Ok(())
}
