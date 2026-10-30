use crate::LabError;
use crate::engine::{ContentCheck, DenialCode};
use leak_fence_core::{Context, MAX_BODY_BYTES, Policy, validate_policies};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path};

const MAX_METADATA_BYTES: usize = 65_536;
const MAX_POLICY_BYTES: usize = 131_072;
const MAX_CASES: usize = 16;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SuiteSpec {
    schema_version: u32,
    id: String,
    version: u32,
    policy_file: String,
    contract: String,
    context_file: String,
    cases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseSpec {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub kind: CaseKind,
    pub synthetic: bool,
    pub response_file: String,
    pub expected: Expected,
    pub lesson: String,
    pub highlights: Vec<Highlight>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseKind {
    Boundary,
    Limitation,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "content_check", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Passed {},
    Denied { reason: DenialCode },
}

impl Expected {
    pub fn matches(&self, actual: &ContentCheck) -> bool {
        match (self, actual) {
            (Self::Passed {}, ContentCheck::Passed { .. }) => true,
            (Self::Denied { reason: wanted }, ContentCheck::Denied { reason: actual }) => {
                wanted == actual
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Policy,
    Context,
    Response,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Highlight {
    pub id: String,
    pub source: Source,
    pub start_line: usize,
    pub end_line: usize,
    pub expected_text: String,
    pub label: String,
}

pub struct LoadedCase {
    pub(crate) spec: CaseSpec,
    pub(crate) response: String,
}

pub struct LoadedSuite {
    pub(crate) id: String,
    pub(crate) version: u32,
    pub(crate) policy: Policy,
    pub(crate) context: Context,
    pub(crate) policy_text: String,
    pub(crate) context_text: String,
    pub(crate) cases: Vec<LoadedCase>,
}

fn valid_slug(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

fn read_text(root: &Path, relative: &str, limit: usize) -> Result<String, LabError> {
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(LabError::new("fixture_path"));
    }
    // Reject symlinks at every component, including those pointing within root.
    // The local checkout/host must be trusted against concurrent replacement.
    let mut path = root.to_path_buf();
    for component in relative.components() {
        path.push(component);
        let metadata = fs::symlink_metadata(&path).map_err(|_| LabError::new("fixture_file"))?;
        if metadata.file_type().is_symlink() {
            return Err(LabError::new("fixture_path"));
        }
    }
    if !fs::metadata(&path)
        .map_err(|_| LabError::new("fixture_file"))?
        .is_file()
    {
        return Err(LabError::new("fixture_file"));
    }
    let file = File::open(&path).map_err(|_| LabError::new("fixture_file"))?;
    if !file
        .metadata()
        .map_err(|_| LabError::new("fixture_file"))?
        .is_file()
    {
        return Err(LabError::new("fixture_file"));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| LabError::new("fixture_read"))?;
    if bytes.len() > limit {
        return Err(LabError::new("fixture_limit"));
    }
    String::from_utf8(bytes).map_err(|_| LabError::new("fixture_utf8"))
}

fn parse<T: DeserializeOwned>(text: &str) -> Result<T, LabError> {
    serde_json::from_str(text).map_err(|_| LabError::new("fixture_json"))
}

fn validate_highlights(
    highlights: &[Highlight],
    policy: &str,
    context: &str,
    response: &str,
) -> Result<(), LabError> {
    let mut ids = BTreeSet::new();
    let mut ranges: Vec<(Source, usize, usize)> = Vec::new();
    for h in highlights {
        if !valid_slug(&h.id) || !ids.insert((h.source, &h.id)) {
            return Err(LabError::new("highlight_id"));
        }
        let raw = match h.source {
            Source::Policy => policy,
            Source::Context => context,
            Source::Response => response,
        };
        // A final LF ends the last line rather than creating another empty one.
        let lines: Vec<_> = raw.strip_suffix('\n').unwrap_or(raw).split('\n').collect();
        if h.start_line == 0 || h.end_line < h.start_line || h.end_line > lines.len() {
            return Err(LabError::new("highlight_range"));
        }
        if ranges.iter().any(|&(source, start, end)| {
            source == h.source && h.start_line <= end && h.end_line >= start
        }) {
            return Err(LabError::new("highlight_overlap"));
        }
        if lines[h.start_line - 1..h.end_line].join("\n") != h.expected_text {
            return Err(LabError::new("highlight_text"));
        }
        ranges.push((h.source, h.start_line, h.end_line));
    }
    Ok(())
}

pub fn load_suite(root: &Path) -> Result<LoadedSuite, LabError> {
    let metadata = fs::symlink_metadata(root).map_err(|_| LabError::new("fixture_root"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(LabError::new("fixture_root"));
    }
    let root = root
        .canonicalize()
        .map_err(|_| LabError::new("fixture_root"))?;
    let suite: SuiteSpec = parse(&read_text(&root, "suite.json", MAX_METADATA_BYTES)?)?;
    if suite.schema_version != 1 || suite.version == 0 || !valid_slug(&suite.id) {
        return Err(LabError::new("suite_schema"));
    }
    if suite.cases.is_empty() || suite.cases.len() > MAX_CASES {
        return Err(LabError::new("suite_cases"));
    }
    let policy_text = read_text(&root, &suite.policy_file, MAX_POLICY_BYTES)?;
    let policies: Vec<Policy> = parse(&policy_text)?;
    validate_policies(&policies).map_err(|_| LabError::new("configuration"))?;
    let policy = policies
        .into_iter()
        .find(|p| p.id == suite.contract)
        .ok_or_else(|| LabError::new("contract"))?;
    let context_text = read_text(&root, &suite.context_file, MAX_METADATA_BYTES)?;
    let context: Context = parse(&context_text)?;
    let mut cases = Vec::new();
    let mut ids = BTreeSet::new();
    for path in suite.cases {
        let spec: CaseSpec = parse(&read_text(&root, &path, MAX_METADATA_BYTES)?)?;
        if !valid_slug(&spec.id) {
            return Err(LabError::new("case_id"));
        }
        if spec.schema_version != 1 || !spec.synthetic {
            return Err(LabError::new("case_schema").for_case(&spec.id));
        }
        if !ids.insert(spec.id.clone()) {
            return Err(LabError::new("duplicate_case").for_case(&spec.id));
        }
        let response = read_text(&root, &spec.response_file, MAX_BODY_BYTES + 1)
            .map_err(|e| e.for_case(&spec.id))?;
        validate_highlights(&spec.highlights, &policy_text, &context_text, &response)
            .map_err(|e| e.for_case(&spec.id))?;
        cases.push(LoadedCase { spec, response });
    }
    Ok(LoadedSuite {
        id: suite.id,
        version: suite.version,
        policy,
        context,
        policy_text,
        context_text,
        cases,
    })
}
