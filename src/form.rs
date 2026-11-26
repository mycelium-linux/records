//! A canonical, inert envelope for homoiconic Soil forms in a record body.
//!
//! `records` bodies are JSON today. A Soil form has distinctions JSON does not:
//! symbols, keywords and strings are three values; `nil` is not an empty list;
//! a list of keyword pairs is not necessarily an object. This module is an
//! optional schema envelope, not a new store or evaluator:
//!
//! ```text
//! {"$soil":"form","v":1,"value":["symbol","project"]}
//! ```
//!
//! The tag is explicit, canonical JSON (BTreeMap key order from serde_json),
//! and the value is data. Nothing here evaluates a form or grants authority.
//! Use schema [`SCHEMA`] so a reader can choose the codec by schema id.

use serde_json::{Value, json};

/// Schema id for a body encoded by [`Form::encode`].
pub const SCHEMA: &str = "fungos.soil/form";
pub const VERSION: i64 = 1;
const TAG: &str = "$soil";
const FORM: &str = "form";
const ENVELOPE_VERSION: u64 = 1;

/// The subset of Soil values this codec preserves exactly.
///
/// `List` is the Lisp list. Vectors and maps are deliberately absent: today's
/// `soil-lang::sexp::Value` has no distinct vector or map primitive, and
/// inventing them here would make a body that no Soil reader can construct.
#[derive(Clone, Debug, PartialEq)]
pub enum Form {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Symbol(String),
    Keyword(String),
    String(String),
    List(Vec<Form>),
}

impl Form {
    /// The tagged JSON value stored in a record body.
    pub fn encode(&self) -> Value {
        json!({ "$soil": FORM, "v": ENVELOPE_VERSION, "value": self.tagged() })
    }

    /// Stable JSON bytes for a body or a digest. Object key order is sorted by
    /// `serde_json`'s map; no whitespace is emitted.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(&self.encode()).map_err(|error| error.to_string())
    }

    /// Content identity for a form, independent of source formatting.
    pub fn digest(&self) -> Result<String, String> {
        Ok(format!(
            "blake3:{}",
            blake3::hash(&self.canonical_bytes()?).to_hex()
        ))
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let value: Value = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        Self::from_envelope(&value)
    }

    fn tagged(&self) -> Value {
        match self {
            Self::Nil => json!(["nil"]),
            Self::Bool(value) => json!(["bool", value]),
            Self::Int(value) => json!(["int", value]),
            Self::Float(value) if value.is_finite() => json!(["float", value]),
            Self::Float(_) => json!(["invalid-float"]),
            Self::Symbol(value) => json!(["symbol", value]),
            Self::Keyword(value) => json!(["keyword", value]),
            Self::String(value) => json!(["string", value]),
            Self::List(items) => json!([
                "list",
                items.iter().map(Self::tagged).collect::<Vec<_>>()
            ]),
        }
    }

    fn from_envelope(value: &Value) -> Result<Self, String> {
        let object = value.as_object().ok_or("form envelope is not an object")?;
        if object.len() != 3 {
            return Err("form envelope must contain exactly $soil, v and value".into());
        }
        if object.get(TAG).and_then(Value::as_str) != Some(FORM) {
            return Err("not a soil form envelope".into());
        }
        if object.get("v").and_then(Value::as_u64) != Some(ENVELOPE_VERSION) {
            return Err("unsupported form envelope version".into());
        }
        let tagged = object.get("value").ok_or("form envelope has no value")?;
        Self::from_tagged(tagged)
    }

    fn from_tagged(value: &Value) -> Result<Self, String> {
        let parts = value.as_array().ok_or("tagged form is not an array")?;
        let tag = parts.first().and_then(Value::as_str).ok_or("tagged form has no tag")?;
        match tag {
            "nil" if parts.len() == 1 => Ok(Self::Nil),
            "bool" if parts.len() == 2 => parts[1]
                .as_bool()
                .map(Self::Bool)
                .ok_or_else(|| "bool tag is not a boolean".into()),
            "int" if parts.len() == 2 => parts[1]
                .as_i64()
                .map(Self::Int)
                .ok_or_else(|| "int tag is not an i64".into()),
            "float" if parts.len() == 2 => parts[1]
                .as_f64()
                .filter(|value| value.is_finite())
                .map(Self::Float)
                .ok_or_else(|| "float tag is not finite".into()),
            "symbol" if parts.len() == 2 => parts[1]
                .as_str()
                .map(|value| Self::Symbol(value.to_owned()))
                .ok_or_else(|| "symbol tag is not a string".into()),
            "keyword" if parts.len() == 2 => parts[1]
                .as_str()
                .map(|value| Self::Keyword(value.to_owned()))
                .ok_or_else(|| "keyword tag is not a string".into()),
            "string" if parts.len() == 2 => parts[1]
                .as_str()
                .map(|value| Self::String(value.to_owned()))
                .ok_or_else(|| "string tag is not a string".into()),
            "list" if parts.len() == 2 => parts[1]
                .as_array()
                .ok_or("list tag is not an array")?
                .iter()
                .map(Self::from_tagged)
                .collect::<Result<Vec<_>, _>>()
                .map(Self::List),
            other => Err(format!("unknown or malformed form tag {other:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Store;

    fn samples() -> Vec<Form> {
        vec![
            Form::Nil,
            Form::Bool(true),
            Form::Int(-7),
            Form::Float(1.5),
            Form::Symbol("project".into()),
            Form::Keyword("project".into()),
            Form::String("project".into()),
            Form::List(vec![
                Form::Symbol("defn".into()),
                Form::Symbol("f".into()),
                Form::List(vec![Form::Keyword("x".into()), Form::Int(1)]),
                Form::List(vec![Form::Symbol("quote".into()), Form::Symbol("x".into())]),
            ]),
        ]
    }

    #[test]
    fn every_native_type_roundtrips_without_collapsing() {
        for form in samples() {
            let bytes = form.canonical_bytes().unwrap();
            assert_eq!(Form::decode(&bytes).unwrap(), form);
        }
        assert_ne!(
            Form::Symbol("project".into()).canonical_bytes().unwrap(),
            Form::Keyword("project".into()).canonical_bytes().unwrap()
        );
        assert_ne!(
            Form::Keyword("project".into()).canonical_bytes().unwrap(),
            Form::String("project".into()).canonical_bytes().unwrap()
        );
        assert_ne!(
            Form::Nil.canonical_bytes().unwrap(),
            Form::List(vec![]).canonical_bytes().unwrap()
        );
    }

    #[test]
    fn digest_is_stable_and_content_addressed() {
        let a = Form::List(vec![Form::Symbol("quote".into()), Form::Symbol("x".into())]);
        let b = Form::List(vec![Form::Symbol("quote".into()), Form::Symbol("x".into())]);
        assert_eq!(a.digest().unwrap(), b.digest().unwrap());
        assert_ne!(
            a.digest().unwrap(),
            Form::List(vec![Form::Symbol("quote".into()), Form::Keyword("x".into())])
                .digest()
                .unwrap()
        );
    }

    #[test]
    fn a_form_is_a_normal_records_body_and_remains_inert() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path(), &dir.path().join("device_id")).unwrap();
        let form = Form::List(vec![
            Form::Symbol("eval".into()),
            Form::List(vec![
                Form::Symbol("shell".into()),
                Form::String("printf not-executed".into()),
            ]),
        ]);
        let body = form.encode();
        store
            .put_record(
                "personal/test/forms",
                "form-1",
                SCHEMA,
                VERSION,
                body.clone(),
            )
            .unwrap();
        let record = store
            .get_optional("personal/test/forms", "form-1")
            .unwrap()
            .unwrap();
        assert_eq!(record.schema_id, SCHEMA);
        let decoded = Form::decode(&serde_json::to_vec(&record.fields).unwrap()).unwrap();
        assert_eq!(decoded, form);
        assert_eq!(record.fields, body);
        // The body is data: the store returns the same bytes and no child
        // process, grant, or evaluation was created.
        assert_eq!(store.search_book("personal/test/forms", "not-executed", 10).unwrap().len(), 1);
    }

    #[test]
    fn malformed_or_unknown_tags_fail_closed() {
        assert!(Form::decode(br#"{"$soil":"form","v":1,"value":["unknown"]}"#).is_err());
        assert!(Form::decode(br#"{"$soil":"form","v":2,"value":["nil"]}"#).is_err());
        assert!(Form::decode(br#"{"$soil":"form","v":1,"value":["float",1e999]}"#).is_err());
    }
}
