# M3 / T09の引渡し準備・自己再現

2026-10-29。今回の許可範囲は配布物・再現手順・記録様式の作成と別配置の自己再現まで。
公開・push・外部送付・第三者への連絡は行っていない。M3全体とT09全体は未完了。

## 作成物と完了条件

| 作成物・条件 | 結果 |
| --- | --- |
| 必要なソース一式 | tools/package_source.pyで選択。README、Cargo.toml/lock、toolchain、src/assets/fixtures/tests/docs/tools、.gitignore |
| 不要データを含めない | .git、target、out、Cargoキャッシュ、ローカル実行記録・試験コピーは除外 |
| 固定した内容の識別 | アーカイブSHA-256は外側のSHA256SUMS。内部SOURCE_MANIFEST.jsonは各ソースのSHA-256 |
| 再現手順 | REPRODUCE.md。初回通信、前提、展開、作業ディレクトリ、check/build、期待値、出力と失敗を記述 |
| 記録様式 | REPRODUCTION_RECORD.md。自己確認/第三者確認、介入と理解確認を区別。未記入のまま保持 |
| 別配置の自己再現 | 同じMacの新しい一時ディレクトリ、空のCARGO_HOMEと独立したtargetで取得・ビルド・実行成功 |

アーカイブはソーススナップショットで、インストーラー・署名済みバイナリではない。
ハッシュは改変・破損の検出と内容識別用で、真正性を保証しない。ラボの公開ライセンスは未決定。

## 自己再現で実施した確認

- 配布アーカイブのSHA-256、内部manifestの全ソース、通常ファイルだけの構成を照合。
- 隣のleak-fence checkoutが存在しない新しい配置へ展開。個人の絶対path依存なし。
- Rust 1.95.0は既存のインストールを使用。Cargoの依存キャッシュは空から取得。
- cargo test --locked：23件成功。
- cargo run --locked -- check：終了0、total=5 matched=5 mismatched=0。
- cargo run --locked -- build --out out/first：終了0、report.jsonとindex.html生成。
- cargo run --frozen -- build --out out/second：終了0。2回のJSON/HTMLはそれぞれバイト一致。
- 最終の配布物はout/m3-handoff-final-2/。元の準備アーカイブと展開先・実行ログも保全。

機械記録はout/m3-self-reproduction.json（初回）とout/m3-final-reproduction.json（最終配布物）に残す。
これらは配布アーカイブへ含めない。最終配布物のハッシュはSHA256SUMSを正本とする。

## 残る確認

同じMac・既存Rustツールチェーンでの確認であり、別マシン・別OS・第三者の結果ではない。
認証・認可、本番防御、日次枠、導入効果の証明でもない。

第三者確認時には、本人の実行結果と次の説明を記録する必要がある。
別テナントの越境、同一テナントでも未許可IDを拒否する理由、検査対象外、自分のAPIで次に使うCLI。
今回、それらの回答を捏造・代筆して完了扱いにしていない。
