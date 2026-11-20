use terobit_v1_abi::commands::Opcode;

/// A change to the scene that is written into the command buffer and
/// applied by the host at the end of the cycle, or on [`Commands::flush`](super::Commands::flush).
pub trait Command {
    /// Tells the host which command this record is.
    const OPCODE: Opcode;

    /// Writes this command's payload. The opcode and length header are
    /// written by the buffer.
    fn encode(&self, payload: &mut Vec<u8>);
}
