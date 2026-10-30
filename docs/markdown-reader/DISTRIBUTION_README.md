# md-viewer 0.1.0 ソース配布候補

非エンジニアがMarkdownを開いて読むためのRust + Tauriアプリです。
これは開発者向けのソース配布候補です。利用者向けインストーラーではありません。
Windows実機・署名済みインストーラー・公証・第三者確認は未完了です。

## ビルドと検証

Rust 1.95.0、Node.js 24、OSのTauriビルド環境が必要です。
初回の依存取得にはネット接続が必要です。

```sh
cargo build --locked
cargo run --locked -- README.md
npm ci --ignore-scripts
npx playwright install chromium
npm run check:rust
npm run check:docs
npm run test:ui
```

ルートからmd-viewerをビルドします。原稿はUTF-8の.md/.markdownを対象とします。
外部URLは確認してから標準ブラウザに渡し、画像やリンク先を自動取得しません。
画像は64個・合計4000万画素・圧縮24MiBまでです。タブ本文のキャッシュは64MiBまでです。
詳細はdocs/markdown-reader/のPLAN.mdとTASKS.md、ルートのSECURITY.mdを参照してください。
アーカイブ内では設計と検証記録はdocs/markdown-reader/にあります。

## ライセンス

reader/LICENSEのMITを参照してください。適用範囲はLICENSE-SCOPE.mdで定義しています。
第三者依存の一覧と取得できた権利表示はTHIRD_PARTY_NOTICES.jsonに含みます。
第三者ソースとバイナリは同梱していません。将来のバイナリ配布では対象依存の権利表示を改めて確認します。
