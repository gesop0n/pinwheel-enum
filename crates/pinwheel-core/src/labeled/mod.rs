//! ラベル付き（タスクを区別する）スケジュール列挙＝目的B。
//!
//! ラベル付き状態グラフ `G_A`（`graph::LabeledGraph`、Aut で割らない）の単純閉路を
//! 全列挙し（`cycle`、criterion 3）、`Aut`＋巡回シフトで同一視して（`aut`、criterion 1+2）
//! 各同値類の辞書順最小代表を返す。返り値の各要素はタスク添字（0=A, 1=B, …）の列。
//!
//! 個数は目的A（`super::quotient` の商閉路）とは一般に一致しない。1つの商閉路が
//! 複数の持ち上げ（lift）を持つため、tight でない個例では目的B の方が多い。
//! 例: (3,4,4) は目的A = 3 本だが目的B = 5 本。詳細は
//! `docs/列挙の同一視（目的Aと目的B）.md`。
//!
//! 例: (2,4,4) -> [[0,1,0,2]]（ABAC, 1個）, (3,3,3) -> [[0,1,2]]（ABC, 1個）。

use std::collections::BTreeSet;

use crate::instance::PinwheelInstance;

pub mod aut;
pub mod cycle;
pub mod graph;

use aut::{aut_perms, canonical};
use cycle::for_each_simple_cycle;
use graph::LabeledGraph;

/// インスタンスのラベル付きスケジュールを列挙する（目的B）。
/// 返り値はタスク添字の列の集合（辞書順）。
pub fn enumerate_labeled(instance: &PinwheelInstance) -> Vec<Vec<usize>> {
    // (a) ラベル付きグラフを展開し、(b) その単純閉路（criterion 3）を全列挙して、
    //     (c) Aut＋回転（criterion 1+2）の正規形で重複排除する。
    let graph = LabeledGraph::explore(instance);
    let auts = aut_perms(instance.periods());

    let mut result: BTreeSet<Vec<usize>> = BTreeSet::new();
    for_each_simple_cycle(&graph, |seq| {
        result.insert(canonical(seq, &auts));
    });
    result.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(periods: &[u32]) -> Vec<Vec<usize>> {
        enumerate_labeled(&PinwheelInstance::new(periods.to_vec()))
    }

    // (2,4,4): B,C は入れ替えても回転で同じ -> ABAC が1個
    #[test]
    fn labeled_244_is_abac() {
        assert_eq!(run(&[2, 4, 4]), vec![vec![0, 1, 0, 2]]);
    }

    // (3,3,3): ABC と ACB は Aut で同一視 -> ABC が1個
    #[test]
    fn labeled_333_is_abc() {
        assert_eq!(run(&[3, 3, 3]), vec![vec![0, 1, 2]]);
    }

    // (2,2): AB が1個
    #[test]
    fn labeled_22_is_ab() {
        assert_eq!(run(&[2, 2]), vec![vec![0, 1]]);
    }

    // (2,3,6) は割当不能 -> 空
    #[test]
    fn labeled_unschedulable_is_empty() {
        assert!(run(&[2, 3, 6]).is_empty());
    }
}
