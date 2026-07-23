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
| _(なし)_ | 有効 | コアのパースと、ゴールデン corpus 全件で往復検証する humanize。I/O なし・実行時依存なし。 |
| `dates-jiff` | 無効 | `jiff` によるカレンダー演算と相対日付（`next friday`、`来週金曜日`）。 |
| `timezones-jiff` | 無効 | IANA タイムゾーン対応。ただし解決には明示的な `reference_date` が必須です（日付なしにゾーンのオフセットは定まらないため）。指定がなければ `3pm Europe/Paris` は既定ビルドと同じく `IssueCode::TimezoneUnsupported` として報告され、`best` は `None` になります。`dates-jiff` を含みます。 |
| `wasm` | 無効 | ブラウザ / Node 向けの `wasm-bindgen` エクスポート。詳細は [docs/wasm.md](docs/wasm.md)。 |

## 使い方

```rust
use unravel_nl::{humanize, HumanizeCtx, Locale, Parser};

let parser = Parser::japanese_building();
let parsed = parser.parse("5尺3寸");

let best = parsed.best.expect("a canonical reading");
assert_eq!(best.unit.as_deref(), Some("m"));
assert_eq!(
    humanize(&best, Some(HumanizeCtx { locale: Some(Locale::Ja) })),
    "5尺3寸 (approx.)"
);
```

### 設定済み Parser

入力欄が受け付ける次元を知っている場合は、`Parser` instance を使います。
指定した次元集合は、文法dispatch・registry lookup・typo補正・completion・最終受理の
すべてに適用されます。無効な単位は候補に入らず、有効な別単位へのtypoとして捏造されません。

```rust
use unravel_nl::{Dimension, DimensionSet, Parser};

let parser = Parser::new(DimensionSet::from(Dimension::Mass));
let parsed = parser.parse("1,234 kg");
assert_eq!(parsed.best.unwrap().unit.as_deref(), Some("kg"));
```

`Parser::japanese_building()` は長さ＋面積だけの小さいpresetです。locale を仮定しない
同じ最小構成は `Parser::default()`、全catalogを明示的に使う場合だけ
`Parser::unrestricted()` を選びます。
`Parser::parse_dimensions_for_editor()`、`complete()`、`complete_readings()` も同じ境界を再利用します。

`unit_registry` は「存在する語彙」、`expected_dimensions` は「読んだ値の受理方針」です。
`Parser::new()` は両者を同じ集合にするため、通常の利用者が誤って食い違わせることはありません。

### 文中からの寸法抽出

文全体から複数の値を取り出す入口（`parse_all()`）は意図的に提供していません。文の
走査は「どこで値が終わり次が始まるか」を推測せざるを得ず、その推測がレビューのたびに
不具合の発生源になりました。API 形状の参照元ライブラリにも同等の関数はありません。
呼び出し側が範囲を決めたフィールドに対する単一値パースが、サポートされる形です。
判断の根拠となる5回分の検証記録は、公開クレートではなく設計ノート側の
**「Sentence extraction is out of scope」** に残しています。

この節を「それで問題が解決した」と読まないでください。6 回目の検証で、**同じ種類の不具合——
入力が保持していない値を返す——が単一値パース側にも存在**し、それを禁じるはずの性質テスト自体に
3 つの死角があったことが判明しました。そのため 5 回分の前提のうち 1 つを取り下げています。
5 回の検証が示すのは「文の走査の方が悪かった」ことであって、**残った側が完成しているという意味ではありません**。
6 回目で何が変わったかは設計ノート側に記録しており、この件が開いている間は crates.io へ公開しません。

残っているスキャナは構造上狭いものだけです。エディタ欄の中から、ラベルで裏付けの取れる
建築寸法だけを読み取ります。

寸法しか受け付けないエディタ欄では、専用スキャナを使います。通貨・日付・一般文法を
避けつつ、日本の建築単位を保ったまま長さと面積だけを拾います。

```rust
use unravel_nl::Parser;

let parser = Parser::japanese_building();
let matches = parser.parse_dimensions_for_editor(
    "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640"
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
use unravel_nl::{Date, DimensionSet, Locale, ParseCtx, ParsePurpose, Parser};

let parser = Parser::with_context(
    DimensionSet::new(),
    ParseCtx {
        locale: Some(Locale::Ja),
        reference_date: Date::new(2026, 7, 19),
        purpose: ParsePurpose::Date,
        ..ParseCtx::default()
    },
);
let parsed = parser.parse("来週金曜日");

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
`parseAllForUi()`、フィールド一括の `canonicalizeFieldsForUi()`、React 連携）が
入っています。注入する parser は core の `ok` と rank 済み `issues` を返す必要があり、
アダプタ側で受理判定や issue 分類を再実装しません。TypeScript 型定義は
`web/unravel-adapters.d.ts` で、両者の export
一覧を突き合わせるテストがあるため型定義が実装から遅れることはありません。詳細は
[docs/wasm.md](docs/wasm.md)。

カスタム要素（Web Component）は**提供していません**。`defineUnravelElement()` も
`<unravel-input>` もなく、追加する予定もありません。これはこのファイルの export の
うち、**リポジトリ内のどこからも呼ばれていない唯一のもの**でした。テストからも例からも
他のアダプタからも呼ばれておらず、定義行だけが存在していました。テストされていない
`createUnravelFieldController()` のラッパは機能ではなく、同じ機能への2つ目の入口に
すぎません。残したのは実際にテストで叩かれている側です。カスタム要素が必要な場合は
`createUnravelFieldController()` の上に十数行で書けます。そのとき、タグ名・レジストリ・
Shadow DOM をどうするかという登録ポリシーは、このクレートがテストしていない既定値を
受け継ぐのではなく、呼び出し側が自分で決めることになります。

### 繰り返し表現

繰り返し表現の入口は**ありません**。`parse_recurrence_fast()` も `Kind::Recurrence` も
`Reading::recurrence` も `ParsePurpose::Recurrence` もなく、追加する予定もありません。
`every monday`、`毎週月曜`、`every third business day`、生の `FREQ=…` 文字列はいずれも
読み取りません。これらは `best: None` と `IssueCode::NoValue` の finding を伴って返ります。
読めないものが黙って消えるのではなく、他の読めない入力とまったく同じ経路で理由が出ます。

削除の理由は 2 つあり、どちらもコードではなく API 表面の話です。第一に、このクレートが
API 形状の参照元としている `pascalorg/lingo` は、繰り返し表現の公開 API を**一切
記載していません**。root の README にも `packages/lingo` の README にも `recurrence` /
`RRULE` / `every …` に相当する記述は 1 件もありません（固定 commit
`8507914c476026afbbc2f4f9fe84b31f2713c6a2` で実測）。つまりこの入口は独自拡張であり、
外部の契約に対して責任を負っていませんでした。第二に、その表面自身が「何を返さないか」を
言い切れていませんでした。`IssueCode::RecurrenceUnsupported` ——「文法としては繰り返し
表現だと認識したが、規則として表現できない」ことを認めるためだけに存在するコード —— を
持っていたことが、境界が決まっていなかった証拠です。繰り返しスケジュールは RFC 5545 と
いう実在の仕様を持つカレンダー領域の問題であり、値パーサに RRULE の部分集合を後付けした
ものはその仕様ではありません。必要とする呼び出し側は、仕様を実装したライブラリを使う方が
適切です。

日付・時刻・期間・時刻スロットは別の問題であり、影響を受けません。`3pm-4pm`、`1h30`、
`PT1H30M`、`明日`、`来週金曜日` は従来どおり読み取れます。

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
