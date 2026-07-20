# unravel-nl

`unravel-nl` は、非公式・曖昧な自然言語の数量表現を決定論的に正準値へ変換し、
さらに人が読める文へ戻すための Rust ライブラリです。

英語版ドキュメント: [README.md](README.md)

## 保証

- **決定論的**: 同じ入力と同じコンテキストからは常に同じ結果が出ます。乱数も推論モデルも
  使わず、ホストの時計やロケール環境も読みません。
- **no-panic**: 公開 API は panic しない方針で書かれています。読めなかった入力は
  unwind ではなく `findings` として返ります。
- **黙って捨てない (No-Silent-Loss)**: 読めなかった断片・曖昧な解釈・近似は
  `findings` として必ず表面化します。
- **勝手に決めない**: 複数の読みが成り立つときは、競合する読みを捨てずに
  `alternatives` として返します。
- **no-I/O・ゼロ実行時依存**: 既定の計算経路はファイル・ネットワーク・システム時計を読みません。

対応する入力の例:

- 日本の尺貫法・建築系の単位（`5尺3寸`、`6帖`、`1坪`、`4畳半`、`1間半`）
- 面積・寸法（`延床100㎡`、`幅3m×奥行4m`、`寸法3640`、`壁厚105mm`）
- 範囲（`100-120㎡`、`2〜3日`、`between 5 and 10 kg`）
- ロケール別の数値書式（`1.234,56 kg`、`1 234,56 m`、`1,23,456 kg`、`1万2345`、`3.5万円`）
- 全角・互換文字の正規化（`５尺３寸`、`１．５ｍ`、`２㎞`、`百二十平米`）
- 長さ・質量・面積・時間・体積・速度・データ量・圧力・電力・照度など多数の単位
- 期間・時刻・スロット（`1h30`、`PT1H30M`、`3pm`、`14:30`、`3pm-4pm`）
- 繰り返し表現（`every monday`、`毎週月曜日`、`毎月第2月曜日`）を RRULE 形式へ正規化
- 近似・公差・上下限（`約20kg`、`10 ± 0.5 mm`、`10mm以下`、`a few minutes`）
- 通貨（`USD 12.34`、`¥1,234`、曖昧な `$12`）
- 温度（`20°C`、`68 F`、`摂氏20度`）
- 単位のタイポ訂正（`5 meterz` → `m`）

## インストール

```sh
cargo add unravel-nl
```

`Cargo.toml` に直接書く場合:

```toml
[dependencies]
unravel-nl = "0.1"
```

対応する最小 Rust バージョン: **1.88**（2024 edition・let-chains 使用）。

### フィーチャーフラグ

| フィーチャー | 既定 | 内容 |
| --- | --- | --- |
| _(なし)_ | 有効 | コアのパースと humanize。I/O なし・実行時依存なし。 |
| `dates-jiff` | 無効 | `jiff` によるカレンダー演算と相対日付（`next friday`、`来週金曜日`）。 |
| `timezones-jiff` | 無効 | IANA タイムゾーン対応。ただし解決には明示的な `reference_date` が必須です（日付なしにゾーンのオフセットは定まらないため）。指定がなければ `3pm Europe/Paris` は既定ビルドと同じく `IssueCode::TimezoneUnsupported` として報告され、`best` は `None` になります。`dates-jiff` を含みます。 |
| `wasm` | 無効 | ブラウザ / Node 向けの `wasm-bindgen` エクスポート。詳細は [docs/wasm.md](docs/wasm.md)。 |

## 使い方

```rust
use unravel_nl::{parse, humanize, HumanizeCtx, Locale, ParseCtx};

let parsed = parse(
    "5尺3寸",
    Some(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    }),
);

let best = parsed.best.expect("a canonical reading");
assert_eq!(best.unit.as_deref(), Some("m"));
assert_eq!(
    humanize(&best, Some(HumanizeCtx { locale: Some(Locale::Ja) })),
    "5尺3寸 (approx.)"
);
```

### エントリポイントの使い分け

`parse()` は総合入口です。入力が数量・日付・範囲・繰り返し・単位換算・素の数値の
どれになるか分からない場合に使います。UI 側でフィールドの型が既に分かっている場合は、
より狭い入口を使うと速く、誤検出も減ります。

- `parse_quantity_fast()` — 数量のみ
- `parse_number_fast()` — 素の数値のみ
- `parse_date_fast()` — 日付のみ
- `parse_recurrence_fast()` — 繰り返し表現のみ
- `parse_all()` — 文中から複数の値をバイトスパン付きで抽出
- `parse_dimensions_for_editor()` — 寸法・面積のみを対象とするエディタ向けスキャナ
- `complete_readings()` — 補完 UI 向けの順位付き候補

### 文中からの複数値抽出

```rust
use unravel_nl::{parse_all, Dimension, Locale, ParseCtx};

let matches = parse_all(
    "延床100㎡、敷地面積120㎡、高さ3.5m",
    Some(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    }),
);

assert_eq!(matches.len(), 3);
assert_eq!(matches[0].text, "延床100㎡");
```

寸法しか受け付けないエディタ欄では、専用スキャナを使います。通貨・日付・一般文法を
避けつつ、日本の建築単位を保ったまま長さと面積だけを拾います。

```rust
use unravel_nl::{parse_dimensions_for_editor, Locale, ParseCtx};

let matches = parse_dimensions_for_editor(
    "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640",
    Some(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    }),
);

assert_eq!(matches.len(), 4);
```

### 日付

日付のパースには `dates-jiff` フィーチャー（既定は無効）が必要です。有効にしないと、
`next friday` / `来週金曜日` / `05/06/2026` / `明天` / `下周五` はいずれも読み取れず、
findings が返ります。暗黙の「今日」を基準に推測することはありません。

相対日付にはさらに明示的な `reference_date` が要ります。コアパーサはホストの
システム時計もタイムゾーンも読みません。

```rust
use unravel_nl::{parse, Date, Locale, ParseCtx};

let parsed = parse(
    "来週金曜日",
    Some(ParseCtx {
        locale: Some(Locale::Ja),
        reference_date: Date::new(2026, 7, 19),
        ..ParseCtx::default()
    }),
);

assert_eq!(parsed.best.unwrap().date.as_deref(), Some("2026-07-24"));
```

明示的なオフセットや既知の固定略号を伴う時刻（`3pm EST`、`9:30 JST`）は UTC 秒に
正規化されます。`Europe/Paris` のような IANA タイムゾーン名は、`timezones-jiff`
フィーチャーと明示的な `reference_date` の両方が揃ったときにだけ解決されます。
日付が与えられなければゾーンのオフセットは定まらないため、`3pm Europe/Paris` は
黙って推測されるのではなく `IssueCode::TimezoneUnsupported` として報告され、
`best` は `None` になります。これは `timezones-jiff` を有効にしたビルドでも同じです。

### 厳密さ (strictness)

`Forgiving` / `Confirm` / `Strict` の 3 モードで、タイポ訂正や近似値の扱いを制御できます。
`Confirm` では自動訂正せず、`suggestions` に候補を返します。

### WASM / ブラウザ

```sh
wasm-pack build --target web --out-dir pkg -- --features wasm
wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm
node tests/wasm_node_smoke.mjs
```

`web/unravel-adapters.js` に、依存ゼロの ESM アダプタ（DOM 入力、スパン保持の
`parseAllForUi()`、React 連携、カスタム要素ラッパ）が入っています。TypeScript
型定義は `web/unravel-adapters.d.ts` です。詳細は [docs/wasm.md](docs/wasm.md)。

## 開発

```sh
make lint           # cargo fmt --check + clippy -D warnings
make test           # cargo test --all-features
make test-default   # cargo test          （多くの利用者が使うビルド）
make test-dates     # cargo test --features dates-jiff
make test-timezones # cargo test --features timezones-jiff
make test-wasm-lib  # cargo test --features wasm （WASM 版が実際に使う feature 構成）
make test-wasm      # wasm-pack ビルド + Node / ブラウザアダプタのスモークテスト
make web-test       # TypeScript 型定義の型チェック
make check          # lint test test-default test-dates test-timezones test-wasm-lib
```

`make check` は `--all-features` だけに頼らず各 feature 構成を個別に実行します。
片方の構成でしか到達しないコードで実際にバグが出たことがあるためです。実行される
のは `lint` と上記 5 つの cargo テストレーンだけで、`make test-wasm` と
`make web-test` は含まれません。

`make test-wasm` には [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) と
Node.js が必要です。`make test-wasm` と `make web-test` はどちらも事前に `web/` で
`npm install` が必要です（React アダプタのスモークテストが `web/node_modules` から
React を読み込むため）。
どちらも `make check` とは別に自分で実行してください。

## 帰属

公開 API の方向性は `pascalorg/lingo` (MIT) を参考にしています。本クレートは
独立した Rust 実装であり、当該プロジェクトのソースコードを複製していません。

## ライセンス

以下のいずれかを利用者が選択できるデュアルライセンスです。

- Apache License, Version 2.0（[LICENSE-APACHE](LICENSE-APACHE) または
  <http://www.apache.org/licenses/LICENSE-2.0>）
- MIT ライセンス（[LICENSE-MIT](LICENSE-MIT) または
  <http://opensource.org/licenses/MIT>）

### コントリビューション

本プロジェクトへ意図的に提出されたコントリビューションは、Apache-2.0 の定義に従い、
別途明示のない限り上記デュアルライセンスの下で提供されるものとします。
