# perf-instructions.md

plc-comm-slmp-rust の性能改善指示書(SLMP ビット読みのバッチ化)。
この文書は実装担当モデル向けの完結した作業指示である。実装前にこの文書全体を読むこと。

> **本書は同リポジトリの `refactor-instructions.md`(挙動凍結の構造リファクタ)とは別タスクであり、
> あちらが禁止している「ワイヤ挙動の変更」を、唯一ここでだけ・以下の条件つきで許可する。**
> 両タスクを同時並行で実行しないこと。
>
> 条件: ①実機確認(Phase 0)で問題が確認されてから着手する。②返却値の意味は完全に維持する
> (変わるのはワイヤ上のコマンド構成と往復回数だけ)。③採用判定には実機 PLC での再検証
> (Phase 4)が必須で、これはユーザーの管理下で行う。④`cargo publish` はしない。

---

## Objective

`read_named` におけるプレーンなビットデバイス読み(dtype = `BIT`)の
**「1 点 = 1 往復・逐次」を、含有ワードの一括ランダム読みに統合**し、
PLC IO Checker のブロックビュー(ビット 64〜512 点表示)でのポーリング 1 回あたりの
ネットワーク往復回数を 1〜数回に削減する。

公開 API(関数シグネチャ・戻り値の型と意味)は一切変えない。アプリ側(Android / iOS)の
変更は不要であることがゴールの一部である。

---

## 問題の証拠(調査済み 2026-06-11、main `6656861`)

`src/helpers.rs`:

1. `compile_read_plan` がランダム読みバッチ(`word_devices` / `dword_devices`)に積むのは
   `BIT_IN_WORD` と `U/S/D/L/F` 型のみ。プレーンなビットアドレス(`M100`、`X1A0` 等、
   解決 dtype = `BIT`)はどのバッチにも入らない。
2. `read_named_compiled` のエントリ処理で `BIT` は最終 else 分岐 →
   `read_typed(client, entry.device, "BIT")` → `client.read_bits(device, 1)`。
   エントリごとに `await` で**逐次** 1 往復。
3. 一方 `BIT_IN_WORD`(`D50.3` 形式)は含有ワードを `read_random_maps`(`0x0403` ランダム読み、
   1 リクエスト最大 0xFF 点)から取得してビット抽出しており、**バッチ機構は既に存在する**。

影響: ブロックビューはページ内全アドレスを個別に読取要求へ入れる
(`computeDisplayedAddresses` → `currentBlockRangeDevices`、既定ページ 64 点・拡大可)。
ビットページではポーリング(既定 500ms)ごとにページ点数ぶんの逐次往復が発生する
(64 点で 64 往復、256 点で 256 往復 = RTT 1〜3ms でも 0.25〜0.75 秒/回)。
KEYENCE 側(`plc-comm-hostlink-rust`)は `DirectBit` がプラン最適化対象であり、
MELSEC だけが非対称に遅い。

正当性の傍証: 「連続ビットの `read_bits` 結果とワード詰め値の一致」は本リポジトリの実機検証
(iQ-R / iQ-L / iQ-F の `bit_blocks_passed=110〜135`、`docs/*_VALIDATION_2026-05-03.md`)で
確認済みの関係であり、ビットをワード単位で読んで展開する手法自体は検証済みである。

---

## Phase 0: 実機確認ゲート(着手条件)

> **✅ 確認済み(2026-06-11、ユーザー実測)**: 方法 A(計測カウンタ修正済みビルド)で
> 実機 **iQ-R**・ビット 64 点ブロックページ・ポーリング設定 500ms において
> **120 req/s** を観測(理論値 ~128.5)。120 ÷ 64 = 1.875 ポーリング/秒 = 実サイクル
> ~533ms であり、64 点時点で既に設定周期を超過していることも確認された。
> 参考ベースライン: メモリ 147MB / CPU 8.2%。
> **本ゲートは通過済み。実装担当は Phase 0 を再実行せず Phase 1 から開始してよい**
> (再検証したい場合のために以下の手順は残す)。
> 注: 計測カウンタ修正(方法 A)は `PlcIoChecker_Android/rust-core/src/service.rs` に
> 適用済み(2026-06-11 時点で未コミット)。iOS 側
> `melsec-io-core-ffi/src/service.rs` への同修正のミラーは未実施。

静的解析の結論を実測で確定させてから実装する。

> **注意(2026-06-11 確認済みの計測の罠)**: PLC IO Checker のデバッグオーバーレイの
> requests/sec は、**MELSEC ではワイヤ往復数を表示しない**。
> `rust-core/src/service.rs` の `record_slmp_comm_activity` が「JNI の readSnapshot
> 1 回 = request_count +1」「バイト数は最後の 1 フレームのみ加算」で実装されているため、
> 中で何百回往復しても表示は常に ~(1000 ÷ ポーリング間隔ms) req/s になる。
> KEYENCE 側は trace hook で全フレームを数えており、両ベンダで指標の意味が異なる。
> **現状のオーバーレイ読み値を「問題なし」の根拠にしないこと。**

確認方法は次のいずれか:

- **A(推奨・計測修正を兼ねる)**: `PlcIoChecker_Android/rust-core` の
  `record_slmp_comm_activity` を `client.traffic_stats()`(ライブラリの正確なワイヤカウンタ、
  `client.rs:1659` 付近で加算)ベースに修正し、`request_count` をワイヤ往復数、
  tx/rx を累積バイトにする(KEYENCE 側と意味を揃える)。その上で
  `debugFeaturesEnabled = true` ビルドで実機 MELSEC に接続し、ブロックビュー
  (既定 64 点ページ)でオーバーレイの requests/sec を読む。
  **約 (ページ点数 ÷ ポーリング間隔) req/s(64 点 / 500ms なら ~128 req/s)なら問題確定**。
  この計測修正はそれ自体が有益なので、確認後も残してよい(rust-core 側の変更であり、
  本ライブラリの変更ではない)。確認後 `TestParameters` は必ず原状復帰。
  ※デモモードは Mock がネイティブを通らないため確認に使えない。
  ※同じ計測実装は iOS 側 `PlcIoChecker_iOS/rust/melsec-io-core-ffi/src/service.rs` にも
  コピーされている(両ブリッジの service.rs はほぼ同一のコピー同士)。計測修正を採用する
  場合は、パリティのため同じ修正を iOS 側にもミラーする(別コミット・別リポジトリ)。
  ※**Phase 0 の問題確認自体は Android 実機だけで足りる**。原因は共有ライブラリにあり、
  両ブリッジが同一コードで同一の `read_named` を呼ぶことを確認済み(2026-06-11、
  両 service.rs のスナップショット読取部の diff 一致)。
- **B(ライブラリ単体)**: 実機 MELSEC に対し、ビットアドレス 64〜256 点の `Vec<String>` で
  `read_named` を 1 回呼び、前後の `client.traffic_stats()` の差分(リクエスト数)を
  一時コードで記録する(一時コードは除去)。
- **C(外部観測)**: PC を経由した実機接続で Wireshark / tcpdump により対象ポート
  (TCP 1025 等)の MC プロトコルフレーム数を 10 秒間カウントする。

実機 PLC を使えない環境の場合はここで停止し、上記 A の手順をユーザーに依頼する
(Stop And Ask)。**確認結果が「ポーリングあたり数往復程度」だった場合、本書の前提が
誤りなので実装せず、観測値を添えて報告・終了する。**

---

## 実装方針(Phase 2 の設計制約)

- 変更箇所は `compile_read_plan` / `read_named_compiled` 周辺(`src/helpers.rs`)に限定する。
  `SlmpClient` の public メソッド、`read_typed`、`read_bits` 自体は変更しない。
- プレーン `BIT` エントリのうち**安全と確認できるデバイスコードに限り**、含有ワード
  (デバイス番号を 16 点境界に切り下げたアドレス)を `word_devices` バッチに追加し、
  値は `BIT_IN_WORD` と同様に `word >> (number % 16) & 1` で抽出する。
- **安全なコードの決め方(重要)**: 推測で決めない。次の優先順で根拠を取る:
  1. 基準実装 `plc-comm-slmp-dotnet` の同等機能の対象範囲
  2. 既存コードのデバイス能力フラグ(`is_word_batchable` 等)と、ランダム読みワード単位での
     ビットデバイスアクセスを既に検証済みのルート(route validation の random 系)
  3. 判断できないコードは**従来どおり `read_bits(device, 1)` にフォールバック**する
     (フォールバック経路は削除しない)
- **アドレス境界の正確さ**: 16 点切り下げはパース済みの数値(`device.number`)上で行う。
  8 進表記の X/Y(対象ファミリ)もパース後は数値なので算術は同じだが、
  テストで 8 進系ファミリのケースを必ず含める。
- 既存のランダム読みチャンク上限(1 リクエスト 0xFF 点)・リクエスト分割ループは
  そのまま利用する。同一ワードを複数ビットが共有する場合は 1 回だけ読む(既存の
  `seen_word_devices` 重複排除に乗せる)。
- 返却される `NamedAddress` のキー・値・順序・エラー時の挙動を変えない。

---

## Phases

### Phase 1: 現状確認と baseline

1. `git status` 確認(クリーンでなければ停止・報告)
2. baseline 実行・記録:

```bash
cargo fmt --all --check
cargo clippy --all-targets --features cli -- -D warnings
cargo test
cargo build --features cli --bin slmp_verify_client
```

### Phase 2: 実装

実装方針に従って変更する。1 コミット相当の小さな単位で進め、都度 `cargo test`。

### Phase 3: テスト追加(モックサーバ・実機不要)

`tests/` に**新規ファイル**で追加(既存テストと `tests/shared-spec/` は無修正):

1. **値の等価性**: 同一アドレス集合に対し、モックサーバ応答から得る `read_named` の結果が
   変更前の意味(各ビットの真偽)と一致すること。ビットパターンは全 0 / 全 1 /
   交互 / ランダム数種。
2. **往復回数**: モックサーバ側で受信リクエスト数を数え、ビット N 点(同一ワード共有あり /
   なし、複数デバイス種混在、0xFF 超で分割)で期待往復数になること。
3. **フォールバック**: バッチ対象外コードが従来どおり個別 `read_bits` になること。
4. **境界**: ワード境界をまたぐ連番ビット、8 進系ファミリの X/Y、`BIT` と `BIT_IN_WORD` と
   ワード型の混在リクエスト。

### Phase 4: 実機再検証(ユーザー管理下・採用ゲート)

実装担当はここで停止し、ユーザーに以下の実行を依頼する(手順は `docs/live-validation.md` /
各アプリの手順に従う):

1. 利用可能な実機 PLC で `device_range_sample_compare` と `route_validation_compare`
   (過去記録 `docs/*_VALIDATION_*.md` と同水準の合格を確認)
2. Android / iOS ブリッジ経由の同スモーク(両アプリの rust ブリッジが本ライブラリに
   path 依存しているため、再ビルドだけで新コードが載る)
3. アプリのデバッグオーバーレイで requests/sec の前後比較を記録(Phase 0 と同条件)

**Phase 4 を通過するまで、この変更を「完了」と報告しないこと。**

### Phase 5(提案のみ・実装禁止)

- 読取プランのコンパイル結果キャッシュ(`compile_read_plan` は毎呼び出しで全アドレスを
  再パースする。解消にはプラン型と実行関数の public 化 = 公開 API 追加が必要なため、
  設計提案として報告書に書くだけにする。効果は RTT 削減後の残余で小さい)
- 4E フレームのシリアル番号を使った多重リクエスト化(大規模変更。提案のみ)

---

## Non-Negotiables(交渉不可の制約)

- 公開 API の変更・追加をしない(Phase 5 はそのため提案止まり)。
- `tests/` の既存ファイル・`tests/shared-spec/`・`docs/`・`examples/` を変更しない。
- `version` / `CHANGELOG.md` を変更しない。`cargo publish` をしない
  (リリースとバージョン繰り上げはユーザーの別判断。アプリは path 依存のため
  publish なしで効果を得られる)。
- 新しい依存クレートを追加しない。edition / MSRV を変更しない。
- フォールバック経路(従来の per-point `read_bits`)を削除しない。
- `write` 系・他の dtype の読取経路・エラーメッセージを変更しない。
- 一時計測コードは必ず除去してから報告する。
- 正しさが不明な点(特にデバイスコードのワードアクセス可否)は推測せず Stop And Ask。

## Stop And Ask Conditions

- Phase 0 で実機 PLC にアクセスできない(手順 A をユーザーに依頼して停止)
- Phase 0 の実測が本書の前提(1 点 1 往復)と食い違った(観測値を添えて報告・終了)
- あるデバイスコードがワード単位ランダム読みで安全に読めるか、.NET 基準実装からも
  既存コードからも判断できない(そのコードはフォールバックに残した上で質問)
- 変更後に既存テスト(特に `shared_vectors` / `route_guards`)が落ちた
  ⇒ 即座に巻き戻して報告
- 8 進系ファミリの境界計算に既存実装と矛盾する点を見つけた

## Verification Requirements(各フェーズ共通)

```bash
cargo fmt --all --check
cargo clippy --all-targets --features cli -- -D warnings
cargo test
cargo build --features cli --bin slmp_verify_client
# 兄弟リポジトリがある場合
cargo test --manifest-path ../PlcIoChecker_Android/rust-core/Cargo.toml --all-targets
cargo check --manifest-path ../PlcIoChecker_iOS/rust/melsec-io-core-ffi/Cargo.toml
```

- baseline で通っていたテストがすべて通り、Phase 3 の追加分だけ件数が増えていること
- `src/lib.rs` の `pub use` 一覧が無変更であること

## Reporting Format

1. **Phase 0 の実測**: 方法(A / B)、条件、観測した requests/sec または往復数
2. **対象デバイスコードの根拠表**: バッチ化したコード / フォールバックに残したコードと、
   それぞれの判断根拠(.NET 基準・既存フラグ・検証記録のどれか)
3. **変更内容の要約**と diff の範囲
4. **Phase 3 テスト一覧**: 各テストが固定した挙動(値等価・往復数・境界)
5. **Phase 4 の依頼内容**(ユーザー実行待ちの場合はその旨)/ 実施済みなら結果と
   requests/sec 前後比較
6. **各フェーズの検証コマンドと結果**(失敗を隠さない)
7. **Phase 5 提案**
8. **未実施事項**

## Out-of-scope Items

- KEYENCE(hostlink)側の変更(プラン最適化済み。要望があれば別途計測から)
- rust-core ブリッジ / アプリ側の変更(API 不変のため不要)
- プランキャッシュ・多重リクエスト化(Phase 5 提案のみ)
- 公開 API 追加、publish、バージョン変更
- `refactor-instructions.md` の作業(別タスク。同時並行禁止)
