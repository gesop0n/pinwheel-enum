//! 目的B のラベル付き状態グラフ `G_A`（Aut で割らない）。
//!
//! 状態は各タスクの urgency ベクトル（添字＝タスク、ソートしない）。同周期タスクを
//! 畳まないので holonomy が展開された具体状態が頂点になる。目的A の商グラフ
//! （`super::super::quotient::graph::StateGraph`）と対をなす。

use std::collections::{BTreeMap, BTreeSet};

use crate::instance::PinwheelInstance;

/// ラベル付き状態: 各タスク i の urgency u_i = a_i - x_i - 1 を添字順に保持する。
pub type LabeledState = Vec<u32>;

/// タスク `executed` を実行した翌日の状態。`executed` 以外に締切超過のタスクがあれば None。
pub fn labeled_next(
    periods: &[u32],
    state: &LabeledState,
    executed: usize,
) -> Option<LabeledState> {
    let mut next = vec![0u32; periods.len()];
    for (i, &period) in periods.iter().enumerate() {
        if i == executed {
            next[i] = period - 1; // リセット
        } else if state[i] == 0 {
            return None; // タスク i が今日できず締切超過
        } else {
            next[i] = state[i] - 1;
        }
    }
    Some(next)
}

fn initial(periods: &[u32]) -> LabeledState {
    periods.iter().map(|&a| a - 1).collect()
}

/// 初期状態 X0 から前方到達可能なラベル付き状態グラフ。
/// 頂点は状態の昇順（単純閉路を最小添字の頂点で一度だけ数える規約のため順序を固定）、
/// 辺は `(実行タスク添字, 行先の頂点添字)`。
pub struct LabeledGraph {
    states: Vec<LabeledState>,
    /// adj[i] = i 番目の状態から出る (実行タスク添字, 行先の添字) の列
    adj: Vec<Vec<(usize, usize)>>,
}

impl LabeledGraph {
    /// インスタンスの初期状態 X0 から前方到達可能な部分グラフを展開する。
    pub fn explore(instance: &PinwheelInstance) -> Self {
        let periods = instance.periods();
        let k = periods.len();

        // 1. X0 から前方到達可能なラベル付き状態を集める（DFS）
        let mut discovered: BTreeSet<LabeledState> = BTreeSet::new();
        let start = initial(periods);
        let mut stack = vec![start.clone()];
        discovered.insert(start);
        while let Some(s) = stack.pop() {
            for executed in 0..k {
                if let Some(next) = labeled_next(periods, &s, executed)
                    && discovered.insert(next.clone())
                {
                    stack.push(next);
                }
            }
        }

        // 2. 添字付け（BTreeSet は昇順）と隣接リストの構築
        let states: Vec<LabeledState> = discovered.into_iter().collect();
        let index: BTreeMap<LabeledState, usize> = states
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();
        let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); states.len()];
        for (i, s) in states.iter().enumerate() {
            for executed in 0..k {
                if let Some(next) = labeled_next(periods, s, executed) {
                    adj[i].push((executed, index[&next]));
                }
            }
        }

        LabeledGraph { states, adj }
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub fn states(&self) -> &[LabeledState] {
        &self.states
    }

    pub fn successors_of(&self, v: usize) -> &[(usize, usize)] {
        &self.adj[v]
    }
}
