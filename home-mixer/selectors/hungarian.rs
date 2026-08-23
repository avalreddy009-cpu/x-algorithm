//! Kuhn–Munkres (Hungarian) algorithm: optimal bipartite assignment.
//!
//! Real use in a feed: assign posts to *typed slots* (friends / rare-event /
//! explore / general). Sorting by score cannot do that. MMR is greedy; this
//! is the globally optimal matching for a slot-profit matrix.
//!
//! O(n³). Skip when n is large (see `HungarianMaxN`).

const INF: f64 = 1e100;

/// Minimize an n×n cost matrix. Returns `col_of_row[row] = col`.
pub fn hungarian_minimize(cost: &[Vec<f64>]) -> Vec<usize> {
    let n = cost.len();
    assert!(n > 0 && cost.iter().all(|r| r.len() == n));
    let mut u = vec![0.0; n + 1];
    let mut v = vec![0.0; n + 1];
    let mut p = vec![0usize; n + 1];
    let mut way = vec![0usize; n + 1];

    for i in 1..=n {
        p[0] = i;
        let mut j0 = 0usize;
        let mut minv = vec![INF; n + 1];
        let mut used = vec![false; n + 1];
        loop {
            used[j0] = true;
            let i0 = p[j0];
            let mut delta = INF;
            let mut j1 = 0usize;
            for j in 1..=n {
                if used[j] {
                    continue;
                }
                let cur = cost[i0 - 1][j - 1] - u[i0] - v[j];
                if cur < minv[j] {
                    minv[j] = cur;
                    way[j] = j0;
                }
                if minv[j] < delta {
                    delta = minv[j];
                    j1 = j;
                }
            }
            for j in 0..=n {
                if used[j] {
                    u[p[j]] += delta;
                    v[j] -= delta;
                } else {
                    minv[j] -= delta;
                }
            }
            j0 = j1;
            if p[j0] == 0 {
                break;
            }
        }
        loop {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
            if j0 == 0 {
                break;
            }
        }
    }

    let mut col_of_row = vec![0usize; n];
    for j in 1..=n {
        if p[j] > 0 {
            col_of_row[p[j] - 1] = j - 1;
        }
    }
    col_of_row
}

/// Maximize profit. Dummy rows/cols should be filled with 0.
pub fn hungarian_maximize(profit: &[Vec<f64>]) -> Vec<usize> {
    let mut max_v = 0.0_f64;
    for row in profit {
        for &x in row {
            if x.is_finite() && x > max_v {
                max_v = x;
            }
        }
    }
    let cost: Vec<Vec<f64>> = profit
        .iter()
        .map(|row| row.iter().map(|&x| max_v - x).collect())
        .collect();
    hungarian_minimize(&cost)
}

/// Pad a rectangular profit matrix to square with zeros.
pub fn pad_square(profit: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rows = profit.len();
    let cols = profit.first().map(|r| r.len()).unwrap_or(0);
    let n = rows.max(cols);
    let mut out = vec![vec![0.0; n]; n];
    for i in 0..rows {
        for j in 0..cols.min(profit[i].len()) {
            out[i][j] = profit[i][j];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_by_two_swaps_to_max_profit() {
        // Best is 2+3=5, not 1+0=1.
        let profit = vec![vec![1.0, 2.0], vec![3.0, 0.0]];
        let assign = hungarian_maximize(&profit);
        assert_eq!(assign[0], 1);
        assert_eq!(assign[1], 0);
    }

    #[test]
    fn identity_on_diagonal_dominant() {
        let profit = vec![
            vec![10.0, 1.0, 1.0],
            vec![1.0, 10.0, 1.0],
            vec![1.0, 1.0, 10.0],
        ];
        let assign = hungarian_maximize(&profit);
        assert_eq!(assign, vec![0, 1, 2]);
    }

    #[test]
    fn pad_square_keeps_values() {
        let p = pad_square(&[vec![5.0, 1.0]]);
        assert_eq!(p.len(), 2);
        assert_eq!(p[0][0], 5.0);
    }
}
