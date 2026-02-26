//! Process spawning for Claude Code CLI and stream-json parsing.
//! Agent B: runner emits process output as ForgeEvent to EventBus.

pub mod parse;
pub mod runner;
pub mod spawn;
pub mod stream_event;

pub use parse::{parse_line, ParseError};
pub use runner::{ProcessRunner, StreamJsonEvent as RunnerStubEvent, StreamJsonKind};
pub use spawn::{ProcessHandle, SpawnConfig, SpawnError, spawn};
pub use stream_event::{
    AssistantPayload, ContentBlock, ErrorPayload, MessagePayload, ResultPayload,
    StreamJsonEvent, SystemPayload, UserPayload,
};
