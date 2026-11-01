# md-viewer 0.1.1 ソース版

非エンジニアがMarkdownを開いて読むためのRust + Tauriアプリです。
このアーカイブは開発者向けのソースです。v0.1.1では別途、Windows 11 x64の未署名セットアップEXEを試用版として準備しています。
SAC等で実行をブロックされる場合は導入を中止し、OSの保護を無効化しないでください。
導入条件と未検証項目はdocs/markdown-reader/WINDOWS_TRIAL_v0.1.1.mdを参照してください。
Windows・macOSのCIでRust・UI回帰・releaseビルドを確認しています。
所有者はHPのWindows 11 Pro本体で旧未署名x64候補の導入・起動を確認しましたが、権利表示を追加した新候補はSACに開始前に拒否されました。
v0.1.0はソース版のみです。v0.1.1でも未署名MSIX試作・macOSアプリは配布しません。Gitの署名タグとOSのアプリ署名は別です。
IME・読み上げ等の全受入項目、Windowsコード署名、macOSの署名・公証、第三者利用確認は未完了です。
OSの保護機能を無効化して利用することは前提にしません。

## ビルドと検証

Rust 1.95.0、Node.js 24、OSのTauriビルド環境が必要です。
権利表示の検査・ソース梱包にはPython 3.11以降も必要です。
初回の依存取得にはネット接続が必要です。

```sh
cargo build --locked
cargo run --locked -- README.md
npm ci --ignore-scripts
npx playwright install chromium
npm run check:rust
npm run check:docs
npm run test:ui
python tools/generate_notices.py --check
```

ルートからmd-viewerをビルドします。原稿はUTF-8の.md/.markdownを対象とします。
外部URLは確認してから標準ブラウザに渡し、画像やリンク先を自動取得しません。
画像は64個・合計4000万画素・圧縮24MiBまでです。タブ本文のキャッシュは64MiBまでです。
詳細はdocs/markdown-reader/のPLAN.mdとTASKS.md、ルートのSECURITY.mdを参照してください。
アーカイブ内では設計と検証記録はdocs/markdown-reader/にあります。
このアーカイブはmd-viewer向けに対象を絞り、旧ラボのコードと成果物を含みません。GitHub自動生成のSource codeアーカイブはリポジトリ全体であり、内容とライセンスの適用範囲が異なります。

## ライセンス

reader/LICENSEのMITを参照してください。適用範囲はLICENSE-SCOPE.mdで定義しています。
Windows x64候補の第三者表示はTHIRD_PARTY_NOTICES.txt/jsonに含み、同じ内容をreader/resources/noticesに収録しています。
原文の固定取得先・SHA-256を記録し、不足や不一致があれば梱包を失敗させます。MPL対象も、記載した正確な版のcrate URLから対応ソースを取得できます。
第三者ソースとバイナリ自体はこのソースアーカイブへ同梱していません。macOS等のバイナリを公開するときは、その対象の表示を別途確認します。
