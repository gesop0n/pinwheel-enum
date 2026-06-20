//! ラベル付き（タスクを区別する）スケジュール列挙（表示用）。
//!
//! 列挙の個数は目的A（商）と一致させる: ラベル付き単純閉路を
//! Aut（同周期タスクの置換）＋巡回シフトで同一視し、各類の辞書順最小代表を返す。
//! 返り値の各要素はタスク添字（0=A, 1=B, …）の列。
//!
//! 例: (2,4,4) -> [[0,1,0,2]]（ABAC, 1個）, (3,3,3) -> [[0,1,2]]（ABC, 1個）。

use std::collections::{BTreeMap, BTreeSet};

use crate::instance::PinwheelInstance;

/// ラベル付き状態: 各タスク i の urgency u_i = a_i - x_i - 1 を添字順に保持する。
type LabeledState = Vec<u32>;

/// タスク `executed` を実行した翌日の状態。`executed` 以外に締切超過のタスクがあれば None。
fn labeled_next(periods: &[u32], state: &LabeledState, executed: usize) -> Option<LabeledState> {
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

/// インスタンスのラベル付きスケジュールを列挙する（目的A基準）。
/// 返り値はタスク添字の列の集合（辞書順）。
pub fn enumerate_labeled(instance: &PinwheelInstance) -> Vec<Vec<usize>> {
    let periods = instance.periods();
    let k = periods.len();

    // 1. X0 から前方到達可能なラベル付き状態を集める
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

    // 2. 添字付け（BTreeSet は昇順）と隣接リスト（実行タスク添字, 行先）
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

    // 3. 単純閉路を全列挙し、Aut＋回転の正規形で重複排除
    let auts = aut_perms(periods);
    let n = states.len();
    let mut result: BTreeSet<Vec<usize>> = BTreeSet::new();
    let mut on_stack = vec![false; n];
    let mut labels: Vec<usize> = Vec::new();
    for start in 0..n {
        on_stack[start] = true;
        dfs(&adj, start, start, &mut on_stack, &mut labels, &auts, &mut result);
        on_stack[start] = false;
    }
    result.into_iter().collect()
}

/// `start` を最小添字とする単純閉路を探す DFS。
fn dfs(
    adj: &[Vec<(usize, usize)>],
    start: usize,
    v: usize,
    on_stack: &mut [bool],
    labels: &mut Vec<usize>,
    auts: &[Vec<usize>],
    result: &mut BTreeSet<Vec<usize>>,
) {
    for &(label, w) in &adj[v] {
        if w == start {
            labels.push(label);
            result.insert(canonical(labels, auts));
            labels.pop();
        } else if w > start && !on_stack[w] {
            on_stack[w] = true;
            labels.push(label);
            dfs(adj, start, w, on_stack, labels, auts, result);
            labels.pop();
            on_stack[w] = false;
        }
    }
}

/// タスク添字列を、Aut（同周期タスクの置換）と巡回シフトで割った辞書順最小代表に正規化する。
fn canonical(seq: &[usize], auts: &[Vec<usize>]) -> Vec<usize> {
    let n = seq.len();
    let mut best: Option<Vec<usize>> = None;
    for perm in auts {
        let relabeled: Vec<usize> = seq.iter().map(|&e| perm[e]).collect();
        for r in 0..n {
            let rotated: Vec<usize> = (0..n).map(|i| relabeled[(r + i) % n]).collect();
            if best.as_ref().is_none_or(|b| rotated < *b) {
                best = Some(rotated);
            }
        }
    }
    best.unwrap_or_default()
}

/// Aut(A) = 同一周期のタスク同士を入れ替える置換群。
/// 各要素は `perm[i] = タスク i の写り先` を表す長さ k の配列。
fn aut_perms(periods: &[u32]) -> Vec<Vec<usize>> {
    let k = periods.len();
    // 同一周期の連続区間（periods は昇順）でグループ分け
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut i = 0;
    while i < k {
        let mut j = i;
        while j < k && periods[j] == periods[i] {
            j += 1;
        }
        groups.push((i..j).collect());
        i = j;
    }
    // 各グループ内の全順列の直積
    let mut result: Vec<Vec<usize>> = vec![(0..k).collect()];
    for group in &groups {
        let group_perms = permutations(group);
        let mut next = Vec::new();
        for base in &result {
            for gp in &group_perms {
                let mut perm = base.clone();
                for (t, &pos) in group.iter().enumerate() {
                    perm[pos] = gp[t];
                }
                next.push(perm);
            }
        }
        result = next;
    }
    result
}

/// `items` の全順列。
fn permutations(items: &[usize]) -> Vec<Vec<usize>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(i);
        for mut p in permutations(&rest) {
            p.insert(0, head);
            out.push(p);
        }
    }
    out
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

    // 同周期を持つ場合の Aut のサイズ確認
    #[test]
    fn aut_size() {
        assert_eq!(aut_perms(&[2, 4, 4]).len(), 2); // S2
        assert_eq!(aut_perms(&[3, 3, 3]).len(), 6); // S3
        assert_eq!(aut_perms(&[2, 4, 8]).len(), 1); // 自明
    }
}
