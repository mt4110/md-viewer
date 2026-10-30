use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use veil_boundary_lab::{LabError, build_report, evaluate_suite, load_suite};

const HELP: &str = "veil-boundary-lab — synthetic cases / 合成ケース専用\n\
Run from the repository root / リポジトリルートから実行:\n\
  veil-boundary-lab check\n\
  veil-boundary-lab build --out <new-directory>\n\
M2: build saves report.json and offline index.html.\n\
The report saves complete synthetic inputs locally. No network or daily budget check.\n\
合成入力をローカル保存します。日次取得枠・API通信遮断は対象外です。\n\
Exit: 0 expectations matched; 2 mismatched; 1 input/usage/I/O failure.\n";

enum Command {
    Check,
    Build(PathBuf),
    Help,
}

fn command(args: &[OsString]) -> Option<Command> {
    match args {
        [one] if one == "check" => Some(Command::Check),
        [one] if one == "--help" || one == "-h" || one == "help" => Some(Command::Help),
        [one, flag, path] if one == "build" && flag == "--out" && !path.is_empty() => {
            Some(Command::Build(PathBuf::from(path)))
        }
        _ => None,
    }
}

fn execute(command: Command, stdout: &mut impl Write) -> Result<u8, LabError> {
    if matches!(command, Command::Help) {
        stdout
            .write_all(HELP.as_bytes())
            .and_then(|()| stdout.flush())
            .map_err(|_| io_error())?;
        return Ok(0);
    }
    let report = evaluate_suite(load_suite(Path::new("fixtures/demo"))?);
    let output = match command {
        Command::Build(path) => Some(build_report(&report, &path)?),
        _ => None,
    };
    let summary = &report.summary;
    let print = (|| -> io::Result<()> {
        writeln!(
            stdout,
            "total={} matched={} mismatched={}",
            summary.total, summary.matched, summary.mismatched
        )?;
        for case in &report.cases {
            writeln!(
                stdout,
                "case={} result={} expectation={}",
                case.input.id,
                case.actual.code(),
                if case.expectation_check == veil_boundary_lab::report::ExpectationCheck::Matched {
                    "matched"
                } else {
                    "mismatched"
                }
            )?;
        }
        if let Some(path) = &output {
            writeln!(stdout, "stage=m2-json-html report={path:?}")?;
        }
        stdout.flush()
    })();
    print.map_err(|_| io_error())?;
    Ok(if summary.mismatched == 0 { 0 } else { 2 })
}

fn io_error() -> LabError {
    LabError {
        code: "stdout_write",
        case_id: None,
        incomplete_output: None,
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mut stdout = io::stdout().lock();
    let result = match command(&args) {
        Some(cmd) => execute(cmd, &mut stdout),
        None => Err(LabError {
            code: "usage",
            case_id: None,
            incomplete_output: None,
        }),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let mut stderr = io::stderr().lock();
            // A stderr failure must still return 1 rather than panic/succeed.
            let _ = writeln!(stderr, "{error}").and_then(|()| stderr.flush());
            ExitCode::FAILURE
        }
    }
}
