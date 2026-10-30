# データとCLIの契約

状態：M2まで実装済み。buildはreport.jsonとindex.htmlを保存します。M1の記述は過去段階の説明です。
型の実装はsrc/、実際の検証は[VALIDATION_M1.md](VALIDATION_M1.md)を参照します。
設計判断は[DESIGN.md](DESIGN.md)、実装順序は[TASKS.md](TASKS.md)を参照します。

## 1. 共通規則

- JSONのschema_versionは整数`1`。未対応版・未知フィールドは実行失敗にする。
- ケースIDは`[a-z0-9-]{1,64}`、重複を拒否。suiteの列挙順が表示・実行順。
- デモ本文はUTF-8の原本を保持する。`Value`へ変換・再シリアライズしてからinspectへ渡さない。
- fixtureのJSONメタデータには重複キーを許可しない。caseの型付きDeserializeで未知・重複フィールドを拒否する。
- PolicyとContextの検証は上流型と上流のvalidateへ委ね、ラボで認可規則を再実装しない。
- 本文の重複キー等はラボが事前修正せず、そのまま上流に渡す。
- fixture読込は通常ファイルに限定する。相対パスのみ許可し、`..`・絶対パス・symlinkでfixtureルート外へ出ない。
- 読込時に上限+1バイトまでを読み、metadata上の長さだけを信頼しない。

ラボの運用上限はsuite 16ケース、メタデータ各64KiB、Policy配列128KiB、Context64KiB、本文は上流の`MAX_BODY_BYTES + 1`です。
上流最大値を1バイト超える回帰入力まで扱えます。これを超えるfixtureはラボの入力エラーです。
これらはデモを小さく保つための上限で、LeakFenceの一般的な対応範囲や性能保証ではありません。

## 2. suite.json

```json
{
  "schema_version": 1,
  "id": "boundary-demo",
  "version": 1,
  "policy_file": "policy.json",
  "contract": "sample.customers",
  "context_file": "context.json",
  "cases": [
    "cases/allowed/case.json",
    "cases/foreign-tenant/case.json",
    "cases/unauthorized-id/case.json",
    "cases/extra-field/case.json",
    "cases/value-not-classified/case.json"
  ]
}
```

パスはすべて`fixtures/demo/`を基準とします。`response_file`も同じ基準で、case.json基準ではありません。
Policyは既存CLIと同じ配列形式を読み、`validate_policies`後にcontractをIDで一意に選択します。
存在しないcontract・構造不備は実行失敗です。内容拒否として数えません。
suiteと入力は実行開始時に一度読み、判定・表示・保存の途中で再読込しません。

## 3. 共通の合成認可情報

以下は実装した5ケースに使う共有設定です。fixtures/demo/で原本を管理しています。

```json
[
  {
    "id": "sample.customers",
    "budget_group": "customer-read",
    "permission": "customer:read",
    "fields": ["id", "tenant_id", "name"],
    "tenant_field": "tenant_id",
    "id_field": "id",
    "max_records": 10,
    "max_bytes": 16384,
    "daily_records": 50,
    "daily_bytes": 262144
  }
]
```

```json
{
  "principal": "synthetic-reader",
  "tenant": "synthetic-acme",
  "permission": "customer:read",
  "record_ids": ["1", "2"]
}
```

正常応答は`id=1`、所属`synthetic-acme`、nameは`合成顧客A`です。
他テナント混入は正常な1行の後へ`id=2`、所属`synthetic-other`、name`合成顧客B`を追加します。
未許可IDは正常応答のIDだけを`3`へ変更します。
追加項目は正常応答へ`internal_note: "合成の社内メモ"`を追加します。
制約ケースは正常応答のnameを`SYNTHETIC_VALUE_NOT_CLASSIFIED`へ置き換えます。
この値が秘密かどうかを機械判定したケースとは呼びません。

## 4. case.jsonと説明用注釈

```json
{
  "schema_version": 1,
  "id": "foreign-tenant",
  "title": "別顧客の行が混ざる",
  "kind": "boundary",
  "synthetic": true,
  "response_file": "cases/foreign-tenant/response.json",
  "expected": {"content_check": "denied", "reason": "tenant_mismatch"},
  "lesson": "項目とIDが許可済みでも、所属が異なる行を含むため応答全体を拒否します。",
  "highlights": [
    {
      "id": "tenant-comparison",
      "source": "context",
      "start_line": 3,
      "end_line": 3,
      "expected_text": "  \"tenant\": \"synthetic-acme\",",
      "label": "この利用者に許可された所属"
    },
    {
      "id": "tenant-comparison",
      "source": "response",
      "start_line": 9,
      "end_line": 9,
      "expected_text": "    \"tenant_id\": \"synthetic-other\",",
      "label": "混入した行の所属"
    }
  ]
}
```

上記のresponseは次の行構成を前提にします。原本はLF区切りで追跡します。

```json
[
  {
    "id": "1",
    "tenant_id": "synthetic-acme",
    "name": "合成顧客A"
  },
  {
    "id": "2",
    "tenant_id": "synthetic-other",
    "name": "合成顧客B"
  }
]
```

`kind`は`boundary`か`limitation`です。制約ケースの許可を問題の解決と表現しません。
`synthetic`はtrue必須ですが、秘密不在の自動証明ではありません。
Expectedはタグ付きの`passed`、または`denied + reason`だけとします。
passedにreasonがある、deniedにreasonがない、未知のreason等は構造不備です。

注釈のsourceは`policy | context | response`、行番号は1始まり・両端を含みます。
対象範囲をLFで結合し、最終LFを含まない文字列と`expected_text`が完全一致することを確認します。
0行、範囲外、逆順、同一source内の重複・交差範囲を拒否します。
注釈idも`[a-z0-9-]{1,64}`に限定します。同じidは異なるsource間を結ぶために使えますが、同じsource内では一意にします。
DOM IDは固定接頭辞に`case-id/source/highlight-id`を連結します。IDの各要素にスラッシュを許可しないことで衝突を防ぎます。

注釈の整合性検証は「作者が指定した行が変わっていない」ことだけを保証します。
注釈の意味や因果の正しさは、単一原因のfixtureと人のレビューで確かめます。
通常の許可ケースと位置を説明しない回帰入力ではhighlightsを空にできます。

## 5. Rustの責務と型の目安

以下は責務を示す型の目安です。実際の型はsrc/にあり、内容判定と期待照合の区別を保っています。

```rust
enum ContentCheck {
    Passed { cost: Cost },
    Denied { reason: DenialCode },
}

enum ExpectationCheck {
    Matched,
    Mismatched,
}

struct CaseResult {
    // 入力・期待値・説明はLoadedCaseから一度だけコピーする
    input: LoadedCase,
    actual: ContentCheck,
    expectation_check: ExpectationCheck,
    highlights_enabled: bool,
}
```

`Cost`は上流の型またはrecords/bytesだけのDTOです。
`DenialCode`は上流Deniedの9変種に対応し、`reason.code()`を保存します。
Deniedからの変換は網羅matchにし、上流に変種が増えればコンパイルで気付けるようにします。
LabErrorは入力・読込・注釈・保存の失敗を担当し、ContentCheckへ混ぜません。
LabErrorの表示は固定コードとケースID等に限定し、serdeエラーの本文抜粋は出しません。

最終MVPで使う主要関数の目安（render_htmlはM2で追加）：

```text
load_suite(root) -> Result<LoadedSuite, LabError>
evaluate_case(body_bytes, policy, context) -> ContentCheck
evaluate_suite(loaded_suite) -> SuiteReport
render_html(&SuiteReport) -> Result<String, LabError>
build_report(&SuiteReport, new_output_dir) -> Result<ArtifactPaths, LabError>
```

inspectの呼出しはengine.rsだけに置きます。render.rsへPolicy/Contextの認可ロジックを渡しません。
`highlights_enabled`は`expectation_check == matched`かつ注釈検証済みの場合に限ります。
実判定から理由文の見出しを選び、fixtureのlessonは「ケースの説明」として別に表示します。
実判定が変わっても、古いlessonを新しい検出理由として表示しません。

## 6. report.json

単一の`SuiteReport`をserializeし、同じ値をHTMLへ渡します。HTML側でJSONを読み直したり判定したりしません。

必須フィールド：

| フィールド | 契約 |
| --- | --- |
| `schema_version` | 1 |
| `suite` | id、version、検査に選択したcontract。Policy配列が複数契約でも再現できるようにする |
| `generator` | package名とpackage version |
| `engine` | name=`leak-fence-core`、repo URL、固定git_rev |
| `scope` | `content_only=true`, `budget_check="not_performed"`, `network="not_used"`, `data="synthetic"` |
| `summary` | total、matched、mismatched。内容の拒否件数とは区別 |
| `cases` | suite順のCaseResult。原本Policy・Context・本文、説明、期待値、実判定、照合結果、検証済み注釈 |

原本Policy・Context・本文はUTF-8文字列として保存します。Bodyの空白・改行・末尾LFも保持します。
許可結果は`{"content_check":"passed","cost":{"records":1,"bytes":整数}}`、
拒否結果は`{"content_check":"denied","reason":"tenant_mismatch"}`の形にします。
bytesはinspectが返した実値で、文字数ではありません。拒否結果へcostを追加しません。

実行日時、ホスト名、絶対入力パス、乱数IDをレポートへ入れず、同じ入力と固定依存で同じ出力を狙います。
出力先名も内容へ埋め込みません。日時が必要なら外側の実行記録に分離します。
コアのgit_revは実際のCargo解決結果と一致させ、CIの`cargo metadata --locked`で確認します。
M1ではCIは追加しておらず、`cargo metadata --frozen`を使うローカル統合試験で一致を確認しています。
reportには入力一式が残るため再現材料になりますが、署名・改ざん検知・真正性は保証しません。
生成後のHTMLが、現在の編集内容を再検査した結果かどうかは自動追跡しません。

## 7. CLIと終了コード

この節の2ファイル保存は最終MVPの契約です。M1時点では同じ終了コードでreport.jsonのみを保存し、helpと標準出力に中間実装と表示します。

```text
veil-boundary-lab check
veil-boundary-lab build --out <存在しないディレクトリ>
veil-boundary-lab --help
```

コマンドはリポジトリルートの`fixtures/demo`を読みます。cwd依存はhelpに明記します。
独立した配布バイナリ、任意suite指定、stdin入力、上書き用のforceフラグは今回作りません。
`check`は判定・照合・注釈検証を実施して要約だけをstdoutへ出し、成果物を保存しません。
`build`は同じ処理の後にreport.jsonとindex.htmlを保存し、要約と出力先だけをstdoutへ出します。

| 終了コード | 意味 | 成果物 |
| --- | --- | --- |
| 0 | 全ケースが期待と一致、必要な出力処理も完了 | buildなら2ファイル保存 |
| 2 | 実判定と期待結果に不一致あり | buildなら不一致表示付きの2ファイル保存 |
| 1 | 引数・入力構造・注釈・読込・保存等の実行失敗 | 保存失敗時は部分出力があり得る |

拒否が期待どおりなら終了0です。上流CLIの「内容拒否=2」と同じ意味ではありません。
不一致と保存失敗が両方あれば、保存失敗の1を優先します。
標準出力には`total / matched / mismatched`とケースID・固定理由までとし、原本文字列は出しません。
stdout/stderrの書込失敗も成功扱いにしません。

既存の出力先があれば何も変更せず1で終了します。末尾のファイル名だけでなく出力ディレクトリ自体の存在を確認します。
競合に備えてディレクトリ作成と各ファイルの`create_new`も使い、単なる事前exists確認に頼りません。

## 8. 自分のAPIへの案内

CTAは固定URLへの通常リンクです。自動アクセスやブラウザへの入力欄は不要です。
文言は「自分の認可情報と応答を検査する：LeakFence CLIの導入例へ」。
リンク先は[EVIDENCE.md](EVIDENCE.md)の固定revのQUICKSTARTとします。
紹介する既存コマンドは、LeakFenceリポジトリのルートで実行する次の形式です。

```sh
cargo run --locked -p leak-fence-cli -- inspect examples/contracts.json sample.customers examples/inspection/context.json examples/inspection/allowed.json
```

labのfixturesをそのまま実データ向けの保存レポートに変える案内はしません。
