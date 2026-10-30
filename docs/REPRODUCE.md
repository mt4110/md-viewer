# ソース配布物からの再現手順

対象は合成データ専用のRustデモです。アーカイブとSHA256SUMSを同じフォルダに置いてください。
配布物のハッシュは内容の識別・破損検出用で、送信元の真正性や署名を保証しません。
現在は公開・外部送付前の準備物です。ラボ自体の公開ライセンスは未決定です。

## 前提

- Rust/rustupとRust 1.95.0。未導入なら `rustup toolchain install 1.95.0 --profile minimal`。
- gzip/tar、SHA-256確認コマンド（macOSではshasum）。
- 初回の依存・ツールチェーン取得にはインターネット接続が必要です。
- Cloudflare、Node.js、隣のLeakFence checkoutは不要です。
- ブラウザでローカルHTMLを開けること。

## 展開と実行

SHA256SUMSは配布物と同じディレクトリで確認します。

```sh
shasum -a 256 -c SHA256SUMS
mkdir reproduce-run-001
tar -xzf veil-boundary-lab-source.tar.gz -C reproduce-run-001
cd reproduce-run-001/veil-boundary-lab
rustup run 1.95.0 cargo run --locked -- check
rustup run 1.95.0 cargo run --locked -- build --out out/demo-001
```

既存の展開先・出力先は使わず、新しい名前にしてください。削除してやり直す必要はありません。
checkは `total=5 matched=5 mismatched=0`、終了0が期待結果です。
正常・制約ケースはpassed、他テナントはtenant_mismatch、未許可IDはobject_unauthorized、追加項目はunapproved_fieldです。
拒否が期待どおりなら終了0。不一致は2、実行失敗は1です。

`out/demo-001/index.html`を直接ブラウザで開き、別顧客の行が混ざるケースへ移動してください。
左の所属と中央の混入行、右の拒否理由が対応することを確認します。
強調は作者の注釈であり、検査器が返した位置情報ではありません。
report.jsonには本文・Policy・Contextが保存されます。実データを入れないでください。

依存取得後は`--locked`を`--frozen`に変えて、別の新規出力先へ生成できます。
通信を禁止したブラウザでもHTMLは閲覧できます。外部の導入リンクはクリックしないで確認してください。

## 次に自分のAPIで試す

このlabから `-p leak-fence-cli` は実行できません。
[LeakFenceの固定版導入ガイド](https://github.com/mt4110/leak-fence/blob/6a0f416e23da5e0cff5890fd05b6b1d9a4162393/docs/QUICKSTART.md)を参照し、
別に取得したLeakFenceリポジトリのルートで既存CLIを実行します。
内容検査は認証・認可の正しさ、日次枠、本番APIの遮断を証明しません。

## 詰まったとき

依存取得失敗はネットワークとRust版を、fixture_rootは作業ディレクトリを確認してください。
output_existsは別の出力先を指定します。途中失敗した成果物を成功結果と扱わないでください。
本文や認可情報を含むエラー抜粋を第三者へ送らず、固定エラーコードを記録してください。
[確認記録様式](REPRODUCTION_RECORD.md)へ結果と介入を記録します。
