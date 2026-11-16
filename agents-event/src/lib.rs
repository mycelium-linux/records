//! One recorded transcript event: the shape every producer writes and every
//! reader reads.
//!
//! Three programs already speak this schema — colony's `delta-acp` (a Delta
//! thread), edgerunner's records mirror (an ACP session), and `soil`'s
//! `soil-agent-records` (the reader) — and each one carried its own copy of the
//! field names and the ACP→station table. This crate is that one copy.
//!
//! What is *not* here: how a harness classifies its own events, and what an
//! entity id looks like. A producer's ids come from the harness (`msg:<item>`,
//! `evt:<n>`), which is exactly the part that should differ per harness.
//!
//! The record is written through `fungos-records` as `kind: "record.put"` under
//! [`SCHEMA`], in a book per session ([`book_id`]), so one session's transcript
//! pages without scanning the fleet's churn.

use serde_json::{Value, json};

/// The schema id every recorder tags its ops with.
pub const SCHEMA: &str = "fungos.agents/event";
pub const SCHEMA_VERSION: i64 = 1;

/// The kind of op a producer appends.
pub const OP_KIND: &str = "record.put";

/// The book one session's transcript lives in.
pub fn book_id(session: &str) -> String {
    format!("fungos/agents/{session}")
}

/// The `record.put` body for one event.
pub fn put_body(fields: Value) -> Value {
    json!({
        "schema_id": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "fields": fields,
    })
}

/// One line of `records append`: the entry a producer streams.
pub fn put_line(entity_id: &str, fields: Value) -> Value {
    json!({
        "entity_id": entity_id,
        "kind": OP_KIND,
        "body": put_body(fields),
    })
}

/// The field names, so a reader does not match strings a producer renames.
pub mod field {
    pub const SESSION: &str = "session";
    pub const EVENT: &str = "event";
    pub const SEQ: &str = "seq";
    pub const AT_MS: &str = "at_ms";
    pub const HARNESS: &str = "harness";
    pub const RUN: &str = "run";
    pub const TURN: &str = "turn";
    /// The harness's own predecessor id, when it has one.
    pub const AFTER: &str = "after";
    pub const ROLE: &str = "role";
    pub const TEXT: &str = "text";
    pub const AUTHOR: &str = "author";
    pub const ACTOR: &str = "actor";
    pub const NAME: &str = "name";
    pub const ACTIVITY: &str = "activity";
    pub const KIND: &str = "kind";
    pub const LABEL: &str = "label";
    pub const DETAIL: &str = "detail";
    pub const STATUS: &str = "status";
    pub const TOOL: &str = "tool";
    pub const TOOL_CALL: &str = "tool_call";
    /// A turn's liveness: `running`, `finished`, `failed`, `paused`.
    pub const STATE: &str = "state";
    /// A message still growing on the harness side.
    pub const OPEN: &str = "open";
    /// A message the harness has finished writing.
    pub const COMPLETE: &str = "complete";
    /// The harness's own name for a kind this contract does not classify.
    pub const HARNESS_KIND: &str = "harness_kind";
}

/// What one record is, in `soil:agents`' vocabulary rather than any harness's
/// or ACP's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// A finished message: a person's or an agent's.
    Message,
    /// A message still growing; a reader takes `Message` and ignores these.
    Delta,
    Thought,
    /// The seam where a harness elided context.
    Compaction,
    /// One call, at one station.
    Activity,
    /// A edit a person made outside the turn. Not a station the agent visited.
    ExternalEdits,
    /// Something the agent cannot decide alone.
    Prompt,
    PromptAnswered,
    Plan,
    /// A turn's liveness.
    Turn,
    /// Something the current contract does not draw.
    Other,
}

impl Event {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Delta => "delta",
            Self::Thought => "thought",
            Self::Compaction => "compaction",
            Self::Activity => "activity",
            Self::ExternalEdits => "external_edits",
            Self::Prompt => "prompt",
            Self::PromptAnswered => "prompt_answered",
            Self::Plan => "plan",
            Self::Turn => "turn",
            Self::Other => "other",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "message" => Self::Message,
            "delta" => Self::Delta,
            "thought" => Self::Thought,
            "compaction" => Self::Compaction,
            "activity" => Self::Activity,
            "external_edits" => Self::ExternalEdits,
            "prompt" => Self::Prompt,
            "prompt_answered" => Self::PromptAnswered,
            "plan" => Self::Plan,
            "turn" => Self::Turn,
            "other" => Self::Other,
            _ => return None,
        })
    }
}

/// Who said it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::Tool => "tool",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "user" => Self::User,
            "assistant" => Self::Assistant,
            "system" => Self::System,
            "tool" => Self::Tool,
            _ => return None,
        })
    }
}

/// The station a call happened at. Fixed here so every harness and every view
/// agree on what a lit Files station means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivityKind {
    You,
    Model,
    Memory,
    Files,
    Shell,
    Network,
    Browser,
    Agent,
    Skill,
    Hook,
    Other,
}

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::You => "you",
            Self::Model => "model",
            Self::Memory => "memory",
            Self::Files => "files",
            Self::Shell => "shell",
            Self::Network => "network",
            Self::Browser => "browser",
            Self::Agent => "agent",
            Self::Skill => "skill",
            Self::Hook => "hook",
            Self::Other => "other",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "you" => Self::You,
            "model" => Self::Model,
            "memory" => Self::Memory,
            "files" => Self::Files,
            "shell" => Self::Shell,
            "network" => Self::Network,
            "browser" => Self::Browser,
            "agent" => Self::Agent,
            "skill" => Self::Skill,
            "hook" => Self::Hook,
            "other" => Self::Other,
            _ => return None,
        })
    }
}

/// ACP's tool vocabulary into the station vocabulary above.
///
/// One direction only, and one table: ACP is a protocol that will move, the
/// stations are the contract. `None` (a harness that classified nothing) is
/// `Other`, which is a real answer and not a guess.
pub fn activity_kind_from_acp(acp_kind: Option<&str>) -> ActivityKind {
    match acp_kind.unwrap_or_default() {
        "edit" | "read" | "delete" | "move" | "search" => ActivityKind::Files,
        "execute" => ActivityKind::Shell,
        "think" => ActivityKind::Model,
        "fetch" => ActivityKind::Network,
        _ => ActivityKind::Other,
    }
}

/// Did the call land, is it still running, did it fail, is it waiting on a
/// person.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Running,
    Ok,
    Failed,
    Blocked,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "running" => Self::Running,
            "ok" => Self::Ok,
            "failed" => Self::Failed,
            "blocked" => Self::Blocked,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_name_round_trips() {
        for event in [
            Event::Message,
            Event::Delta,
            Event::Thought,
            Event::Compaction,
            Event::Activity,
            Event::ExternalEdits,
            Event::Prompt,
            Event::PromptAnswered,
            Event::Plan,
            Event::Turn,
            Event::Other,
        ] {
            assert_eq!(Event::parse(event.as_str()), Some(event));
        }
        for kind in [
            ActivityKind::You,
            ActivityKind::Model,
            ActivityKind::Memory,
            ActivityKind::Files,
            ActivityKind::Shell,
            ActivityKind::Network,
            ActivityKind::Browser,
            ActivityKind::Agent,
            ActivityKind::Skill,
            ActivityKind::Hook,
            ActivityKind::Other,
        ] {
            assert_eq!(ActivityKind::parse(kind.as_str()), Some(kind));
        }
        for role in [Role::User, Role::Assistant, Role::System, Role::Tool] {
            assert_eq!(Role::parse(role.as_str()), Some(role));
        }
        for status in [Status::Running, Status::Ok, Status::Failed, Status::Blocked] {
            assert_eq!(Status::parse(status.as_str()), Some(status));
        }
        assert_eq!(Event::parse("invented"), None, "an unknown kind is not Other");
    }

    #[test]
    fn acls_tool_vocabulary_lands_on_a_station() {
        assert_eq!(activity_kind_from_acp(Some("edit")), ActivityKind::Files);
        assert_eq!(activity_kind_from_acp(Some("search")), ActivityKind::Files);
        assert_eq!(activity_kind_from_acp(Some("execute")), ActivityKind::Shell);
        assert_eq!(activity_kind_from_acp(Some("think")), ActivityKind::Model);
        assert_eq!(activity_kind_from_acp(Some("fetch")), ActivityKind::Network);
        assert_eq!(activity_kind_from_acp(None), ActivityKind::Other);
        assert_eq!(activity_kind_from_acp(Some("invented")), ActivityKind::Other);
    }

    #[test]
    fn an_event_is_one_record_put_in_one_book() {
        assert_eq!(book_id("abc"), "fungos/agents/abc");
        let line = put_line(
            "evt:7",
            json!({field::EVENT: Event::Message.as_str(), field::TEXT: "hi"}),
        );
        assert_eq!(line["kind"], OP_KIND);
        assert_eq!(line["entity_id"], "evt:7");
        assert_eq!(line["body"]["schema_id"], SCHEMA);
        assert_eq!(line["body"]["schema_version"], SCHEMA_VERSION);
        assert_eq!(line["body"]["fields"]["text"], "hi");
    }
}
