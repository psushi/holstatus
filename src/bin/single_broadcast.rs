use std::collections::HashMap;

use holstatus::{main_loop, Node};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct BroadcastNode {
    id: usize,
    node_id: String,
    node_ids: Vec<String>,
    messages: Vec<usize>,
    topology: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Broadcast {
        message: usize,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    BroadcastOk,
    TopologyOk,
}

impl Node<Payload> for BroadcastNode {
    fn process(
        &mut self,
        input: holstatus::Message<Payload>,
        output: &mut std::io::StdoutLock,
    ) -> anyhow::Result<()> {
        match &input.body.payload {
            Payload::Broadcast { message } => {
                self.messages.push(*message);
                self.reply(Payload::BroadcastOk, input, output)?
            }
            Payload::Topology { topology } => {
                self.topology = topology.clone();
                self.reply(Payload::TopologyOk, input, output)?
            }
            Payload::Read => {
                let payload = Payload::ReadOk {
                    messages: self.messages.clone(),
                };
                self.reply(payload, input, output)?
            }
            Payload::BroadcastOk => {}
            Payload::TopologyOk => {}
            Payload::ReadOk { .. } => {}
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
    let node = BroadcastNode::default();
    main_loop(node)
}
