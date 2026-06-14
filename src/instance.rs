use num_rational::Ratio;

/// Pinwheel Instance
/// 周期は常に昇順で保持する
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PinwheelInstance {
    periods: Vec<u32>,
}

impl PinwheelInstance {
    /// 任意順のインスタンスから生成し、昇順に正規化する
    pub fn new(mut periods: Vec<u32>) -> Self {
        periods.sort_unstable();
        Self { periods }
    }

    pub fn periods(&self) -> &[u32] {
        &self.periods
    }

    /// density(c): 密度
    /// 厳密な有理数比較のため、
    /// Ratio<i64> (cppの boost::rational<long long>のRust版)
    /// で計算
    pub fn density(&self) -> Ratio<i64> {
        self.periods.iter().map(|&a| Ratio::new(1, a as i64)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_value() {
        assert_eq!(
            PinwheelInstance::new(vec![6, 3, 2]).density(),
            Ratio::new(1, 1)
        );
        assert_eq!(
            PinwheelInstance::new(vec![6, 3, 3]).density(),
            Ratio::new(5, 6)
        );
    }
}
