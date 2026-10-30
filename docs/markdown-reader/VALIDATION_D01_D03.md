# md-viewer D01〜D03の実装・検証記録

確認日：2026年10月29日。環境はmacOS 27.0.1、Apple Silicon arm64、Rust 1.95.0。
基本実装と下記の確認を実施した。全OS・全完了条件の確認済みという意味ではない。

## 実装したこと

- reader/に独立したCargo package md-viewerを追加。旧ラボのソース・依存を利用しない。
- Tauri 2の起動、OSからのファイルopen、ドロップ受付、ファイル選択、稼働中の追加openを実装。
- RustでMarkdownを解析。日本語、表、脚注、チェックリスト、コード、ローカル画像を表示。
- 見出しの目次、文書内検索、文字サイズ、読み幅、明暗を実装。
- 追加指示により基本タブとローカルリンクの別タブ表示を追加。同一文書は既存タブへ移動。
- リンク切れは小さなstatusとして表示。上位階層・同階層の再帰探索や代替先の推測をしない。
- 操作を閉じたメニューへ集約。検索は⌘F／Ctrl+Fで開き、Escで閉じる。単一文書ではタブ列も隠す。
- 日本語を既定にし、Englishへの切替を実装。画面とネイティブメニューを連動。文書本文は翻訳しない。
- 文字色・背景色、配色プリセット、初期値への復帰を実装。言語と表示設定は端末内へ保存。
- GitHub名とoriginはmd-viewerを確認。ローカルブランチをfeat/md-viewerへ変更。
- ローカル実体をmd-viewerへ移動。旧veil-boundary-labは過去の参照を保つ互換リンク。

## 検証結果

| 確認 | 結果と範囲 |
| --- | --- |
| cargo build --locked | macOS debugビルド成功。Windows向けビルドではない |
| cargo fmt --check | 成功 |
| cargo test --locked | 7テスト成功 |
| cargo clippy --locked --all-targets -- -D warnings | 成功。対象OSはmacOS |
| markdownlint | READMEと新設計一式を検査 |
| Chromium表示・操作 | 1100×800、550×700。目次、文書内検索、拡大、明暗、狭い画面で横にはみ出さないことを確認 |
| 言語と配色 | JA初期値、EN切替、文字色・背景色変更、再読込後の設定復元、初期化を確認。JSエラーなし |
| macOS実アプリ | 開発用.appで起動、起動引数のREADME表示、OSの追加open、ファイル選択と取消を確認 |
| macOSリンク | README→PLANを別タブで表示。入口.md→関連.mdを別タブで表示 |
| macOSリンク切れ | 元の本文を保ち「リンク切れです。」を表示。余分なタブを作らない |
| macOSローカライズ | 画面のJA→EN切替と、ファイル／編集／表示→File／Edit／Viewへの同期を確認 |
| アクセシビリティ | macOSのAXツリーでボタン名、見出し、表、検索欄、色設定の名前を確認。音声読み上げの実地試験とは異なる |

Rustテストは、日本語・重複見出し、HTML／javascriptリンクの無効化、外部画像の拒否、文書サイズ上限、
UTF-8 BOM・不正UTF-8、画像表示、パーセントエンコードされたパス、シンボリックリンクの範囲外参照、
リンク切れ、原稿のバイト列が変わらないことを対象にした。

Browserプラグインが利用できないため、表示試験は同梱Playwright＋Chromium headless 153.0.8010.12で実施。
Rustの実レンダラーが生成したJSONを表示用ページへ渡した。ファイル選択などのネイティブIPCは、この試験では代替していない。
ネイティブIPCは別途macOSアプリ上で確認した。

## 入力と資源の上限

本文2MiB、パースイベント100,000、画像1点8MiB・20メガピクセル、文書内画像合計24MiB。
基本タブは最大20件、保持する表示HTML文字列はUTF-16換算64MiB以内とする。
これは初期の資源制限であり、あらゆる入力での最大メモリ使用量や性能を保証する値ではない。
現在の基本タブはHTMLを上限内で保持する。背景タブの本格的な遅延読込・キャッシュ破棄はD04で整理する。

## 依存確認

Cargo.lockを追加して版を固定した。主な解決版はTauri 2.12.5、tauri-build 2.7.4、
dialog 2.8.4、single-instance 2.5.5、pulldown-cmark 0.13.5、image 0.25.10。
Cargo metadataの462 packageにライセンス情報が存在することを確認した。公開ライセンスの決定や法的審査の代わりではない。

cargo-audit 0.22.2でlockfile全体を照合し、vulnerabilitiesは0件、warningsは5件。
警告を無視する設定は追加していない。

- proc-macro-error：保守終了（RUSTSEC-2024-0370）。
- zbus_names：保守終了（RUSTSEC-2026-0371）。
- zvariant：保守終了（RUSTSEC-2026-0370）。
- zvariant_derive：保守終了（RUSTSEC-2026-0372）。
- glib：健全性の警告（RUSTSEC-2024-0429）。

監査は全OS分の依存を含む。少なくともglibとproc-macro-errorはmacOS対象の依存ツリーには現れなかった。
他のOSでの影響評価や一般配布可否を、この結果だけで確定しない。

## 性能の確認範囲

開発用debugアプリで4文書を表示した後のpsの単一スナップショットでは、
本体116,304KiB、対応するWebContent 47,776KiB、GPU 42,960KiB、Networking 14,656KiBだった。
合計221,696KiB（約216.5MiB）。各プロセスの表示CPUは0.0%だった。
これは瞬間値で、定常負荷の測定・消費電力測定・releaseビルドの性能値ではない。
起動から本文表示までの時間、空画面／前面／背景／最小化の比較、Windows測定は未実施。
軽量・省エネルギーを達成済みとは報告しない。

## 未完了・未検証

- Windowsのビルド、.exe作成、導入、Explorer関連付け、WebView2、Narrator、表示倍率。
- macOSの正式な配布用.app／.dmg作成、署名、公証、Finderの関連付け選択、手動ドロップ操作。
- VoiceOverの音声操作、日本語IMEを使う一連の実地操作、コピーのOSクリップボード確認。
- 厳密な起動時間・定常負荷・消費電力の測定。
- 外部Webリンクをブラウザで開く機能、参照フォルダーの追加許可UI。
- タブの履歴、並行した複数openの順序、再起動時の文書復元、更新通知、設定以外の読書状態保存。
- タブ横断検索。現段階はこの文書内の検索のみ。インライン要素をまたぐ文字列の検索も未対応。
- 第三者の使いやすさ・継続利用確認。

D01・D03は基本実装済みだが全条件は未確認のため、タスク表のチェックは残す。
D02は今回の静的入力・境界確認を完了。D04・D05全体は未完了。
ソースのpush、公開、外部送付、既存成果物の削除は行っていない。

## 再実行

リポジトリのルートで以下を実行する。

```sh
rustup run 1.95.0 cargo test --manifest-path reader/Cargo.toml --locked
rustup run 1.95.0 cargo clippy --manifest-path reader/Cargo.toml --locked --all-targets -- -D warnings
rustup run 1.95.0 cargo run --manifest-path reader/Cargo.toml --locked -- README.md
```

初回取得にはネットワークと各OSのビルド環境が必要。
/tmp/md-viewer-local.appは実機確認用の開発バンドルで、配布用インストーラーではない。
