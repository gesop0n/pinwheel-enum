//! 商状態グラフ: 状態を頂点、`next_state`による遷移を有向辺とする

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    instance::PinwheelInstance,
    state::{State, next_state},
};

/// `state` から実行できる各手について、`(実行した周期, 翌日状態)` を返す
/// - GWS Prop 4.4 (同周期タスクは区別しない): 同一 `(urgency, periods)` のタスクは先頭のみ試す
/// - GWS Prop 4.5: 実行不能な手(`next_state`がNone)は除く
pub fn successors(state: &State) -> Vec<(u32, State)> {
    let tasks = state.as_slice();
    let mut out = Vec::new();
    let mut prev: Option<(u32, u32)> = None;
    for executed in 0..tasks.len() {
        let t = tasks[executed];
        let key = (t.urgency, t.period);
        if prev == Some(key) {
            continue; // 直前と同一のタスク -> 商グラフでは同じ辺
        }
        prev = Some(key);
        if let Some(next) = next_state(state, executed) {
            out.push((t.period, next));
        }
    }
    out
}

/// 初期状態 X0 から前方到達できる状態だけからなる有向グラフ
/// 頂点は `State` の昇順に並べ、index 0..n を割り当てる
/// （単純閉路を「最小添字の頂点」で一度だけ数える規約のため、順序を固定する
pub struct StateGraph {
    states: Vec<State>,
    /// adj[i] = i 番目の状態から出る（周期ラベル、行先の添字）の列
    adj: Vec<Vec<(u32, usize)>>,
}

impl StateGraph {
    /// インスタンスの初期状態 X0 から前方到達可能な部分グラフを展開する
    pub fn explore(instance: &PinwheelInstance) -> Self {
        let start = State::initial(instance);

        // 1. 到達可能な状態をすべて集める(DFS). BTreeSetなので昇順で得られる
        let mut discovered: BTreeSet<State> = BTreeSet::new();
        let mut stack = vec![start.clone()];
        discovered.insert(start);
        while let Some(s) = stack.pop() {
            for (_label, next) in successors(&s) {
                if discovered.insert(next.clone()) {
                    stack.push(next);
                }
            }
        }

        // 2. 昇順の頂点リストと、状態 -> 添字 の対応表を作る
        let states: Vec<State> = discovered.into_iter().collect();
        let index: BTreeMap<State, usize> = states
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();

        // 3. 隣接リストを構築
        let mut adj = vec![Vec::new(); states.len()];
        for (i, s) in states.iter().enumerate() {
            for (label, next) in successors(s) {
                adj[i].push((label, index[&next]));
            }
        }

        StateGraph { states, adj }
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub fn states(&self) -> &[State] {
        &self.states
    }

    pub fn successors_of(&self, v: usize) -> &[(u32, usize)] {
        &self.adj[v]
    }
}
