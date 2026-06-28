//! 目的A: 商状態グラフ（同周期タスクを区別しない）の単純閉路を列挙する。
//!
//! 状態をソートして Aut を早期に畳んだ商グラフ上で単純閉路を取り、周期列
//! `Schedule` の巡回シフト同値類として数える。目的B（`super::labeled`）と違い、
//! Aut はグラフを作る前（状態のソート）で消えている。詳細は
//! `docs/列挙の同一視（目的Aと目的B）.md`。

pub mod cycle;
pub mod graph;
pub mod schedule;
pub mod state;

pub use cycle::{enumerate, enumerate_schedules};
