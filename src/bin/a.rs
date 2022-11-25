use proconio::input;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
const N: usize = 50;
pub type Output = String;
const DIJ: [(usize, usize); 4] = [(0, !0), (0, 1), (!0, 0), (1, 0)];

#[allow(dead_code)]
/// 時間を管理するクラス
struct TimeKeeper {
    start_time_: f64,
    time_threshold_: f64,
}
#[allow(dead_code)]
impl TimeKeeper {
    /// 時間制限をミリ秒単位で指定してインスタンスをつくる。
    pub fn new(time_threshold: f64) -> Self {
        TimeKeeper {
            start_time_: get_time(),
            time_threshold_: time_threshold,
        }
    }
    /// インスタンス生成した時から指定した時間制限を超過したか判断する。
    pub fn is_time_over(&self) -> bool {
        get_time() - self.start_time_ - self.time_threshold_ >= 0.
    }
}
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
    let mut rng = ChaCha20Rng::seed_from_u64(20210325);
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

#[allow(unused)]

type Action = usize;
type Actions = Vec<usize>;
type ScoreType = i32;
const INF: ScoreType = 1000000000;

pub fn get_time() -> f64 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    t.as_secs() as f64 + t.subsec_nanos() as f64 * 1e-9
}

#[derive(Clone)]
struct Position {
    i_: usize,
    j_: usize,
}

#[derive(Clone)]
#[allow(non_snake_case)]
/// N: タイルの幅(50で固定)
/// M: タイルの枚数
/// END_TURN_: 探索を終了するターン
/// turn_: 現在のターン
///
struct TileState {
    M_: usize,
    END_TURN_: usize,
    seen: Vec<bool>,
    turn_: usize,
    pos_: Position,
    pub game_score_: i32,
    pub evaluate_score_: ScoreType,
    pub first_action_: Action,
}

#[allow(dead_code)]
impl TileState {
    /// [どのゲームでも実装する]: 探索用の盤面評価をする
    pub fn evaluate_score(&mut self) {
        self.evaluate_score_ = self.game_score_;
    }

    /// [どのゲームでも実装する]: ゲームの終了判定
    pub fn is_done(&self) -> bool {
        self.turn_ == self.END_TURN_
    }

    /// [どのゲームでも実装する]: 指定したactionでゲームを1ターン進める
    pub fn advance(&mut self, input: &Input, action: Action) {
        self.pos_.i_ += DIJ[action].0;
        self.pos_.j_ += DIJ[action].1;
        self.game_score_ += input.ps[self.pos_.i_][self.pos_.j_];
        self.turn_ += 1;
    }

    /// [どのゲームでも実装する]: 現在の状況でプレイヤーが可能な行動を全て取得する
    pub fn legal_actions(&self, input: &Input) -> Actions {
        let mut actions: Actions = vec![];
        for action in 0..4 {
            let ni = self.pos_.i_ + DIJ[action].0;
            let nj = self.pos_.j_ + DIJ[action].1;
            if ni < N && nj < N && !self.seen[input.tiles[ni][nj]] {
                actions.push(action);
            }
        }
        actions
    }

    /// [実装しなくてもよいが実装すると便利]: 現在のゲーム状況を標準エラー出力に出力する
    pub fn to_string(&self) {
        !todo!();
        // eprintln!("turn : {}", self.turn_);
        // eprintln!("score: {}", self.game_score_);
        // for h in 0..self.h_ {
        //     for w in 0..self.w_ {
        //         let mut c = '.';
        //         if self.walls_[h][w] == 1 {
        //             c = '#';
        //         }
        //         if self.character_.y_ == h && self.character_.x_ == w {
        //             c = '@';
        //         }
        //         if self.points_[h][w] != 0 {
        //             c = (b'0' + self.points_[h][w] as u8) as char;
        //         }
        //         eprint!("{}", c);
        //     }
        //     eprintln!();
        // }
    }
}

/// [どのゲームでも実装する] : 探索時のソート用に評価を比較する
impl Ord for TileState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.evaluate_score_.cmp(&other.evaluate_score_)
    }
}
impl PartialEq for TileState {
    fn eq(&self, other: &Self) -> bool {
        self.evaluate_score_ == other.evaluate_score_
    }
}
impl Eq for TileState {} // ここは空でOK
impl PartialOrd for TileState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.evaluate_score_.partial_cmp(&other.evaluate_score_)
    }
}

type State = TileState;
#[allow(dead_code, non_snake_case)]
/// ランダムに行動を決定する
fn randomAction(rng: &mut ChaCha20Rng, input: &Input, state: &State) -> Action {
    let legal_actions = state.legal_actions(input);
    return legal_actions[rng.gen_range(0, 100) as usize % (legal_actions.len())];
}

#[allow(dead_code, non_snake_case)]
/// 貪欲法で行動を決定する
fn greedyAction(input: &Input, state: &State) -> Action {
    let legal_actions = state.legal_actions(input);
    let mut best_score: ScoreType = -INF;
    let mut best_action = !0;
    for action in legal_actions {
        let mut now_state = state.clone();
        now_state.advance(input, action);
        now_state.evaluate_score();
        if now_state.evaluate_score_ > best_score {
            best_score = now_state.evaluate_score_;
            best_action = action;
        }
    }
    best_action
}

#[allow(dead_code, non_snake_case)]
/// ビーム幅と深さを指定してビームサーチで行動を決定する
fn beamSearchAction(input: &Input, state: &State, beam_width: usize, beam_depth: usize) -> Action {
    use std::collections::BinaryHeap;
    let mut now_beam = BinaryHeap::new();
    let mut best_state = state;
    now_beam.push(state.clone());
    for t in 0..beam_depth {
        let mut next_beam = BinaryHeap::new();
        for _ in 0..beam_width {
            if now_beam.is_empty() {
                break;
            }
            let now_state = now_beam.pop().unwrap();
            let legal_actions = now_state.legal_actions(input);
            for action in legal_actions {
                let mut next_state = now_state.clone();
                next_state.advance(input, action);
                next_state.evaluate_score();
                if t == 0 {
                    next_state.first_action_ = action;
                }
                next_beam.push(next_state);
            }
        }

        now_beam = next_beam;
        best_state = now_beam.peek().unwrap();

        if best_state.is_done() {
            break;
        }
    }
    let ans_action = best_state.first_action_;
    if ans_action == !0 {
        panic!("can't find action in beam_search")
    }
    return best_state.first_action_;
}

#[allow(dead_code, non_snake_case)]
/// ビーム幅と制限時間(s)を指定してビームサーチで行動を決定する
fn beamSearchActionWithTimeThreshold(
    input: &Input,
    state: &State,
    beam_width: usize,
    time_threshold: f64,
) -> Action {
    use std::collections::BinaryHeap;
    let timekeeper = TimeKeeper::new(time_threshold);
    let mut now_beam = BinaryHeap::new();
    let mut best_state = state.clone();
    now_beam.push(state.clone());

    for t in 0.. {
        let mut next_beam = BinaryHeap::new();
        for _ in 0..beam_width {
            if timekeeper.is_time_over() {
                return best_state.first_action_;
            }
            if now_beam.is_empty() {
                break;
            }
            let now_state = now_beam.pop().unwrap();
            let legal_actions = now_state.legal_actions(input);
            for action in legal_actions {
                let mut next_state = now_state.clone();
                next_state.advance(input, action);
                next_state.evaluate_score();
                if t == 0 {
                    next_state.first_action_ = action
                }
                next_beam.push(next_state);
            }
        }

        now_beam = next_beam;
        best_state = now_beam.peek().unwrap().clone();

        if best_state.is_done() {
            break;
        }
    }
    let ans_action = best_state.first_action_;
    if ans_action == !0 {
        panic!("can't find action in beam search")
    }
    ans_action
}

#[allow(dead_code, non_snake_case)]
/// ビーム1本あたりのビーム幅とビームの本数を指定してchokudaiサーチで行動を決定する
fn chokudaiSearchAction(
    input: &Input,
    state: &State,
    beam_width: usize,
    beam_depth: usize,
    beam_number: usize,
) -> Action {
    use std::collections::BinaryHeap;
    let mut beam = vec![BinaryHeap::new(); beam_depth + 1];
    beam[0].push(state.clone());
    for _ in 0..beam_number {
        for t in 0..beam_depth {
            for _ in 0..beam_width {
                if let Some(now_state) = beam[t].pop() {
                    if now_state.is_done() {
                        beam[t].push(now_state);
                        break;
                    }
                    let legal_actions = now_state.legal_actions(input);
                    for action in legal_actions {
                        let mut next_state = now_state.clone();
                        next_state.advance(input, action);
                        next_state.evaluate_score();
                        if t == 0 {
                            next_state.first_action_ = action
                        }
                        beam[t + 1].push(next_state)
                    }
                }
            }
        }
    }
    for t in (0..=beam_depth).rev() {
        let now_beam = &beam[t];
        if !now_beam.is_empty() {
            return now_beam.peek().unwrap().first_action_;
        }
    }
    !0
}

/// ビーム1本あたりのビーム幅と制限時間(s)を指定してchokudaiサーチで行動を決定する
#[allow(dead_code, non_snake_case)]
fn chokudaiSearchActionWithTimeThreshold(
    input: &Input,
    state: &State,
    beam_width: usize,
    beam_depth: usize,
    time_threshold: f64,
) -> Action {
    use std::collections::BinaryHeap;
    let timekeeper = TimeKeeper::new(time_threshold);
    let mut beam = vec![BinaryHeap::new(); beam_depth + 1];
    beam[0].push(state.clone());
    loop {
        for t in 0..beam_depth {
            for _ in 0..beam_width {
                if let Some(now_state) = beam[t].pop() {
                    if now_state.is_done() {
                        beam[t].push(now_state);
                        break;
                    }
                    let legal_actions = now_state.legal_actions(input);
                    for action in legal_actions {
                        let mut next_state = now_state.clone();
                        next_state.advance(input, action);
                        next_state.evaluate_score();
                        if t == 0 {
                            next_state.first_action_ = action
                        }
                        beam[t + 1].push(next_state)
                    }
                }
            }
        }
        if timekeeper.is_time_over() {
            break;
        }
    }
    for t in (0..=beam_depth).rev() {
        if !beam[t].is_empty() {
            return beam[t].peek().unwrap().first_action_;
        }
    }
    return 6;
}
