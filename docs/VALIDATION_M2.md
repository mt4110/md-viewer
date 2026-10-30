# M2 実装・検証（2026-10-29）

対象：T05〜T08。公開・push・コミットなし。M3の第三者再現は未実施。
最終目的はLeakFenceの理解と導入検討への入口。今回は同じSuiteReportからJSONとオフラインHTMLを生成し、その対応を確認した。

## 変更

- Rustのrender.rs、固定HTML/CSSを追加。判定を再実装せず、作者の説明と実判定を別表示。
- Context/Policy、原本応答、実判定を3列で表示。狭い画面は縦積み。
- 注釈行のmark、ページ内リンク、Policyのdetails、期待不一致バナーを追加。不一致ケースではmarkを無効化。
- 全入力文字列をescape。JavaScript、外部素材、fetch、動的URLなし。CSPはscript-src none。
- JSONを先に、HTMLを最後にcreate_newで保存。途中失敗では終了1、部分出力を保全。
- 実画面で検出した期待値文字列の横はみ出しを折り返しで修正。

## 検証結果

環境：macOS arm64、Rust/cargo 1.95.0。依存追加なし。

| 条件 | 結果 |
| --- | --- |
| fmt、cargo test --locked、Clippy -D warnings | 成功、23試験 |
| 固定coreの5ケース | 全期待一致 |
| JSON/HTMLを別ディレクトリへ2回生成 | 両ファイルがそれぞれバイト一致 |
| HTML保存中のOS書込上限 | JSON完了後、HTMLが10,000 bytesで失敗。終了1とincomplete_outputを確認 |
| escape・不一致・境界 | 攻撃文字列のescape、不一致mark無効、件数・UTF-8バイト境界、入れ子、不正JSON、空配列、最初の拒否理由を試験 |
| Chromium 153.0.8010.12 | offlineコンテキストでfile://を開き、全5ケースの状態・理由・照合をJSONと一致確認 |
| 原本表示 | Context/Policy/response計15箇所のDOM textContentが原本と完全一致 |
| 通信・console | HTTP(S)要求0、関連console warning/error・pageerror 0 |
| 操作 | ケースリンク→hash移動、summaryへfocus→Enterで展開を確認 |
| レイアウト | 1440×1000、390×844、720×500で200%文字拡大。ページの横はみ出しなし。128文字の追記でも確認 |
| Safari 27.0.1 | ネイティブSafariでfile://表示、全ケースのAX上の内容、他テナントへのリンク移動、修正後HTMLの読込を確認 |

専用Browserプラグイン/スキルは未提供だったため、バンドル済みPlaywrightを使用。
その既定ブラウザ版は未導入だったため、インストール済みChromium executableを明示。ブラウザの新規導入なし。
SafariはCUAで確認。Safariのconsole/DOM自動照合は未実施で、詳細照合はChromiumでの結果。

## 再実行・証拠

リポジトリルートから、存在しない出力先を指定する。

```sh
rustup run 1.95.0 cargo test --locked
rustup run 1.95.0 cargo fmt --all --check
rustup run 1.95.0 cargo clippy --locked --all-targets -- -D warnings
rustup run 1.95.0 cargo run --frozen -- build --out out/new-run
```

- 最終成果物：out/m2-final-a/ と out/m2-final-b/（report.json、index.html）
- 保存失敗証拠：out/m2-html-partial/（意図的な部分HTML）。削除していない。
- Chromium検証スクリプト：/tmp/veil-m2-qa.cjs
- スクリーンショット：/tmp/veil-m2-final-desktop.png、/tmp/veil-m2-final-mobile.png

書込上限は子プロセスだけにRLIMIT_FSIZE=10000、SIGXFSZ無視を設定した。実ディスク枯渇・電源断試験ではない。
旧M1と今回の途中出力も保全。checkは引き続き保存を行わない。

## 残る範囲

- VoiceOverの音声読み上げ、ブラウザ全体の200%ズーム、別OS、第三者再実行は未確認。AXの順序と文字拡大を確認した。
- offline設定はChromiumのコンテキストに限定。OS全体のネットワーク設定は変更していない。
- 日次枠、認証・認可、本番通信遮断、販売価値の証明ではない。
- T09以降は未着手。実装とローカル検証の完了を最終ゴール達成とは扱わない。
