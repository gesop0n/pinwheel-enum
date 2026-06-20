//! スケジュール（その日に行う仕事の列）

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Schedule(pub Vec<u32>);

impl Schedule {
    pub fn canonical(periods: &[u32]) -> Self {
        let n = periods.len();
        if n == 0 {
            return Schedule(Vec::new());
        }
        let mut best = 0usize;
        for start in 1..n {
            if rotation_less(periods, start, best) {
                best = start
            }
        }
        let rotated = (0..n).map(|i| periods[(best + i) % n]).collect();
        Schedule(rotated)
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// `periods` を添字 `a` から始めた回転が、`b` から始めた回転より辞書順で小さいか
fn rotation_less(periods: &[u32], a: usize, b: usize) -> bool {
    let n = periods.len();
    for i in 0..n {
        let x = periods[(a + i) % n];
        let y = periods[(b + i) % n];
        if x != y {
            return x < y;
        }
    }
    false // 完全に等しい回転
}
