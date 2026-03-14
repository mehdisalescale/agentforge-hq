//! Process spawning for Claude Code CLI and stream-json parsing.
//! Agent B: runner emits process output as ForgeEvent to EventBus.

pub mod best_of_n;
pub mod concurrent;
pub mod context_pruner;
pub mod loop_detect;
pub mod parse;
pub mod pipeline;
pub mod runner;
pub mod spawn;
pub mod stream_event;
pub mod task_type;

pub use best_of_n::{BestOfNRunner, SelectionResult, select_best};
pub use concurrent::{ConcurrentRunner, SubTask, SubTaskResult, aggregate_results};
pub use loop_detect::{ExitGateConfig, LoopDetector, check_completion_patterns, validate_exit};
pub use parse::{parse_line, ParseError};
pub use pipeline::{Pipeline, PipelineRunner, PipelineStep, StepResult};
pub use runner::{ProcessRunner, StreamJsonEvent as RunnerStubEvent, StreamJsonKind};
pub use spawn::{ProcessHandle, SpawnConfig, SpawnError, spawn};
pub use stream_event::{
    AssistantPayload, ContentBlock, ErrorPayload, MessagePayload, ResultPayload,
    StreamJsonEvent, SystemPayload, UserPayload,
};
