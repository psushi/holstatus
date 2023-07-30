use std::io::{BufRead, StdoutLock, Write};

use anyhow::{bail, Context};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    pub dest: String,
    pub body: MessageBody<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBody<Payload> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

pub trait Node<Payload>
where
    Payload: Serialize,
{
    fn process(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
    fn reply(
        &mut self,
        reply: Payload,
        input: &Message<Payload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let reply = Message {
            src: input.dest.to_owned(),
            dest: input.src.to_owned(),
            body: MessageBody {
                msg_id: Some(self.update_msg_id()),
                in_reply_to: input.body.msg_id,
                payload: reply,
            },
        };
        serde_json::to_writer(&mut *output, &reply)
            .context("serialize response failed")
            .unwrap();
        output.write_all(b"\n").unwrap();
        Ok(())
    }
    fn set_node_id(&mut self, node_id: String);
    fn update_msg_id(&mut self) -> usize;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

pub fn main_loop<S, Payload>(mut state: S) -> anyhow::Result<()>
where
    Payload: Serialize + DeserializeOwned,
    S: Node<Payload>,
{
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut init_msg_str = String::new();
    stdin
        .read_line(&mut init_msg_str)
        .context("Must get init message")?;

    let init_msg = serde_json::from_str::<Message<InitPayload>>(&init_msg_str)
        .context("Failed to deserialize init message")?;

    match init_msg.body.payload {
        InitPayload::Init { node_id, .. } => {
            state.set_node_id(node_id);
            let reply = Message {
                src: init_msg.dest,
                dest: init_msg.src,
                body: MessageBody {
                    msg_id: Some(state.update_msg_id()),
                    in_reply_to: init_msg.body.msg_id,
                    payload: InitPayload::InitOk,
                },
            };
            serde_json::to_writer(&mut stdout, &reply)
                .context("serialize response failed")
                .unwrap();
            stdout.write_all(b"\n").unwrap();
        }
        InitPayload::InitOk => bail!("received init_ok"),
    };

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<Payload>>();

    for input in inputs {
        let input = input.context("Maelstrom input fron STDIN cannot be deserialized")?;

        state
            .process(input, &mut stdout)
            .context("Step function failed")?;
    }
    Ok(())
}
