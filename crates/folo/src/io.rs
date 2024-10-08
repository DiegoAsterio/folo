mod completion_port;
mod completion_port_shared;
mod driver;
mod driver_shared;
mod error;
mod operation;
mod operation_shared;
mod operation_result;
mod operation_result_shared;
mod pinned_buffer;
mod pinned_buffer_shared;
mod primitive;
mod waker;

pub(crate) use completion_port::*;
pub(crate) use completion_port_shared::*;
pub(crate) use driver::*;
pub(crate) use driver_shared::*;
pub use error::*;
pub(crate) use operation::*;
pub use operation_result::*;
pub use operation_result_shared::*;
pub use pinned_buffer::*;
pub use pinned_buffer_shared::*;
pub(crate) use primitive::*;
pub(crate) use waker::*;

/// Max number of I/O operations to dequeue in one go. Presumably getting more data from the OS with
/// a single call is desirable but the exact impact of different values on performance is not known.
///
/// Known aspects of performance impact:
/// * GetQueuedCompletionStatusEx duration seems linearly affected under non-concurrent synthetic
///   message load (e.g. 40 us for 1024 items).
pub const IO_DEQUEUE_BATCH_SIZE: usize = 1024;
