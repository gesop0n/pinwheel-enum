//! criterion 3: ラベル付きグラフの単純閉路を列挙する。
//!
//! `on_stack` で経路上の頂点を再訪しない＝単純閉路だけを拾う。各閉路を最小添字の
//! 頂点でちょうど1回数え（`w == start` で閉じ、`w > start` でしか進まない）、実行タスク
//! 添字のラベル列を `visit` に渡す。Aut＋巡回シフトでの同一視は呼び出し側
//! （`super::aut`）が行う＝目的B の「遅延商」。

use super::graph::LabeledGraph;

/// `graph` の全単純閉路を列挙し、各閉路のラベル列（実行タスク添字の列）を `visit` に渡す。
pub fn for_each_simple_cycle(graph: &LabeledGraph, mut visit: impl FnMut(&[usize])) {
    let n = graph.len();
    let mut on_stack = vec![false; n];
    let mut labels: Vec<usize> = Vec::new();
    for start in 0..n {
        on_stack[start] = true;
        dfs(graph, start, start, &mut on_stack, &mut labels, &mut visit);
        on_stack[start] = false;
    }
}

/// `start` を最小添字とする単純閉路を探す DFS。
fn dfs(
    graph: &LabeledGraph,
    start: usize,
    v: usize,
    on_stack: &mut [bool],
    labels: &mut Vec<usize>,
    visit: &mut impl FnMut(&[usize]),
) {
    for &(label, w) in graph.successors_of(v) {
        if w == start {
            labels.push(label);
            visit(labels);
            labels.pop();
        } else if w > start && !on_stack[w] {
            on_stack[w] = true;
            labels.push(label);
            dfs(graph, start, w, on_stack, labels, visit);
            labels.pop();
            on_stack[w] = false;
        }
    }
}
