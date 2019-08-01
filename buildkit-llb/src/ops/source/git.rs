use std::collections::HashMap;
use std::sync::Arc;

use buildkit_proto::pb::{self, op::Op, OpMetadata, SourceOp};

use crate::ops::{OperationBuilder, SingleBorrowedOutputOperation, SingleOwnedOutputOperation};
use crate::serialization::{Context, Node, Operation, OperationId, Result};
use crate::utils::{OperationOutput, OutputIdx};

#[derive(Default, Debug)]
pub struct GitSource {
    id: OperationId,
    url: String,
    description: HashMap<String, String>,
    ignore_cache: bool,
}

impl GitSource {
    pub(crate) fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: OperationId::default(),
            url: url.into(),
            description: Default::default(),
            ignore_cache: false,
        }
    }
}

impl<'a> SingleBorrowedOutputOperation<'a> for GitSource {
    fn output(&'a self) -> OperationOutput<'a> {
        OperationOutput::borrowed(self, OutputIdx(0))
    }
}

impl<'a> SingleOwnedOutputOperation<'static> for Arc<GitSource> {
    fn output(&self) -> OperationOutput<'static> {
        OperationOutput::owned(self.clone(), OutputIdx(0))
    }
}

impl OperationBuilder<'static> for GitSource {
    fn custom_name<S>(mut self, name: S) -> Self
    where
        S: Into<String>,
    {
        self.description
            .insert("llb.customname".into(), name.into());

        self
    }

    fn ignore_cache(mut self, ignore: bool) -> Self {
        self.ignore_cache = ignore;
        self
    }
}

impl Operation for GitSource {
    fn id(&self) -> &OperationId {
        &self.id
    }

    fn serialize(&self, _: &mut Context) -> Result<Node> {
        let head = pb::Op {
            op: Some(Op::Source(SourceOp {
                identifier: format!("git://{}", self.url),
                attrs: Default::default(),
            })),

            ..Default::default()
        };

        let metadata = OpMetadata {
            description: self.description.clone(),
            ignore_cache: self.ignore_cache,

            ..Default::default()
        };

        Ok(Node::new(head, metadata))
    }
}

#[test]
fn serialization() {
    crate::check_op!(
        GitSource::new("any.url"),
        |digest| { "sha256:ecde982e19ace932e5474e57b0ca71ba690ed7d28abff2a033e8f969e22bf2d8" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "git://any.url".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        GitSource::new("any.url").custom_name("git custom name"),
        |digest| { "sha256:ecde982e19ace932e5474e57b0ca71ba690ed7d28abff2a033e8f969e22bf2d8" },
        |description| { vec![("llb.customname", "git custom name")] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "git://any.url".into(),
                attrs: Default::default(),
            })
        },
    );
}