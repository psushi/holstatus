use holstatus::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Generate,

    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

#[derive(Default)]
pub struct GenerateNode {
    id: usize,
    node_id: String,
}

impl Node<Payload> for GenerateNode {
    fn process(
        &mut self,
        input: holstatus::Message<Payload>,
        output: &mut std::io::StdoutLock,
    ) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Generate => {
                let payload = Payload::GenerateOk {
                    guid: format!("{}-{}", self.node_id, self.id),
                };
                self.reply(payload, input, output)?
            }
            Payload::GenerateOk { .. } => {}
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
    let node = GenerateNode::default();
    main_loop(node)
}
