//! criterion 1+2: 同周期タスクの置換群 Aut と、Aut＋巡回シフトによる正準化。
//!
//! 目的B は閉路を全部取った「あと」にここで同一視する（遅延商）。`canonical` が
//! criterion 2（同周期入替）と criterion 1（回転）をまとめて当て、辞書順最小を代表とする。

/// タスク添字列を、Aut（同周期タスクの置換）と巡回シフトで割った辞書順最小代表に正規化する。
pub fn canonical(seq: &[usize], auts: &[Vec<usize>]) -> Vec<usize> {
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
pub fn aut_perms(periods: &[u32]) -> Vec<Vec<usize>> {
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

    // 同周期を持つ場合の Aut のサイズ確認
    #[test]
    fn aut_size() {
        assert_eq!(aut_perms(&[2, 4, 4]).len(), 2); // S2
        assert_eq!(aut_perms(&[3, 3, 3]).len(), 6); // S3
        assert_eq!(aut_perms(&[2, 4, 8]).len(), 1); // 自明
    }

    // canonical: 同周期入替（Aut）＋回転の辞書順最小をとる
    #[test]
    fn canonical_folds_aut_and_rotation() {
        // (2,4,4) の Aut = {恒等, B<->C}。ABAC(0,1,0,2) と ACAB(0,2,0,1) は同じ代表へ。
        let auts = aut_perms(&[2, 4, 4]);
        assert_eq!(canonical(&[0, 1, 0, 2], &auts), vec![0, 1, 0, 2]);
        assert_eq!(canonical(&[0, 2, 0, 1], &auts), vec![0, 1, 0, 2]);
    }
}
