# refactor-instructions.md

plc-comm-slmp-rust のリファクタリング指示書。
この文書は実装担当モデル向けの完結した作業指示である。実装前にこの文書全体を読むこと。

> **最重要の前提**: このクレートは crates.io に公開済み(`plc-comm-slmp-rust` 0.1.10)であり、
> かつワイヤプロトコル(SLMP フレームのバイト列)は**実機 PLC 6 機種で検証済みの記録**
> (`docs/*_VALIDATION_*.md`、`TODO.md`)に紐づく。
> **公開 API とフレームのバイト列は 1 バイトたりとも変えてはならない。**
> 本タスクの性質は「動いている通信ライブラリの内部整理」であり、改善余地はアプリ 2 本より小さい。
> 無理に変更量を増やさないこと。やることが無くなったら、それを正直に報告して終了してよい。

---

## Objective

公開 API・ワイヤプロトコル・クロススタック互換(Python / .NET / C++ / Node-RED / cross-verify)を
一切壊さずに、以下を行う:

- `src/client.rs`(約 2,350 行)に同居する self 非依存の純粋ヘルパ(検証ガード・デバイス分類・
  ビット展開等)の `pub(crate)` モジュールへの move-only 分離
- CI への `cargo fmt --check` / `cargo clippy` ゲート追加(現在欠落しており、fmt ドリフトの実績がある)
- 移動対象ロジックへの特性テスト追加(既存のモックサーバ / 共有ベクトル方式を踏襲)

「全面書き換え」「モジュール構成の再設計」「公開 API の整理」は**目的ではなく、禁止事項**である。

---

## Project Understanding

### 何のライブラリか

三菱 SLMP(MC プロトコル)Binary 3E / 4E の async Rust クライアント。TCP / UDP 対応。
`plc-comm-slmp-dotnet` を基準実装とし、Python / .NET / C++ / Node-RED / Rust の各実装が
`plc-comm-slmp-cross-verify` ハーネスで相互検証される一族の Rust 版。

### 利用者(壊すと影響が出る範囲)

1. **crates.io の一般利用者**(公開クレート。semver 契約)
2. **PLC IO Checker Android**: `../PlcIoChecker_Android/rust-core/`(path 依存)
3. **PLC IO Checker iOS**: `../PlcIoChecker_iOS/rust/melsec-io-core-ffi/`(path 依存)
4. **cross-verify ハーネス**: `src/bin/slmp_verify_client.rs`(`--features cli`)
5. **napi-rs Node スキャフォールド**: `crates/slmp-node`(workspace メンバ。7 行のプレースホルダ)

### モジュール構成(src/、計約 10,400 行)

| ファイル | 行数 | 内容 |
|---|---|---|
| `error_codes.rs` | 3,023 | SLMP エンドコードの名前 / 英語 / 日本語メッセージの match テーブル(約 2,000 アーム) |
| `client.rs` | 2,346 | `SlmpClient`(public)+ `ClientInner`(transport I/O、フレーム構築 / 解析、ペイロード構築、検証ガード、デバイス分類述語) |
| `device_ranges.rs` | 1,267 | PLC ファミリ別デバイスレンジカタログ(inline `#[cfg(test)]` あり) |
| `route_validation.rs` | 1,157 | ルート検証ハーネス。**ライブラリ内に意図的に同梱**(Android / iOS ブリッジから呼ばれる) |
| `helpers.rs` | 867 | `read_named` / `write_named` 等の高水準ヘルパ |
| `device_range_sample.rs` | 616 | 全デバイスサンプル検証ハーネス(同上、アプリから呼ばれる) |
| `model.rs` | 596 | 接続オプション、フレーム型、コマンド等の public 型 |
| `address.rs` | 383 | アドレス解析(inline test あり) |
| `error.rs` / `lib.rs` | 54 / 147 | エラー型、公開 re-export(**公開 API の一覧表**) |

### テスト戦略(既存の安全網)

- `tests/route_guards.rs`(539 行): ローカルモック TCP / UDP サーバを立ててガード挙動を検証
- `tests/shared_vectors.rs`(264 行): `tests/shared-spec/*.json` の**クロス実装共有ベクトル**と
  フレームバイト列を突き合わせる golden テスト(フレーム互換の番人)
- `tests/label_commands.rs` / `error_codes.rs` / `cpu_operation_state.rs` / `remote_cpu_control.rs`
- `examples/`(`device_matrix_compare` 1,114 行、`iql_live_stress` 741 行など)は実機 PLC 向け
  ライブツール。CI ではビルドのみ
- 実機検証記録: `docs/IQR_*` / `IQL_*` / `IQF_*` / `LCPU_*` / `QNUDV_*` 等(日付つき。**編集禁止**)

### CI(`.github/workflows/ci.yml`)

`cargo test` → `cargo check -p slmp-node` → `cargo build --features cli --bin slmp_verify_client`
→ `cargo doc --no-deps`。**`cargo fmt --check` と `cargo clippy` が無い**(後述 D2)。

### ツールチェーン

edition 2024、`rust-version = "1.85"`。変更禁止。

---

## Behaviors To Preserve(絶対に壊さない既存挙動)

1. **公開 API**: `src/lib.rs` の `pub use` 一覧がそのまま公開面である。公開アイテムの
   rename / 削除 / シグネチャ変更 / 追加を一切しない(追加も semver ノイズになるため不可)。
2. **フレームのバイト列**: `tests/shared-spec/*.json` のベクトルと `tests/shared_vectors.rs` が
   契約。`tests/` と `tests/shared-spec/` は**編集禁止**(特性テストの追加は新規ファイルで行う)。
3. **PLC ファミリ既定値**: `iQ-R` / `iQ-L` / `MX-R` / `MX-F` = `Frame4E` + `Iqr`、
   `iQ-F` と `QCPU` / `LCPU` / `QnU` / `QnUDV` = `Frame3E` + `Legacy`(README の表)。
4. **エンドコードのメッセージ文字列**: アプリがそのまま表示する。`end_code_name` /
   `end_code_message_en` / `end_code_message_ja` の入出力を変えない。
5. **検証ハーネス API**: `run_device_range_sample_compare` / `run_route_validation_compare` と
   そのレポート型は Android JNI / iOS C ABI から JSON 化されて使われる。型・フィールド・
   serde 表現を変えない。
6. **デバイス制約のガード挙動**: `LCS` / `LCC` のブロック禁止、long timer 系の random 経路、
   `.bit` 記法はワードデバイス限定、等(`tests/route_guards.rs` が仕様)。
7. **`SlmpError` の variant とメッセージ**(アプリのエラー分類が依存)。
8. **feature 構成**: `default = []`、`cli` feature の依存追加。default に依存を増やさない。
9. **crates.io 公開**: 本タスクで `cargo publish` をしない。`version` も `CHANGELOG.md` も
   変更しない(リリースはユーザーの別判断)。
10. **`crates/slmp-node`**: 意図的なプレースホルダ。触らない(`cargo check -p slmp-node` が
    通り続けること)。

---

## Non-Negotiables(交渉不可の制約)

- 最初に `git status` を確認する。未コミット変更があれば混ぜず、報告して停止する。
- 編集前に Baseline Commands をすべて実行し、結果(テスト件数含む)を記録する。
- 変更は小さく戻しやすい単位。コミットはユーザーの指示があるまで行わない。
- 無関係な整形・「ついで」リファクタリング・clippy 指摘の一括修正をしない
  (D2 で clippy を CI に足す際、既存コードに警告が出る場合の扱いは D2 の手順に従う)。
- 新しい依存クレートを追加しない。`Cargo.toml` は D2 で必要になる場合を除き変更しない
  (D2 でも原則変更不要のはず)。
- `unsafe` を導入しない(現状ゼロのはず。ゼロでなければ報告)。
- 移動した関数の可視性は `pub(crate)` まで。`pub` にしない。
- `edition` / `rust-version` / MSRV に影響する構文を使わない。
- `docs/`(実機検証記録)、`tests/`、`tests/shared-spec/`、`examples/`、`CHANGELOG.md`、
  `crates/slmp-node` を変更しない(特性テストは `tests/` に**新規ファイル追加のみ**可)。
- 正しさが不明な場合は実装を止め、「Stop And Ask」として質問を報告書に書く。
- 各フェーズ完了ごとに Verification Requirements を実行する。

---

## Stop And Ask Conditions(即時停止して質問する条件)

- 移動しようとした関数が実は `&self` の状態(compatibility mode / frame type 等)に依存しており、
  引数化(= 私的シグネチャの変更)が必要になった ⇒ その関数はスキップして報告(引数化は
  本タスクでは行わない)
- 変更後に `tests/shared_vectors.rs` または `tests/route_guards.rs` が落ちた
  ⇒ 即座に変更を巻き戻し、原因を報告(フレーム契約に触れた可能性が高い)
- `cargo clippy` が既存コードに対して `-D warnings` で大量の指摘を出し、機械的修正が
  挙動変更を伴いうる(例: `needless_range_loop` 以外の、演算順序・丸め・所有権に絡む指摘)
- 公開 API・serde 表現・エラーメッセージに影響しうる変更が必要に見えた
- 依存先アプリ(`../PlcIoChecker_Android/rust-core`、`../PlcIoChecker_iOS/rust/melsec-io-core-ffi`)
  のビルドが自分の変更後に失敗した
- 本書の Debt Map に無い大きな問題を発見した(報告のみ。勝手に直さない)

---

## Baseline Commands

作業ディレクトリ: リポジトリルート。Rust 1.85+ が必要。OS は問わない
(テストは localhost のモックサーバのみ使用。実機 PLC は不要・接続禁止)。

```bash
git status                                          # クリーンであることを確認
cargo test                                          # 単体 + 統合テスト(CI と同一)
cargo check -p slmp-node                            # workspace メンバ
cargo build --features cli --bin slmp_verify_client # cross-verify ラッパ
cargo doc --no-deps -p plc-comm-slmp-rust           # doc ビルド(doctest 含むのは cargo test 側)
cargo fmt --all --check                             # 現状を記録(CI には未導入)
cargo clippy --all-targets -- -D warnings           # 現状を記録(CI には未導入。失敗してもよい)
cargo clippy --all-targets --features cli -- -D warnings   # cli feature 込み
```

兄弟リポジトリがある環境では、依存側の baseline も記録(無ければスキップと明記):

```bash
cargo test --manifest-path ../PlcIoChecker_Android/rust-core/Cargo.toml --all-targets
cargo check --manifest-path ../PlcIoChecker_iOS/rust/melsec-io-core-ffi/Cargo.toml
```

---

## Debt Map

行番号は調査時点(main, commit `6656861`)のアンカー。ドリフトしていたら宣言名で探すこと。

### D1. `client.rs` 内の self 非依存ヘルパの同居 【実装可 / 主作業】

- **根拠**: `impl ClientInner`(617 行〜)に、状態を持たない静的関数が多数同居している。
  確認済みの候補(いずれも `self` を取らない):
  - 検証ガード: `validate_u16_count`(1605)、`validate_direct_bit_read`(1885)、
    `validate_direct_bit_write`(1896)、`validate_direct_word_read`(1907)、
    `validate_direct_word_write`(1923)、`validate_direct_dword_read`(1934)、
    `validate_direct_dword_write`(1945)、`validate_random_read_devices`(1956)、
    `validate_random_write_word_devices`(1987)、`validate_no_lcs_lcc_block_read`(2029)、
    `validate_no_lcs_lcc_block_write`(2061)
  - デバイス分類述語: `is_long_timer_state_device`(2002)、`requires_random_bit_write`(2009)、
    `is_long_current_value_device`(2014)、`is_dword_only_scalar_device`(2021)、
    `is_random_dword_only_read_device`(2025)
  - データ変換: `unpack_bit_values`(2246)、`parse_long_timer_words`(2263)
  - 自由関数: `map_plc_family_to_range_family`(603)、`decode_cpu_operation_state`(2309)
- **なぜ負債か**: 2,346 行のファイルでプロトコル仕様(どのデバイスがどの経路を使えるか)と
  I/O 実装が混ざり、仕様部分だけを読む・テストすることができない。
- **影響範囲**: クレート内部のみ(全関数が private / `pub(crate)` 相当)。
- **変更リスク**: 低(move-only + 呼び出し箇所の修飾子変更のみ)。
- **改善案**: 新規 private モジュール(例: `src/client/guards.rs` 等へのサブモジュール化、または
  `src/frame_rules.rs` のような単一ファイル。**`lib.rs` に `mod` を足す場合も `pub mod` にしない**)
  へ move-only で移し、`pub(crate) fn` にする。`&self` 依存が見つかった関数はスキップ
  (Stop And Ask 条件参照)。
- **検証**: 全 baseline テスト + 共有ベクトルテストが無修正で通ること。
- **注意**: `build_request_frame` / `parse_response` / `build_read_write_payload` /
  `encode_device_spec`(inner 版)など **`&self` を取るフレーム構築・解析系は対象外**。
  これらの分離は引数化を伴うため Phase 5(提案のみ)に回す。

### D2. CI に fmt / clippy ゲートが無い 【実装可】

- **根拠**: `.github/workflows/ci.yml` は test / napi check / cli build / doc のみ。
  直近コミット `6656861 Apply rustfmt cleanup` が示すとおり、ゲートが無いため fmt ドリフトが
  実際に起きた。兄弟リポジトリ(`PlcIoChecker_Android` / `PlcIoChecker_iOS`)の CI は
  自分のブリッジクレートに fmt + clippy `-D warnings` を課しており、本体ライブラリだけ無防備。
- **改善案**: ci.yml に `cargo fmt --all --check` と
  `cargo clippy --all-targets --features cli -- -D warnings` のステップを追加。
- **手順の制約**: 先にローカルで clippy を実行し、
  (a) 警告ゼロならそのまま CI に追加。
  (b) 警告があり、修正が**機械的かつ挙動同一と確信できる**もの(未使用 import、`&` の冗長など)
  だけなら最小修正して追加。
  (c) 挙動に関わりうる指摘が混ざるなら、**CI 追加を見送り Stop And Ask に回す**
  (`#[allow]` の一括貼り付けで黙らせることは禁止)。
- **検証**: CI 定義の YAML 構文と、ローカルでの同コマンド成功。

### D3. `error_codes.rs` の 3 重 match テーブル 【提案のみ・実装禁止】

- **根拠**: 約 2,000 の match アームで `end_code_name` / `end_code_message_en` /
  `end_code_message_ja` が同じコード集合を別々に列挙しており、片方だけ追加し忘れる
  ドリフトが構造的に起こりうる(`tests/error_codes.rs` はサンプル検査のみで全網羅は不可能)。
- **なぜ提案止まりか**: `const` テーブル + 検索への書き換えは動いているデータ表の全面改変で、
  2,000 項目の転記ミスのリスクが利益を上回る。**実装しない。** 報告書で、3 表の整合を機械検査
  できる構造(単一テーブル化、または codegen)の設計案として提示だけする。

### D4. その他(現状維持 / 報告のみ)

- `route_validation.rs` / `device_range_sample.rs` がライブラリ本体に同梱されているのは、
  Android / iOS ブリッジから同一ハーネスを呼ぶための**意図的設計**。動かさない。
- `examples/device_matrix_compare.rs`(1,114 行)等の実機ライブツールは検証資産。触らない。
- `crates/slmp-node`(7 行)は将来の Node パッケージング用プレースホルダ。触らない。
- `helpers.rs` / `address.rs` / `model.rs` / `device_ranges.rs` は規模・責務とも妥当。
  本タスクでは触らない(D1 の移動で参照修正が必要になる場合の機械的変更を除く)。

---

## Implementation Phases

各フェーズの最後に必ず Verification Requirements を実行し、通ってから次へ進む。

### Phase 0: 現状確認

1. `git status` 確認(クリーンでなければ停止・報告)
2. Baseline Commands を実行し、結果を記録(clippy / fmt の現状結果を必ず含める)

### Phase 1: 安全網(特性テスト)の追加

1. D1 の移動候補のうち、現在テストが薄いもの(デバイス分類述語、`unpack_bit_values`、
   `parse_long_timer_words`、`decode_cpu_operation_state`)に対し、`tests/` に**新規ファイル**
   (例: `tests/client_rules_characterization.rs`)で特性テストを追加する。
2. ただし対象は private 関数なので、この時点では public API 経由(モックサーバで該当経路を
   叩く既存方式)で到達できるものに限る。到達できないものは Phase 2 の `pub(crate)` 化後に
   ユニットテストを書く(`tests/` からは `pub(crate)` も見えないため、その場合は移動先
   モジュール内の `#[cfg(test)]` で書く。`device_ranges.rs` に前例あり)。
3. テストは「現在の出力をそのまま期待値にする」。仕様の発明禁止。

### Phase 2: `client.rs` の self 非依存ヘルパ分離(D1、move-only)

1. 1 グループ(検証ガード → 分類述語 → データ変換 → 自由関数)ずつ移動し、都度全テスト実行
2. 移動先モジュール内に `#[cfg(test)]` ユニットテストを追加(Phase 1 で書けなかった分)
3. `&self` 依存が判明した関数はスキップして報告

### Phase 3: CI ゲート追加(D2)

1. ローカルで clippy を実行し、D2 の手順(a)/(b)/(c) に従って判断
2. ci.yml にステップ追加

### Phase 4: 検証と報告

1. 全 Verification Requirements を最終実行(依存アプリのビルド確認を含む)
2. Reporting Format に従って報告書を作成

### Phase 5(提案のみ・実装禁止)

- フレーム構築 / 解析(`build_request_frame` / `parse_response` / ペイロード構築)の
  純関数化と専用モジュール分離の設計案(compatibility mode / frame type の引数化を含む)
- D3 のエンドコード表の単一テーブル化 / codegen 案

---

## Verification Requirements

各フェーズ完了時に最低限:

```bash
cargo test
cargo build --features cli --bin slmp_verify_client
cargo check -p slmp-node
cargo doc --no-deps -p plc-comm-slmp-rust
cargo fmt --all --check
```

最終フェーズでは追加で:

```bash
cargo clippy --all-targets --features cli -- -D warnings   # D2 実施後は必須
# 依存アプリ(兄弟リポジトリがある場合)
cargo test --manifest-path ../PlcIoChecker_Android/rust-core/Cargo.toml --all-targets
cargo check --manifest-path ../PlcIoChecker_iOS/rust/melsec-io-core-ffi/Cargo.toml
```

- baseline で通っていたテストがすべて通ること(件数が減っていないこと)
- `tests/` の既存ファイルと `tests/shared-spec/` が無変更であること(`git diff --stat` で確認)
- `src/lib.rs` の `pub use` 一覧が無変更であること(D1 で private `mod` 宣言を足す場合、
  `pub` の無い `mod` 行の追加のみ許可)
- 実機 PLC への接続を行わないこと(ライブ系 example / 環境変数ゲートのテストを実行しない)

---

## Reporting Format

作業完了時(または中断時)に以下を Markdown で報告する:

1. **Baseline 結果**: 実行コマンドと結果(テスト件数、clippy / fmt の初期状態)
2. **Phase 2 の移動一覧**: 移動した関数と移動先、スキップした関数とその理由(self 依存の内容)
3. **追加した特性テスト**: テスト名と固定した挙動の要約
4. **D2 の判断**: clippy 初期警告の内訳と、(a)/(b)/(c) のどれを選んだか
5. **各フェーズの検証結果**: 最後に実行したコマンドと結果(失敗を隠さない)
6. **公開 API 無変更の確認**: `src/lib.rs` の diff が `mod` 追加のみであることの確認結果
7. **Stop And Ask**: 発生した質問と停止範囲
8. **Phase 5 提案**: 実装しなかった設計案
9. **未実施事項**: 依存アプリのビルド確認ができなかった等の明記

---

## Out-of-scope Items(やらないこと)

- 公開 API の変更・追加・整理(re-export の並べ替えすら不可)
- フレーム構築 / 解析ロジックの書き換え(Phase 5 の提案のみ)
- `error_codes.rs` のテーブル構造変更(D3。提案のみ)
- バージョン番号変更、`CHANGELOG.md` 更新、`cargo publish`
- 依存クレートの追加・更新、edition / MSRV 変更
- `tests/` 既存ファイル・`tests/shared-spec/`・`docs/`・`examples/`・`crates/slmp-node` の変更
- 実機 PLC を使う検証(ライブ検証はユーザーの管理下でのみ行う)
- 兄弟リポジトリ(`PlcIoChecker_Android` / `PlcIoChecker_iOS` / `plc-comm-hostlink-rust` /
  cross-verify 一族)の変更
- 「死コード」と思われるものの削除(このリポジトリの一見未使用なコードは cross-verify /
  ライブ検証 / 将来の Node binding 用である可能性が高い)
