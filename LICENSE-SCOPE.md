# ライセンスの適用範囲

md-viewer本体のreader/、docs/markdown-reader/と、新規のビルド・CI・テスト・梱包コードは[MIT](reader/LICENSE)です。
旧ラボのsrc/・tests/・assets/・fixtures/・README.lab.md・旧文書と配布成果物へ、この許諾を一律適用しません。
第三者コード・依存ライブラリには、それぞれのライセンスが適用されます。
Windows x64配布候補は、製品のMIT本文をLICENSE.txt、依存の本文・著作権・NOTICE・対応ソースの取得先をTHIRD_PARTY_NOTICES.txt/jsonとして同梱します。
一覧にはWindows向けの固定依存（ビルド用を含む）と、静的リンクされるMicrosoft WebView2ローダーの表示を収録します。全項目が実行時にリンクされるという意味ではありません。
tools/generate_notices.py --checkでCargo.lockとの整合を検証します。依存パッケージに本文がない場合は、固定リビジョンの公開元から取得してハッシュを記録したtools/licensesの原文を使います。
MPL対象crateは、バージョン固定の原文アーカイブURLとSHA-256から対応ソースを取得できます。第三者コードを製品のMITとして再許諾しません。
macOS の配布用アプリは、Apple Silicon と Intel 向けの固定 Cargo 依存（ビルド用を含む）を `THIRD_PARTY_NOTICES.txt` と `THIRD_PARTY_NOTICES.json` として同梱します。フォーマッターの第三者表示は `MARKDOWNLINT-NOTICES.txt` に収録しています。
macOS 向け通知は `node reader/packaging/generate-macos-notices.mjs --check` で検証できます。ロック更新後は `--check` を外して再生成し、差分を確認します。
`objc2` 系では、固定版のライセンス宣言と、同じライセンス条件を本文として追加した上流の固定コミットを記録しています。Apple SDK 由来のバインディングについての上流注記も収録しています。
