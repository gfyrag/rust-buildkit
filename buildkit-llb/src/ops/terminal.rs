use std::io::{self, Write};

use buildkit_proto::pb::{self, Input};
use prost::Message;

use crate::serialization::{Output, SerializedNode};
use crate::utils::OperationOutput;

pub struct Terminal<'a> {
    input: OperationOutput<'a>,
}

impl<'a> Terminal<'a> {
    pub fn with(input: OperationOutput<'a>) -> Self {
        Self { input }
    }

    pub fn into_definition(self) -> pb::Definition {
        let (def, metadata) = {
            self.serialize()
                .unwrap()
                .into_iter()
                .map(|item| (item.bytes, (item.digest, item.metadata)))
                .unzip()
        };

        pb::Definition { def, metadata }
    }

    pub fn write_definition(self, mut writer: impl Write) -> io::Result<()> {
        let mut bytes = Vec::new();
        self.into_definition().encode(&mut bytes).unwrap();

        writer.write_all(&bytes)
    }

    fn serialize(&self) -> Result<Output, ()> {
        let serialized_input = self.input.0.serialize()?;

        let head = pb::Op {
            inputs: vec![Input {
                digest: serialized_input.head.digest.clone(),
                index: self.input.1.into(),
            }],

            ..Default::default()
        };

        Ok(Output {
            head: SerializedNode::new(head, Default::default()),
            tail: serialized_input.into_iter().collect(),
        })
    }
}