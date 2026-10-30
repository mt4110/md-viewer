# 設計の根拠と確認範囲

確認日：2026-10-29。本書は設計時の根拠を残します。後続のM1実装と実行検証は[VALIDATION_M1.md](VALIDATION_M1.md)を参照してください。

## 1. 参照した状態

- labの作業開始時HEAD：`81e6282`。追跡ファイルはREADME.mdのみ。開始時の作業ツリーに変更なし。
- ローカルLeakFenceのHEAD：`6a0f416e23da5e0cff5890fd05b6b1d9a4162393`。
- 同じrevの公開coreソースはGitHub接続経由でも取得できることを確認。
- LeakFence側には未追跡文書2つが存在。変更・削除していない。
- 適用範囲の親ディレクトリと両リポジトリルートでAGENTS.md / Agent.md / CLAUDE.md / Claude.mdを確認したが、ファイルは見つからなかった。会話で提示された指示を適用。
- 現在の端末で`rustc 1.98.1`、`cargo 1.98.1`を確認。上流の固定値は1.95.0であり、同じ実行環境とは扱わない。
- Fixture / Reproの具体的な既存APIは未確認。作業ディレクトリ直下の関連名探索から対象を特定できなかったため、導入済みとは記載しない。

公開revでソースを参照できることと、独立checkoutでCargoビルドに成功することは別です。
後者はT01とT09の実装・検証タスクとして残します。

## 2. ソースに基づく事実

リンクは確認した固定revです。ローカルの行番号も同じrevに基づきます。

| 事実 | 根拠 |
| --- | --- |
| Contextは主体・所属・権限・許可ID集合 | [core/lib.rs L13–20](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/core/src/lib.rs#L13-L20) |
| Policyは許可項目・件数・バイト数・日次設定等を含む | [core/lib.rs L22–36](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/core/src/lib.rs#L22-L36) |
| 固定理由は9種類 | [core/lib.rs L45–72](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/core/src/lib.rs#L45-L72) |
| inspectはCostまたはDeniedのみを返し、位置・全検査結果を返さない | [core/lib.rs L129–168](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/core/src/lib.rs#L129-L168) |
| 検査は設定→Context→本文バイト→パース→件数→各行の所属・ID・項目・入れ子の順で最初の失敗を返す | [inspect本体](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/core/src/lib.rs#L129-L168) |
| 重複キーや対応しない文書形はパーサーで拒否 | [独自Deserialize](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/core/src/lib.rs#L170-L232) |
| 既存demoは正常・他テナント・追加項目の3ケースを照合 | [cli/main.rs L67–113](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/cli/src/main.rs#L67-L113) |
| 既存CLIのinspectは日次枠未検査、本文を出力しない | [cli/main.rs L145–173](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/cli/src/main.rs#L145-L173) |
| CLIの終了コードと上限等に回帰試験がある | [cli/tests/inspection.rs](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/cli/tests/inspection.rs) |
| 許可値の意味・出所は判定しない、正しい認可層を信頼する | [THREAT_MODEL.md](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/docs/THREAT_MODEL.md) |
| 上流coreはpublish=false、MIT、edition 2024、rust-version 1.95 | [core/Cargo.toml](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/crates/core/Cargo.toml)、[workspace Cargo.toml](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/Cargo.toml) |
| 実用化の条件と第三者導入は別途確認が必要 | [READINESS.md](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/docs/READINESS.md) |

MAX_BODY_BYTESは262,144、MAX_RECORDSは1,000です。
件数上限の理由は入力形・上流パース上限にも依存します。例えば1,000件を超す配列は、
設定のRecordLimit比較より先にパース段階でInvalidJsonになることがあるため、ラボ側で理由を推測しません。
ラボで件数境界を試す場合は、上流の絶対上限より小さいPolicy上限を使います。

## 3. 設計判断と未決定事項

ソースから決まるのは上記の検査挙動です。
単一package、JavaScriptを使わない画面、5ケース、行注釈、出力先を新規に限定することは今回の設計判断です。
上流が要求している構成ではありません。

Fixture / Reproの実際の資産を統合する必要性、ラボ自身の公開ライセンス、主力プロダクトとの優先順位、
第三者確認の協力者・日程、公開タイミングは未決定です。
これらを未決定のままでも設計は作れますが、統合・公開・他案件の停止を自動実行しません。

## 4. 設計時に実施した確認

- 作業ツリーと適用指示の探索。
- 上流core、CLI、CLI試験、サンプル設定、README、信頼境界、実用化条件、導入ガイドの読取。
- GitHub上の固定revのcoreが参照可能なことの確認。
- Cargo公式のGit依存・lockfile、MDNのCSP資料の参照。
- 作成した文書の差分・相互リンク・JSON例・行注釈・タスク対応の整合性確認。

設計時点ではcargo build / test / clippy、既存demoの再実行、ラボの生成、ブラウザ実画面、
第三者再現、脆弱性監査、クラウド・日次枠・本番防御の検証は実施していません。
試験コードが存在することを、この作業で成功した証拠として扱いません。

## 5. 外部仕様の参照

- [Cargo：Git依存の指定](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories)
- [Cargo：Cargo.tomlとCargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
- [MDN：CSP script-src](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/script-src)

これらの参照は設計の根拠であり、このラボへの監査・認定・動作保証を意味しません。
