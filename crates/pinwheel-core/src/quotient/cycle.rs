//! 単純閉路の全列挙

use std::collections::BTreeSet;

use crate::instance::PinwheelInstance;

use super::{graph::StateGraph, schedule::Schedule};

/// グラフ中の単純有向閉路を全て列挙し、スケジュール（周期ラベル列）の
/// 巡回シフト同値類の代表元の集合を返す
pub fn enumerate(graph: &StateGraph) -> BTreeSet<Schedule> {
    let mut result = BTreeSet::new();
    let n = graph.len();
    let mut on_stack = vec![false; n];
    let mut labels: Vec<u32> = Vec::new();

    for start in 0..n {
        on_stack[start] = true;
        dfs(graph, start, start, &mut on_stack, &mut labels, &mut result);
        on_stack[start] = false;
        debug_assert!(labels.is_empty());
    }

    result
}

/// `start` を最小添字とする単純閉路を探すDFS
/// `start` 以下の添字へは進まないことで、各閉路を最小頂点で一度だけ数える
fn dfs(
    graph: &StateGraph,
    start: usize,
    v: usize,
    on_stack: &mut [bool],
    labels: &mut Vec<u32>,
    result: &mut BTreeSet<Schedule>,
) {
    for &(label, w) in graph.successors_of(v) {
        if w == start {
            // startに戻った -> 単純閉路が一本確定
            labels.push(label);
            result.insert(Schedule::canonical(labels));
            labels.pop();
        } else if w > start && !on_stack[w] {
            on_stack[w] = true;
            labels.push(label);
            dfs(graph, start, w, on_stack, labels, result);
            labels.pop();
            on_stack[w] = false
        }
    }
}

/// インスタンスから直接列挙する
/// X0 から前方到達可能な範囲
pub fn enumerate_schedules(instance: &PinwheelInstance) -> BTreeSet<Schedule> {
    let graph = StateGraph::explore(instance);
    enumerate(&graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sched(p: &[u32]) -> Schedule {
        Schedule(p.to_vec())
    }
    fn run(periods: &[u32]) -> BTreeSet<Schedule> {
        enumerate_schedules(&PinwheelInstance::new(periods.to_vec()))
    }

    // (2,2): 周期2を毎日やる自己ループ1個 → [2]
    #[test]
    fn two_two_collapses_to_single_selfloop() {
        assert_eq!(run(&[2, 2]), BTreeSet::from([sched(&[2])]));
    }

    // (3,3,3): ABC/ACBは同一視され、商グラフでは長さ1の自己ループ [3]
    #[test]
    fn three_three_three_collapses() {
        assert_eq!(run(&[3, 3, 3]), BTreeSet::from([sched(&[3])]));
    }

    // (2,3,M) は割当不能 → 閉路なし
    #[test]
    fn unschedulable_is_empty() {
        assert!(run(&[2, 3, 6]).is_empty());
    }

    // 割当可能なインスタンスは非空（(2,3) は tightly feasible）
    #[test]
    fn schedulable_is_nonempty() {
        assert!(!run(&[2, 3]).is_empty());
        assert!(!run(&[2, 4, 4]).is_empty());
        assert!(!run(&[2, 4, 8, 8]).is_empty());
    }

    #[test]
    fn canonical_is_lex_min_rotation() {
        assert_eq!(Schedule::canonical(&[4, 2, 4, 2]), sched(&[2, 4, 2, 4]));
        assert_eq!(Schedule::canonical(&[3, 2, 2]), sched(&[2, 2, 3]));
    }
}
