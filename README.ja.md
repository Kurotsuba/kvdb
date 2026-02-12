# kvdb - Kurotsuba's (Kantan) Vector Database

Rustで一からベクトルデータベースを実装する学習プロジェクトです。**Kantan**（簡単）の名の通り、本番環境の複雑さを排除し、ベクトルデータベースの核心技術を理解することに焦点を当てています。

## エグゼクティブサマリー

kvdbは、**RAG（Retrieval-Augmented Generation）システムの基盤技術**であるベクトルデータベースを、ゼロから実装して学ぶための教育プロジェクトです。社内で計画されているRAGプロジェクトで使用される技術スタックを理解し、実務に必要な知識を習得することを目的としています。

| 項目 | 内容 |
|------|------|
| **目的** | 社内RAGプロジェクトに必要なベクトルDB技術の習得、Rustの練習 |
| **技術スタック** | Rust, Python（ベンチマーク） |
| **現状** | v2.0完成（永続化実装済み、38個のテストパス） |
| **成果物** | 動作するベクトルDB、永続化、BERTセマンティック検索のデモ |

**現在のバージョン**: v2.0 - bincode形式によるデータ永続化

---

## RAGシステムにおけるベクトルデータベースの役割

**RAG（Retrieval-Augmented Generation）** とは、大規模言語モデル（LLM）に外部の知識データベースを組み合わせて、より正確で最新の情報を提供する技術です。

```
ユーザーの質問
    |
[1] 質問をベクトルに変換（Embedding）
    |
[2] ベクトルデータベースで類似文書を検索  <-- このプロジェクトで学ぶ部分
    |
[3] 関連文書をLLMに渡して回答生成
    |
回答を返す
```

kvdbは、この[2]の検索エンジン部分を実装したものです。

### ビジネスでの活用例

**RAGシステムでの利用**:
- 「契約書の中から、この質問に関連する条項を探す」
- 「過去の問い合わせから、類似の事例を検索する」
- 「製品マニュアルから、ユーザーの質問に最も関連する部分を抽出」

**その他のAIアプリケーション**:
- 「この画像に似ている商品画像を探す」（ECサイト）
- 「この文章に近い意味の文章を見つける」（検索エンジン）
- 「このユーザーの行動履歴に基づいてレコメンド」（推薦システム）

従来のキーワード検索と異なり、**「意味」の類似性**で検索できるのが特徴です。

---

## 機能

- **L2ベクトル正規化**: 挿入時に自動で正規化
- **コサイン類似度検索**: 正規化ベクトルのドット積による高速検索
- **フラット配列ストレージ**: キャッシュ効率の良い連続メモリ配置
- **永続化**: bincodeバイナリ形式によるディスクへの保存・読み込み
- **ライブラリファースト設計**: コアロジックとインターフェースの分離
- **テスト**: ユニットテスト38個 + エンドツーエンド永続化テスト

## ライブラリとしての使い方

```rust
use kvdb::VecDB;

let mut db = VecDB::new();

// ベクトルを追加（自動的にL2正規化されます）
db.insert("doc1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
db.insert("doc2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
db.insert("doc3".to_string(), vec![0.7, 0.7, 0.0]).unwrap();

// 類似ベクトルを検索
let results = db.search(vec![1.0, 1.0, 0.0], 2).unwrap();
for (id, _vector, score) in results {
    println!("{}: 類似度 = {:.4}", id, score);
}

// IDで取得
let vec = db.get("doc1").unwrap();

// 削除
db.delete("doc2").unwrap();

// 件数
println!("{} vectors", db.count());

// ディスクに保存
db.save("my_database.db").unwrap();

// ディスクから読み込み
let db = VecDB::load("my_database.db").unwrap();
```

## CLI / REPL

コマンドラインインターフェースで対話的に操作できます。

### REPLモード
```bash
./target/release/kvdb

kvdb> insert vec1 1.0 0.0 0.0
kvdb> insert vec2 0.0 1.0 0.0
kvdb> search 0.7 0.7 0.0 --k_top 2
kvdb> save my_data.db
kvdb> load my_data.db
kvdb> count
kvdb> get vec1
kvdb> delete vec1
kvdb> list
kvdb> help
kvdb> exit
```

### シングルコマンドモード
```bash
# 使い方: kvdb <DBパス> <コマンド> [引数...]
# DBを自動読み込み（存在しなければ新規作成）、コマンド実行後に自動保存

./target/release/kvdb data.db insert vec1 1.0 2.0 3.0
./target/release/kvdb data.db search 1.0 2.0 3.0 --k_top 5
./target/release/kvdb data.db count
./target/release/kvdb data.db list
```

## アーキテクチャ

```
src/
├── lib.rs       # 公開API（VecDB）
├── vector.rs    # ベクトル演算（L2正規化、ドット積）
├── db.rs        # データベース本体 + 永続化
├── cli.rs       # CLI解析、REPL、コマンド実行
└── main.rs      # エントリーポイント
```

### ストレージ
- 全ベクトルを連続配列で格納: `[v1_d1, v1_d2, ..., v2_d1, v2_d2, ...]`
- 並列ID配列: `["vec1", "vec2", ...]`
- キャッシュ局所性に優れたメモリレイアウト
- 将来のSIMD最適化に対応した構造

### 永続化
- serdeによるbincodeバイナリシリアライゼーション
- バッファードI/Oによる効率的なファイル操作

## パフォーマンス

### 計算量
| 操作 | 時間 | 空間 |
|------|------|------|
| Insert | O(d) | O(d) |
| Search | O(n*d) | O(k) |
| Get | O(n) | O(d) |
| Delete | O(n*d) | O(1) |
| Save | O(n*d) | O(1) |
| Load | O(n*d) | O(n*d) |

n = ベクトル数、d = 次元数、k = top_k結果数

### ベンチマーク（10万ベクトル、768次元）
```
挿入:   ~12,000 inserts/秒
検索:   ~22 searches/秒（全件探索）
保存:   ~0.3秒（294 MB）
読込:   ~0.2秒
```

## サンプル

`examples/`ディレクトリにデモ用スクリプトがあります:

- `gen_demo.rs` - 10万個のランダム768次元ベクトルを生成しdemo.dbに保存
- `demo_operations.rs` - demo.dbに対して検索・挿入・削除の操作
- `embed_wikipedia.rs` - BERTモデル（BAAI/bge-base-en-v1.5, 768次元）でWikipedia記事を埋め込み
- `demo_semantic_search.rs` - wikipedia.dbに対するセマンティック検索CLI
- `fetch_wikipedia.py` - Wikipedia記事10万件（タイトル＋説明）の取得

```bash
cargo run --release --example gen_demo
cargo run --release --example demo_operations
cargo run --release --example demo_semantic_search -- "famous physicist"
```

## ロードマップ

### 完了
- [x] CRUD操作
- [x] L2正規化 + コサイン類似度
- [x] REPL・CLIモード
- [x] 永続化（bincodeシリアライゼーション）

### 予定

**v3.0 - HTTP API**
- [ ] Axum/ActixによるREST API
- [ ] JSONリクエスト/レスポンス
- [ ] 並行リクエスト処理

**v4.0 - 性能最適化**
- [ ] HNSWインデックス
- [ ] Product Quantization（積量子化）
- [ ] SIMD高速化ドット積
- [ ] Rayonによる並列検索
- [ ] メモリマップドファイル対応

---

## 社内プロジェクトへの応用

本プロジェクトで学んだ知識は、以下の形で社内のRAGプロジェクトに活かせます:

**システム設計時**:
- ベクトル次元数、インデックスタイプの選択根拠を理解できる
- 性能要件（レイテンシ、スループット）と実装方式の関係がわかる

**運用時**:
- パフォーマンス問題発生時に、原因の切り分けができる
- 適切なチューニング方法を判断できる

**技術選定時**:
- Qdrant、Milvus、Weaviateなどの製品を、内部構造を理解した上で比較評価できる

### このプロジェクトで習得できるスキル

1. **AIアプリケーションの基盤技術** - ベクトル埋め込み、類似度計算、RAGアーキテクチャ
2. **データ構造とアルゴリズム** - フラット配列ストレージ、キャッシュ局所性
3. **Rustシステムプログラミング** - 所有権、ゼロコスト抽象化、シリアライゼーション
4. **ソフトウェア設計** - ライブラリファーストアーキテクチャ、段階的な機能拡張

---

## 用語解説

| 用語 | 説明 |
|------|------|
| **ベクトル** | 数値の並び（例: [1.0, 2.0, 3.0]）。AIが文章や画像を数値化したもの |
| **正規化** | ベクトルの長さを1に揃えること。方向だけで比較できるようにする前処理 |
| **類似度** | 二つのベクトルがどれくらい似ているかを表す数値（1に近いほど類似） |
| **永続化** | データをディスクに保存し、プログラム再起動後も利用可能にすること |
| **Embedding** | テキストや画像をベクトルに変換する処理 |
| **RAG** | 外部知識データベースとLLMを組み合わせて回答精度を向上させる技術 |

## ライセンス

MITライセンス - Copyright (c) 2026 Kurotsuba

詳細は [LICENSE](LICENSE) を参照してください。

---

*Kantan（簡単）- シンプルな実装から、深い理解へ。*
