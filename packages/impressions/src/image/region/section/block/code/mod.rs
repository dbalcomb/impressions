//! The code block.

mod cursor;
mod error;

use serde::{Deserialize, Serialize};

use crate::analysis::Completion;
use crate::instruction::operation::Error as OperationError;
use crate::instruction::{Error as InstructionError, Instruction};
use crate::memory::cursor::AsCursor;
use crate::memory::extent::{Extent, Size};
use crate::memory::inspect::{Inspect, Inspector};
use crate::memory::region::ops::decode::{Decode, Decoder};
use crate::memory::region::ops::encode::{Encode, Encoder, Error as EncodeError};

pub use self::cursor::CodeCursor;
pub use self::error::Error;

/// A block of code.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Code {
    instructions: Vec<Instruction>,
}

impl Extent for Code {
    fn size(&self) -> Size {
        Size::try_sum(self.instructions.iter().map(Extent::size))
            .expect("sum of sizes does not exceed maximum size")
    }
}

impl Completion for Code {
    fn identified(&self) -> u64 {
        self.size().get()
    }
}

impl Inspect for Code {
    fn inspect(&self, inspector: &mut dyn Inspector) {
        inspector
            .record(self.address_space())
            .label(&"Code")
            .finish();
    }
}

impl Encode for Code {
    fn encode(&self, encoder: &mut dyn Encoder) -> Result<(), EncodeError> {
        for instruction in &self.instructions {
            instruction.encode(encoder)?;
        }

        Ok(())
    }
}

impl Decode for Code {
    type Context<'a> = ();
    type Error = Error;

    fn decode_with(decoder: &mut dyn Decoder, _: Self::Context<'_>) -> Result<Self, Self::Error> {
        let mut instructions = Vec::new();

        while decoder.has_remaining() {
            let instruction = match Instruction::decode(decoder) {
                Ok(instruction) => instruction,
                Err(InstructionError::Operation(OperationError::InvalidOpcode(_))) => break,
                Err(err) => return Err(err.into()),
            };

            instructions.push(instruction);
        }

        if instructions.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Self { instructions })
    }
}

impl AsCursor for Code {
    type Cursor<'a> = CodeCursor<'a>;

    fn cursor(&self) -> Self::Cursor<'_> {
        CodeCursor::new(self)
    }
}
