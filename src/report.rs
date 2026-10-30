use crate::LabError;
use crate::engine::{ContentCheck, ENGINE_REPO, ENGINE_REV, evaluate_case};
use crate::fixture::{CaseKind, Expected, Highlight, LoadedSuite};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct SuiteReport {
    pub schema_version: u32,
    pub suite: SuiteInfo,
    pub generator: GeneratorInfo,
    pub engine: EngineInfo,
    pub scope: Scope,
    pub summary: Summary,
    pub cases: Vec<CaseResult>,
}

#[derive(Debug, Serialize)]
pub struct SuiteInfo {
    pub id: String,
    pub version: u32,
    pub contract: String,
}

#[derive(Debug, Serialize)]
pub struct GeneratorInfo {
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Serialize)]
pub struct EngineInfo {
    pub name: &'static str,
    pub repo: &'static str,
    pub git_rev: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Scope {
    pub content_only: bool,
    pub budget_check: &'static str,
    pub network: &'static str,
    pub data: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Summary {
    pub total: usize,
    pub matched: usize,
    pub mismatched: usize,
}

#[derive(Debug, Serialize)]
pub struct CaseResult {
    pub input: CaseInput,
    pub actual: ContentCheck,
    pub expectation_check: ExpectationCheck,
    pub highlights_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct CaseInput {
    pub id: String,
    pub title: String,
    pub kind: CaseKind,
    pub synthetic: bool,
    pub policy: String,
    pub context: String,
    pub response: String,
    pub expected: Expected,
    pub lesson: String,
    pub highlights: Vec<Highlight>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExpectationCheck {
    Matched,
    Mismatched,
}

pub fn evaluate_suite(suite: LoadedSuite) -> SuiteReport {
    let mut cases = Vec::new();
    let mut matched = 0;
    for case in suite.cases {
        let actual = evaluate_case(case.response.as_bytes(), &suite.policy, &suite.context);
        let agrees = case.spec.expected.matches(&actual);
        matched += usize::from(agrees);
        cases.push(CaseResult {
            actual,
            expectation_check: if agrees {
                ExpectationCheck::Matched
            } else {
                ExpectationCheck::Mismatched
            },
            highlights_enabled: agrees && !case.spec.highlights.is_empty(),
            input: CaseInput {
                id: case.spec.id,
                title: case.spec.title,
                kind: case.spec.kind,
                synthetic: case.spec.synthetic,
                policy: suite.policy_text.clone(),
                context: suite.context_text.clone(),
                response: case.response,
                expected: case.spec.expected,
                lesson: case.spec.lesson,
                highlights: case.spec.highlights,
            },
        });
    }
    SuiteReport {
        schema_version: 1,
        suite: SuiteInfo {
            id: suite.id,
            version: suite.version,
            contract: suite.policy.id,
        },
        generator: GeneratorInfo {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
        },
        engine: EngineInfo {
            name: "leak-fence-core",
            repo: ENGINE_REPO,
            git_rev: ENGINE_REV,
        },
        scope: Scope {
            content_only: true,
            budget_check: "not_performed",
            network: "not_used",
            data: "synthetic",
        },
        summary: Summary {
            total: cases.len(),
            matched,
            mismatched: cases.len() - matched,
        },
        cases,
    }
}

fn write_json(report: &SuiteReport, writer: &mut impl Write) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, report).map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

/// Save JSON first and HTML last; never replace an existing output directory.
/// A failed write leaves the new directory and partial file for inspection.
pub fn build_report(report: &SuiteReport, output: &Path) -> Result<PathBuf, LabError> {
    let html = crate::render::render_html(report)?;
    // symlink_metadata also detects dangling symlinks; never overwrite them.
    match fs::symlink_metadata(output) {
        Ok(_) => return Err(LabError::new("output_exists")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(_) => return Err(LabError::new("output_directory")),
    }
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|_| LabError::new("output_parent"))?;
    }
    fs::create_dir(output).map_err(|_| LabError::new("output_directory"))?;
    let path = output.join("report.json");
    let saved = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        write_json(report, &mut file)?;
        let mut html_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join("index.html"))?;
        html_file.write_all(html.as_bytes())?;
        html_file.flush()
    })();
    saved.map_err(|_: io::Error| LabError {
        code: "report_write",
        case_id: None,
        incomplete_output: Some(output.to_path_buf()),
    })?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingWriter {
        bytes: Vec<u8>,
        remaining: usize,
        fail_flush: bool,
    }

    impl Write for FailingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.remaining == 0 {
                return Err(io::Error::other("injected write failure"));
            }
            let count = bytes.len().min(self.remaining);
            self.bytes.extend_from_slice(&bytes[..count]);
            self.remaining -= count;
            Ok(count)
        }

        fn flush(&mut self) -> io::Result<()> {
            if self.fail_flush {
                Err(io::Error::other("injected flush failure"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn partial_write_and_flush_errors_are_not_successes() {
        let report = evaluate_suite(crate::load_suite(Path::new("fixtures/demo")).unwrap());
        let mut writer = FailingWriter {
            bytes: Vec::new(),
            remaining: 80,
            fail_flush: false,
        };
        assert!(write_json(&report, &mut writer).is_err());
        assert_eq!(writer.bytes.len(), 80);
        let mut writer = FailingWriter {
            bytes: Vec::new(),
            remaining: usize::MAX,
            fail_flush: true,
        };
        assert!(write_json(&report, &mut writer).is_err());
        assert!(!writer.bytes.is_empty());
    }
}
