use proconio::input;
use rand::prelude::*;

const N: usize = 50;
pub type Output = String;
const DIJ: [(usize, usize); 4] = [(0, !0), (0, 1), (!0, 0), (1, 0)];

pub struct Input {
    pub s: (usize, usize),
    pub tiles: Vec<Vec<usize>>,
    pub ps: Vec<Vec<i32>>,
}

fn main() {
    input! {
        s: (usize, usize),
        tiles: [[usize; N]; N],
        ps: [[i32; N]; N],
    }
    let input = Input { s, tiles, ps };
}

pub fn compute_score_detail(
    input: &Input,
    out: &Output,
) -> (i32, String, Vec<usize>, Vec<(usize, usize)>) {
    let mut used = vec![0; N * N];
    let (mut i, mut j) = input.s;
    used[input.tiles[i][j]] = 1;
    let mut score = input.ps[i][j];
    let mut steps = vec![(i, j)];
    let mut err = String::new();
    for c in out.chars() {
        let (di, dj) = match c {
            'L' => (0, !0),
            'R' => (0, 1),
            'U' => (!0, 0),
            'D' => (1, 0),
            _ => {
                return (0, "Illegal output".to_owned(), used, steps);
            }
        };
        i += di;
        j += dj;
        if i >= N || j >= N {
            return (0, "Out of range".to_owned(), used, steps);
        }
        steps.push((i, j));
        if used[input.tiles[i][j]] != 0 {
            err = "Stepped on the same tile twice".to_owned();
        }
        used[input.tiles[i][j]] += 1;
        score += input.ps[i][j];
    }
    if err.len() > 0 {
        score = 0;
    }
    (score, err, used, steps)
}
