# M1 実装・検証記録

確認日：2026-10-29。対象はT01〜T04。HTML、公開、push、第三者への連絡は実施していません。
最終ゴールはLeakFenceの理解と導入検討への入口です。今回達成したのは、合成ケースを実検査器で照合してJSONへ残すM1です。

## 1. 環境と対象

- macOS 27.0.1（26A434）、arm64。
- rustc 1.95.0（59807616e）、cargo 1.95.0（f2d3ce0bd）。全Cargo操作を`rustup run 1.95.0`で実行。
- labの基点は`81e6282`。今回のコード・fixture・文書は未コミットの作業ツリーに存在。
- 実検査器：leak-fence-core 0.1.0、Git rev `6a0f416e23da5e0cff5890fd05b6b1d9a4162393`。
- Git依存とCargo.lockから取得してビルド。隣のローカルLeakFenceへのpath依存・変更なし。
- CLIとライブラリは単一Cargoパッケージ。実行時のネットワーク、認証、日次枠、Worker、Wasm、DBは追加していない。

## 2. 完了条件との対応

| タスク | 完了条件 | 確認結果 |
| --- | --- | --- |
| T01 | 固定依存とRust、check、解決rev、依存・ライセンス・助言確認 | check成功。metadataのcore revとレポート値が一致。Worker依存なし。詳細は5節 |
| T02 | 5ケース、原本保持、構造・パス・読込上限・注釈検証 | 読込と10件のsuite統合試験で確認。fixtureはLF、日本語本文、7つの作者注釈 |
| T03 | 実coreの許可・拒否・理由を変えずに照合 | 全5ケース一致。異なる結果・異なる拒否理由の期待値で不一致。9変種のコード対応も確認 |
| T04 | check/build/help、終了0/2/1、JSON保存、再現性、既存出力保全、I/O失敗 | CLI統合試験9件と書込試験。2回のJSONがバイト一致。OS書込上限による部分失敗も確認 |

M1のbuildはreport.jsonだけを保存する中間実装です。help・README・stdoutへ明記しています。
最終契約のindex.html生成、画面の照合・escape確認はM2に残しています。

## 3. 実際の判定

| ケース | 実判定 | 期待照合 |
| --- | --- | --- |
| allowed | passed、1件、90 bytes | matched |
| foreign-tenant | denied / tenant_mismatch | matched |
| unauthorized-id | denied / object_unauthorized | matched |
| extra-field | denied / unapproved_field | matched |
| value-not-classified | passed、1件、107 bytes | matched |

bytesは原本のUTF-8バイト数です。拒否結果のcostを推測・保存していません。
制約ケースの許可は、値の意味・出所を検査していないことを示します。
他テナントのケースは正常行と違反行が混在し、違反行のID自体は許可集合に含まれています。

実行したコマンド：

```sh
rustup run 1.95.0 cargo check --locked
rustup run 1.95.0 cargo fmt --all --check
rustup run 1.95.0 cargo test --locked
rustup run 1.95.0 cargo clippy --locked --all-targets -- -D warnings
rustup run 1.95.0 cargo run --frozen -- check
rustup run 1.95.0 cargo run --frozen -- build --out out/m1-20261029-final-a
rustup run 1.95.0 cargo run --frozen -- build --out out/m1-20261029-final-b
```

check、fmt、21件の試験、Clippyに成功。CLIの上記3回も終了0、全5ケース一致。
21件の内訳は、ライブラリ単体2件、suite統合10件、CLI/report統合9件です。
試験数を網羅性や本番品質の証明には使いません。

## 4. 重要な異常系と再現性

- 許可を拒否に変えた期待値と、別の拒否理由に変えた期待値の両方が不一致になる。
- 不一致は終了2で、実判定・期待・元入力を保存し、理由に結び付く注釈を無効にする。
- Policy配列に複数契約があるときも選択したcontractを保存し、実際に選んだ契約のpermissionで検査する。
- 注釈の文字列ずれ、逆順・0行・範囲外、重複ID・交差範囲は実行失敗になる。
- 本文の重複キーは正規化せずcoreへ渡してinvalid_jsonとなる。末尾LFと空白も保持する。
- 無効なschema、未知・重複フィールド、壊れた期待値、空・過剰ケース、不明contract、無効Policyは内容拒否へ混ぜない。
- 読込ファイルの不足・ディレクトリ・絶対パス・親への逸脱・symlink・サイズ超過・不正UTF-8を拒否する。
- 本文が上流の絶対バイト上限+1ならそのままcoreのbyte_limitとなり、ラボの上限超過と区別する。
- 既存ディレクトリ、既存ファイル、既存レポート、dangling symlinkを置換しない。
- 閉じたstdoutへの出力は終了1。serde等の入力抜粋をstderrへ出さない。
- 注入した短い書込とflush失敗を成功扱いにしない。

OS側の保存失敗は、子プロセスだけの`RLIMIT_FSIZE=128`とSIGXFSZ無視で確認しました。
128 bytesの部分report.jsonが残り、終了1、stdout空、stderrに`report_write`と`incomplete_output`が出ました。
同じ出力先への再試行も終了1で、部分ファイルは128 bytesのまま保全されました。
上限の設定はその子プロセスだけで、端末や親プロセスの上限は変更していません。
これはローカル書込失敗の再現で、実ディスク枯渇や電源断の試験ではありません。

最終状態で2回生成したreport.jsonは各9,310 bytesで完全一致しました。
SHA-256は`61d7eddf4dfce01fc7ed3e2c011c59982aaebfc8cabfdfbefa7bfaff7e4ba414`です。
日時・絶対入力パス・出力先はreportへ含めていません。
同一環境・同一入力での2回の一致であり、全プラットフォームのビット再現を保証しません。

残したローカル証拠（out/はGit対象外）：

- out/m1-20261029-final-a/report.json と out/m1-20261029-final-b/report.json
- out/m1-write-limit-20261029-final/report.json（意図的な部分出力）
- out/m1-final-observations.json
- out/m1-dependencies.json、out/m1-rustsec-review.json
- out/test-runs/ 配下の各試験専用コピー。自動削除なし

contract記録追加前のout/m1-20261029-a、b、out/m1-write-limit-20261029、
out/m1-validation-observations.jsonも経過証拠として保全しています。上記のfinal付きが最終実装に対応します。

## 5. 依存・ライセンス・既知助言

直接依存はcore、serde、serde_jsonだけです。coreは上流の既存検査を使うため、serde群は型付き入出力のためです。
解決された依存はGit由来のcore 1件とcrates.io由来11件で、追加のdev-dependencyはありません。

| package | 解決版 | manifest上のlicense |
| --- | --- | --- |
| leak-fence-core | 0.1.0 / 固定rev | MIT |
| serde / serde_core / serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| syn | 3.0.9 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| zmij | 1.0.23 | MIT |

これはCargo metadataの表記確認で、ライセンス監査や配布時の表記確認の代替ではありません。
ラボ自身の公開ライセンスは未決定のまま、publish=falseを維持しています。

[RustSec公式DB](https://github.com/RustSec/advisory-db/tree/40f5be6fb12ed93fc591bf4ab23b7d793b3f2239)を取得し、
rev `40f5be6fb12ed93fc591bf4ab23b7d793b3f2239`のcrates/以下1,361ファイルから、全12依存のpackage名を照合しました。
該当する助言ファイルは0件でした。非推奨・保守停止の助言もこの照合では見つかっていません。
各リポジトリの将来の保守継続や、新しい解決版の安定性を保証した結果ではありません。
cargo-auditは未導入だったため実行しておらず、今回の結果は固定DBのpackage名による直接照合です。
coreのような未公開Git依存や、未登録の脆弱性、Rustツールチェーンの助言まで網羅したとは扱いません。

## 6. 残る制約

- HTML、ブラウザ、読み上げ、CSP・escape、JSONとDOMの一致はM2で確認する。
- `--frozen`でCargoの通信取得禁止を確認したが、OS全体でネットワーク遮断した試験ではない。
- 第三者の再実行、別OS・CPU、完全な別マシンでのビルドは未確認。
- fixtureのsynthetic=trueは作者の宣言。秘密不在を機械的に保証しない。
- ローカルcheckoutとホストを信頼する。ファイルの同時差し替えを行う攻撃者への防御は保証しない。
- 日次取得枠、認証・認可の正しさ、稼働APIの遮断、クラウド、本番防御、需要・販売の検証ではない。
- M2以降は未着手。今回の完了は最終ゴール達成とは区別する。

## 7. 隣のリポジトリの状態差

開始時、ローカルLeakFenceには未追跡のdocs/GET_ROUTE_ADOPTION_PLAN_2026-10-15.mdと
docs/PRODUCT_API_SELECTION_2026-10-09.mdがありました。終了時には両パスが存在せず、statusは空でした。
HEADは開始時と同じ`6a0f416e23da5e0cff5890fd05b6b1d9a4162393`です。
この作業で隣のリポジトリに対して行ったのは読取・状態確認だけで、編集・削除・復元はしていません。
状態差が生じた原因は未確認です。固定Git依存のビルド・判定結果には影響していません。
