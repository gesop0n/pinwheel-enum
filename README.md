# pinwheel-enum

輪番割当（pinwheel scheduling）問題のスケジュール列挙。

## CLI の使い方

```bash
# スケジュールを列挙（既定 --format tasks: A,B,C の具体的な日割）
cargo run -p pinwheel-cli -- enumerate 2 4 4

# 周期列で表示（--format periods: 商グラフの単純閉路）
cargo run -p pinwheel-cli -- enumerate 2 4 4 --format periods

# 件数だけ表示
cargo run -p pinwheel-cli -- enumerate 2 4 8 8 --count

# 密度 D(A) = Σ 1/a_i を表示
cargo run -p pinwheel-cli -- density 2 4 4

# ヘルプ
cargo run -p pinwheel-cli -- --help
```

出力例:

```
$ cargo run -p pinwheel-cli -- enumerate 2 4 4
[2, 4, 4]: 1 schedule(s)
labels: A=2, B=4, C=4
  ABAC
```

スケジュールはタスクを `A, B, C, …`（周期の小さい順）で表示する。`labels:` 行が各タスクの周期。割当不能なインスタンスは `0 schedule(s)` になる。

2つの `--format` は数える対象が異なる；

- `tasks`: 具体的なスケジュールをすべて列挙。
- `periods`: 商グラフの単純閉路（周期列）。

一般に `periods` の件数 ≤ `tasks` の件数（例: `(3,4,4)` は periods=3, tasks=5）。

## ビルド / テスト

```bash
cargo build
cargo test --workspace
```
