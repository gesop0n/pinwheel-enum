use crate::instance::PinwheelInstance;

/// 状態
/// 各仕事の (残り日数 r, 周期 c) の組みを `(r, c)` の昇順で保持する
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct State(Vec<(u32, u32)>);

impl State {
    /// 任意順の `(r, c)` 列から生成し、昇順に正規化する
    pub fn new(mut q: Vec<(u32, u32)>) -> Self {
        q.sort_unstable();
        Self(q)
    }

    /// 既に昇順とわかっている列から生成する
    /// `next_state` は構成上ソート済みの列を作るので再ソートを避ける
    fn from_sorted(q: Vec<(u32, u32)>) -> Self {
        debug_assert!(q.windows(2).all(|w| w[0] <= w[1]), "State must be sorted");
        Self(q)
    }

    pub fn as_slice(&self) -> &[(u32, u32)] {
        &self.0
    }

    /// initial_state(c): インスタンス の初期状態
    /// 各周期a を `(a-1, a)` とする
    pub fn initial(c: &PinwheelInstance) -> Self {
        State::new(c.periods().iter().map(|&a| (a - 1, a)).collect())
    }
}

/// next_state(v, j0)
/// 状態vの日にj0番目の仕事を行った時の翌日の状態
/// j0が「行える入力」であることは呼び出し側が保証する
/// Prop 4.5 の枝刈りに抵触する（他の仕事が締め切りに間に合わない）場合はNone
pub fn next_state(v: &State, j0: usize) -> Option<State> {
    let q = &v.0;
    let c0 = q[j0].1; // 今日行う仕事の周期
    let mut w: Vec<(u32, u32)> = Vec::with_capacity(q.len());
    let mut shifting = false; // j0由来のリセット組をまだ挿入していない区間でtrue
    for j in 0..q.len() {
        if j == j0 {
            shifting = true
        } else {
            // Prop 4.5: shifting中でなく、残り日数が位置j以下なら実行不能
            if !shifting && (q[j].0 as usize) <= j {
                return None;
            }
            w.push((q[j].0 - 1, q[j].1));
        }
        // リセット組 (c0 - 1, c0) を昇順を保つ位置に挿入する
        if shifting && (j + 1 == q.len() || q[j + 1].0 >= c0) {
            w.push((c0 - 1, c0));
            shifting = false;
        }
    }
    Some(State::from_sorted(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inst(periods: &[u32]) -> PinwheelInstance {
        PinwheelInstance::new(periods.to_vec())
    }

    fn st(pairs: &[(u32, u32)]) -> State {
        State::new(pairs.to_vec())
    }

    #[test]
    fn initial_state_632() {
        assert_eq!(
            State::initial(&inst(&[6, 3, 2])),
            st(&[(1, 2), (2, 3), (5, 6)])
        )
    }

    #[test]
    fn initial_state_633() {
        assert_eq!(
            State::initial(&inst(&[6, 3, 3])),
            st(&[(2, 3), (2, 3), (5, 6)])
        )
    }

    // (6, 3, 2): 各仕事を選んだ時の遷移
    #[test]
    fn next_state_632() {
        let v = State::initial(&inst(&[6, 3, 2])); // [(1, 2), (2, 3), (5, 6)]
        assert_eq!(next_state(&v, 0), Some(st(&[(1, 2), (1, 3), (4, 6)])));
        assert_eq!(next_state(&v, 1), Some(st(&[(0, 2), (2, 3), (4, 6)])));
        assert_eq!(next_state(&v, 2), Some(st(&[(0, 2), (1, 3), (5, 6)])));
    }
    // (6,3,3): 論文 Figure 5–8 のゴールデン木に対応する遷移
    #[test]
    fn next_state_633() {
        let v = State::initial(&inst(&[6, 3, 3])); // [(2,3),(2,3),(5,6)]
        // 周期3の同一の仕事なので j0 = 0, 1 は同じ後続状態
        assert_eq!(next_state(&v, 0), Some(st(&[(1, 3), (2, 3), (4, 6)])));
        assert_eq!(next_state(&v, 1), Some(st(&[(1, 3), (2, 3), (4, 6)])));
        assert_eq!(next_state(&v, 2), Some(st(&[(1, 3), (1, 3), (5, 6)])));
    }
    // Prop 4.5 の枝刈り:
    // 周期3が2つとも残り1日なのに周期6を選ぶと実行不能
    #[test]
    fn next_state_prunes_infeasible() {
        let v = st(&[(1, 3), (1, 3), (5, 6)]);
        assert_eq!(next_state(&v, 2), None);
    }

    // 出力が常に昇順（State の不変条件）であること
    #[test]
    fn next_state_stays_sorted() {
        let v = State::initial(&inst(&[6, 3, 3]));
        for j0 in 0..v.as_slice().len() {
            if let Some(w) = next_state(&v, j0) {
                assert!(w.as_slice().windows(2).all(|x| x[0] <= x[1]));
            }
        }
    }
}
