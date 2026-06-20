use crate::instance::PinwheelInstance;

/// 各仕事の状態を表す 1 要素
/// `urgency` 昇順（同 urgency なら `period` 昇順）で `State` 内に保持される
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Task {
    /// 締切まであと何日待てるか（GSW の urgency `u_i = a_i - x_i - 1`）。小さいほど切迫
    pub urgency: u32,
    /// 周期 `a_i`（GSW の frequency、河村の周期）
    pub period: u32,
}

/// 状態
/// 各仕事の `Task { urgency, period }` を urgency 昇順で保持する
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct State(Vec<Task>);

impl State {
    /// 任意順の `Task` 列から生成し、昇順に正規化する
    pub fn new(mut tasks: Vec<Task>) -> Self {
        tasks.sort_unstable();
        Self(tasks)
    }

    /// 既に昇順とわかっている列から生成する
    /// `next_state` は構成上ソート済みの列を作るので再ソートを避ける
    fn from_sorted(tasks: Vec<Task>) -> Self {
        debug_assert!(
            tasks.windows(2).all(|w| w[0] <= w[1]),
            "State must be sorted"
        );
        Self(tasks)
    }

    pub fn as_slice(&self) -> &[Task] {
        &self.0
    }

    /// インスタンスの初期状態
    /// 各周期 `a` を `Task { urgency: a - 1, period: a }` とする（GSW の `X0 = (0, …, 0)` に対応）
    pub fn initial(instance: &PinwheelInstance) -> Self {
        State::new(
            instance
                .periods()
                .iter()
                .map(|&period| Task {
                    urgency: period - 1,
                    period,
                })
                .collect(),
        )
    }
}

/// `state` の日に `executed` 番目（urgency 昇順での順位）の仕事を行った時の翌日の状態
/// `executed` が「行える入力」であることは呼び出し側が保証する
/// Prop 4.5 の枝刈りに抵触する（他の仕事が締め切りに間に合わない）場合は None
pub fn next_state(state: &State, executed: usize) -> Option<State> {
    let tasks = &state.0;
    let executed_period = tasks[executed].period; // 今日行う仕事の周期
    let mut next: Vec<Task> = Vec::with_capacity(tasks.len());
    let mut reset_pending = false; // 実行タスクのリセット組をまだ挿入していない区間で true
    for pos in 0..tasks.len() {
        if pos == executed {
            reset_pending = true
        } else {
            // Prop 4.5: リセット待ち区間でなく、urgency が順位 pos 以下なら実行不能
            if !reset_pending && (tasks[pos].urgency as usize) <= pos {
                return None;
            }
            next.push(Task {
                urgency: tasks[pos].urgency - 1,
                period: tasks[pos].period,
            });
        }
        // リセット組 (urgency = executed_period - 1) を昇順を保つ位置に挿入する
        if reset_pending && (pos + 1 == tasks.len() || tasks[pos + 1].urgency >= executed_period) {
            next.push(Task {
                urgency: executed_period - 1,
                period: executed_period,
            });
            reset_pending = false;
        }
    }
    Some(State::from_sorted(next))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inst(periods: &[u32]) -> PinwheelInstance {
        PinwheelInstance::new(periods.to_vec())
    }

    fn st(pairs: &[(u32, u32)]) -> State {
        State::new(
            pairs
                .iter()
                .map(|&(urgency, period)| Task { urgency, period })
                .collect(),
        )
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
        let state = State::initial(&inst(&[6, 3, 2])); // [(1, 2), (2, 3), (5, 6)]
        assert_eq!(next_state(&state, 0), Some(st(&[(1, 2), (1, 3), (4, 6)])));
        assert_eq!(next_state(&state, 1), Some(st(&[(0, 2), (2, 3), (4, 6)])));
        assert_eq!(next_state(&state, 2), Some(st(&[(0, 2), (1, 3), (5, 6)])));
    }
    // (6,3,3): 論文 Figure 5–8 のゴールデン木に対応する遷移
    #[test]
    fn next_state_633() {
        let state = State::initial(&inst(&[6, 3, 3])); // [(2,3),(2,3),(5,6)]
        // 周期3の同一の仕事なので executed = 0, 1 は同じ後続状態
        assert_eq!(next_state(&state, 0), Some(st(&[(1, 3), (2, 3), (4, 6)])));
        assert_eq!(next_state(&state, 1), Some(st(&[(1, 3), (2, 3), (4, 6)])));
        assert_eq!(next_state(&state, 2), Some(st(&[(1, 3), (1, 3), (5, 6)])));
    }
    // Prop 4.5 の枝刈り:
    // 周期3が2つとも残り1日なのに周期6を選ぶと実行不能
    #[test]
    fn next_state_prunes_infeasible() {
        let state = st(&[(1, 3), (1, 3), (5, 6)]);
        assert_eq!(next_state(&state, 2), None);
    }

    // 出力が常に昇順（State の不変条件）であること
    #[test]
    fn next_state_stays_sorted() {
        let state = State::initial(&inst(&[6, 3, 3]));
        for executed in 0..state.as_slice().len() {
            if let Some(next) = next_state(&state, executed) {
                assert!(next.as_slice().windows(2).all(|x| x[0] <= x[1]));
            }
        }
    }
}
