//! Presentation only: all outcomes come from SuiteReport, never from JSON reinspection.
use crate::{
    LabError,
    engine::ContentCheck,
    fixture::Source,
    report::{ExpectationCheck, SuiteReport},
};
use std::fmt::Write;

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub fn render_html(report: &SuiteReport) -> Result<String, LabError> {
    let mut html = include_str!("../assets/report.html")
        .replace("{{CSS}}", include_str!("../assets/report.css"));
    let _ = write!(
        html,
        "<header><p class=eyebrow>VEIL BOUNDARY LAB · 合成データ</p><h1>このレスポンス、返していい？</h1><p>秘密らしい文字列がなくても、返す相手を間違えれば越境します。</p><p>日次取得枠：未検査 · API通信の遮断：未実施</p><p>保存済みの内容検査結果です。更新にはCLIの再実行が必要です。</p><small>検査器 rev: {}</small></header>",
        escape(report.engine.git_rev)
    );
    if report.summary.mismatched > 0 {
        html.push_str("<p role=alert class=warning>期待結果と実判定が一致していません。説明用の強調は不一致ケースで無効です。</p>");
    }
    html.push_str("<nav aria-label=ケース一覧>");
    for case in &report.cases {
        if !slug(&case.input.id) {
            return Err(LabError::new("render_id"));
        }
        let _ = write!(
            html,
            "<a href=\"#case-{}\">{}</a>",
            case.input.id,
            escape(&case.input.title)
        );
    }
    html.push_str("</nav><main>");
    for case in &report.cases {
        let input = &case.input;
        let matched = case.expectation_check == ExpectationCheck::Matched;
        let (state, reason, label) = match case.actual {
            ContentCheck::Passed { .. } => ("passed", "", "内容検査に通過"),
            ContentCheck::Denied { reason } => ("denied", reason.code(), "応答全体を拒否"),
        };
        let _ = write!(
            html,
            "<section id=\"case-{}\" tabindex=\"-1\"><h2>{}</h2><div class=columns><article><h3>返してよい範囲</h3><p>サーバー側の認可結果を模した合成データ</p>",
            input.id,
            escape(&input.title)
        );
        for (source, name, raw) in [
            (Source::Context, "context", input.context.as_str()),
            (Source::Policy, "policy", input.policy.as_str()),
            (Source::Response, "response", input.response.as_str()),
        ] {
            if source == Source::Response {
                html.push_str("</article><article><h3>APIが返そうとしたJSON</h3>");
            }
            if source == Source::Policy {
                html.push_str(
                    "<details><summary>Policyの原本（日次上限はこの検査では未使用）</summary>",
                );
            }
            let _ = write!(html, "<pre data-source=\"{name}\"><code>");
            for (index, line) in raw.split_inclusive('\n').enumerate() {
                let mark = input.highlights.iter().find(|h| {
                    h.source == source && h.start_line <= index + 1 && h.end_line > index
                });
                if matched
                    && case.highlights_enabled
                    && let Some(h) = mark
                {
                    if !slug(&h.id) {
                        return Err(LabError::new("render_id"));
                    }
                    let _ = write!(
                        html,
                        "<mark id=\"lab-{}/{}/{}-{}\" title=\"{}\">{}</mark>",
                        input.id,
                        name,
                        h.id,
                        index + 1,
                        escape(&h.label),
                        escape(line)
                    );
                } else {
                    html.push_str(&escape(line));
                }
            }
            html.push_str("</code></pre>");
            if source == Source::Policy {
                html.push_str("</details>");
            }
        }
        let _ = write!(
            html,
            "</article><article class=result data-case-id=\"{}\" data-content-check=\"{state}\" data-reason=\"{reason}\" data-expectation-check=\"{}\"><h3>判定と理由</h3><strong>{label}</strong><p class=reason>{reason}</p>",
            input.id,
            if matched { "matched" } else { "mismatched" }
        );
        if let ContentCheck::Passed { cost } = case.actual {
            let _ = write!(
                html,
                "<p>{}件 · {} UTF-8 bytes</p>",
                cost.records, cost.bytes
            );
        }
        let expected =
            serde_json::to_string(&input.expected).map_err(|_| LabError::new("render_expected"))?;
        let _ = write!(
            html,
            "<p>期待値：<code>{}</code></p><p>期待照合：{}</p><h4>ケースの説明（作者）</h4><p>{}</p>",
            escape(&expected),
            if matched { "一致" } else { "不一致" },
            escape(&input.lesson)
        );
        if matched && case.highlights_enabled {
            html.push_str("<p>説明用の強調。検査器は位置情報を返しません。</p><ul>");
            for h in &input.highlights {
                let name = match h.source {
                    Source::Policy => "policy",
                    Source::Context => "context",
                    Source::Response => "response",
                };
                let _ = write!(
                    html,
                    "<li><a href=\"#lab-{}/{}/{}-{}\">{}：{}</a></li>",
                    input.id,
                    name,
                    h.id,
                    h.start_line,
                    escape(&h.id),
                    escape(&h.label)
                );
            }
            html.push_str("</ul>");
        }
        html.push_str(
            "<p>日次取得枠：未検査<br>API通信の遮断：未実施</p></article></div></section>",
        );
    }
    html.push_str("</main><footer><p>認証・認可の正しさ、許可値の意味・出所、本番防御はこのデモで確認していません。</p><a href=\"https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/docs/QUICKSTART.md\">自分の認可情報と応答を検査する：LeakFence CLIの導入例へ</a></footer></body></html>");
    Ok(html)
}
