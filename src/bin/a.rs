#![allow(non_snake_case)]

// [世界四連覇AIエンジニアがゼロから教えるゲーム木探索入門]
// (https://qiita.com/thun-c/items/058743a25c37c87b8aa4)
// を参考にしています。thunderさんに多大なる感謝を…
// Copyright [2021] <Copyright Eita Aoki (Thunder) >

use proconio::input;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

// 型の定義
type Action = usize;
type Actions = Vec<usize>;
type ScoreType = i32;
pub type Output = String;

// 定数
const INF: ScoreType = 1000000000;
const TILE_SIZE: usize = 50;
const DIJ: [(usize, usize); 4] = [(0, !0), (0, 1), (!0, 0), (1, 0)];
const DIR: [char; 4] = ['L', 'R', 'U', 'D'];

// 好みで変更する
const TIME_LIMIT: f64 = 1.9;
const SEED: u64 = 20210325;
const VIEW_POINTS: bool = false; // デバッグの時得点を表示するかどうか

/// 時間を管理するクラス
struct TimeKeeper {
    start_time_: f64,
    time_threshold_: f64,
}
impl TimeKeeper {
    /// 時間制限を秒単位で指定してインスタンスをつくる。
    pub fn new(time_threshold: f64) -> Self {
        TimeKeeper {
            start_time_: Self::get_time(),
            time_threshold_: time_threshold,
        }
    }
    /// インスタンス生成した時から指定した時間制限を超過したか判断する。
    pub fn isTimeOver(&self) -> bool {
        Self::get_time() - self.start_time_ - self.time_threshold_ >= 0.
    }
    /// 経過時間をミリ秒単位で返す
    pub fn time(&self) -> usize {
        ((Self::get_time() - self.start_time_) * 1000.) as usize
    }
    fn get_time() -> f64 {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        t.as_secs() as f64 + t.subsec_nanos() as f64 * 1e-9
    }
}

/// 入力で与えられる情報をまとめた構造体
/// s: 開始位置  
/// tiles: タイルの位置  
/// ps: 座標ごとの得点  
pub struct Input {
    pub s: (usize, usize),
    pub tiles: Vec<Vec<usize>>,
    pub ps: Vec<Vec<i32>>,
}

#[derive(Clone)]
/// 位置を表す構造体
struct Position {
    i_: usize,
    j_: usize,
}

#[derive(Clone)]
/// END_TURN_: 探索を終了するターン<br>
/// turn_: 現在のターン<br>
/// seen_: タイルを踏んだかどうか<br>
/// pos_: 現在位置<br>
/// output_: 経路の出力<br>
/// steps_: 移動経路の座標<br>
/// game_score_: 得点(実際の得点)<br>
/// evaluated_score_: 探索上で評価したスコア<br>
/// first_action_: 探索木のルートノードで最初に選択した行動<br>
struct TileState {
    END_TURN_: usize,
    turn_: usize,
    seen_: Vec<bool>,
    pos_: Position,
    pub output_: Output,
    pub steps_: Vec<(usize, usize)>,
    pub game_score_: i32,
    pub evaluated_score_: ScoreType,
    pub first_action_: Action,
}

impl TileState {
    pub fn new(input: &Input, end_turn: usize, pos: (usize, usize)) -> Self {
        let M_ = input
            .tiles
            .iter()
            .map(|t| t.iter().max().unwrap())
            .max()
            .unwrap()
            + 1;
        let mut seen_ = vec![false; M_];
        let pos_ = Position {
            i_: pos.0,
            j_: pos.1,
        };
        seen_[input.tiles[pos_.i_][pos_.j_]] = true;
        let steps_ = vec![(pos_.i_, pos_.j_)];
        let game_score_ = input.ps[pos_.i_][pos_.j_];
        let evaluated_score_ = 0;
        Self {
            END_TURN_: end_turn,
            turn_: 0,
            seen_,
            pos_,
            steps_,
            output_: String::new(),
            game_score_,
            evaluated_score_,
            first_action_: !0,
        }
    }
    /// [どのゲームでも実装する]: 探索用の盤面評価をする
    /// 探索ではゲーム本来のスコアに別の評価値をプラスするといい探索ができるので、ここに工夫の余地がある。
    pub fn evaluateScore(&mut self) {
        self.evaluated_score_ = self.game_score_;
    }

    /// [どのゲームでも実装する]: ゲームの終了判定
    pub fn isDone(&self) -> bool {
        self.turn_ == self.END_TURN_
    }

    /// [どのゲームでも実装する]: 指定したactionでゲームを1ターン進める
    pub fn advance(&mut self, input: &Input, action: Action) {
        self.pos_.i_ = self.pos_.i_.wrapping_add(DIJ[action].0);
        self.pos_.j_ = self.pos_.j_.wrapping_add(DIJ[action].1);
        self.steps_.push((self.pos_.i_, self.pos_.j_));
        self.game_score_ += input.ps[self.pos_.i_][self.pos_.j_];
        self.seen_[input.tiles[self.pos_.i_][self.pos_.j_]] = true;
        self.turn_ += 1;
        self.output_.push(DIR[action]);
    }

    /// [どのゲームでも実装する]: 現在の状況でプレイヤーが可能な行動を全て取得する
    pub fn legalActions(&self, input: &Input) -> Actions {
        let mut actions: Actions = vec![];
        for action in 0..4 {
            let ni = self.pos_.i_.wrapping_add(DIJ[action].0);
            let nj = self.pos_.j_.wrapping_add(DIJ[action].1);
            if ni < TILE_SIZE && nj < TILE_SIZE && !self.seen_[input.tiles[ni][nj]] {
                actions.push(action);
            }
        }
        actions
    }

    /// [実装しなくてもよいが実装すると便利]: 現在のゲーム状況を標準エラー出力に出力する
    pub fn toString(&self, input: &Input) {
        let mut path = vec![vec!["  "; TILE_SIZE]; TILE_SIZE];
        let string: Vec<Vec<String>> = input
            .ps
            .iter()
            .map(|pvec| pvec.iter().map(|p| format!("{:02}", p)).collect())
            .collect();
        if VIEW_POINTS {
            for i in 0..TILE_SIZE {
                for j in 0..TILE_SIZE {
                    path[i][j] = string[i][j].as_str();
                }
            }
        }
        // 移動経路に罫線を引く
        let (i, j) = input.s;
        path[i][j] = "@@";
        for i in 1..self.turn_ {
            let (h, w) = self.steps_[i];
            let mut dir = String::new();
            dir.push(self.output_.chars().nth(i - 1).unwrap());
            dir.push(self.output_.chars().nth(i).unwrap());
            // 直前の移動方向 + 今回の移動方向によって引く罫線を決定
            path[h][w] = match dir.as_str() {
                "LL" => "━━",
                "LU" => "┗━",
                "LD" => "┏━",
                "RR" => "━━",
                "RU" => "┛ ",
                "RD" => "┓ ",
                "UL" => "┓ ",
                "UR" => "┏━",
                "UU" => "┃ ",
                "DL" => "┛ ",
                "DR" => "┗━",
                "DD" => "┃ ",
                _ => unreachable!(),
            }
        }
        // 出力パート
        let isConnectHorizontal =
            |h: usize, w: usize| w + 1 < TILE_SIZE && input.tiles[h][w] == input.tiles[h][w + 1];
        let isConnectVertical =
            |h: usize, w: usize| h + 1 < TILE_SIZE && input.tiles[h][w] == input.tiles[h + 1][w];
        for h in 0..TILE_SIZE {
            for w in 0..TILE_SIZE {
                if !isConnectVertical(h, w) {
                    // 下のタイルとつながっていなかったら下線を引く
                    eprint!("\x1b[4m");
                }
                if self.seen_[input.tiles[h][w]] {
                    // 踏んだタイルなら色を塗る
                    eprint!("\x1b[46m");
                }
                eprint!("{}", path[h][w]);
                if isConnectHorizontal(h, w) {
                    // 右のタイルと繋がっていたら文字修飾を引き継いで空白を出力
                    eprint!(" ")
                } else {
                    // 右のタイルと繋がっていなかったら修飾をリセットして|を出力
                    eprint!("\x1b[0m");
                    eprint!("|");
                }
            }
            eprintln!();
        }
        eprintln!("turn : {}", self.turn_);
        eprintln!("score: {}", self.game_score_);
    }
}

/// [どのゲームでも実装する] : 探索時のソート用に評価を比較する
impl Ord for TileState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.evaluated_score_.cmp(&other.evaluated_score_)
    }
}
impl PartialEq for TileState {
    fn eq(&self, other: &Self) -> bool {
        self.evaluated_score_ == other.evaluated_score_
    }
}
impl Eq for TileState {} // ここは空でOK
impl PartialOrd for TileState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.evaluated_score_.partial_cmp(&other.evaluated_score_)
    }
}

type State = TileState;

#[allow(dead_code)]
/// ランダムに行動を決定する
fn randomAction(rng: &mut ChaCha20Rng, input: &Input, state: &State) -> Option<Action> {
    let legalActions = state.legalActions(input);
    if legalActions.is_empty() {
        return None;
    }
    return Some(legalActions[rng.gen_range(0, 100) as usize % (legalActions.len())]);
}

#[allow(dead_code)]
/// 貪欲法で行動を決定する
fn greedyAction(input: &Input, state: &State) -> Option<Action> {
    let mut best_score: ScoreType = -INF;
    let mut best_action = !0;
    let legalActions = state.legalActions(input);
    if legalActions.is_empty() {
        return None;
    }
    for action in legalActions {
        let mut now_state = state.clone();
        now_state.advance(input, action);
        now_state.evaluateScore();
        if now_state.evaluated_score_ > best_score {
            best_score = now_state.evaluated_score_;
            best_action = action;
        }
    }
    return Some(best_action);
}

#[allow(dead_code)]
/// ビーム幅と深さを指定してビームサーチで行動を決定する
fn beamSearchAction(
    input: &Input,
    state: &State,
    beam_width: usize,
    beam_depth: usize,
) -> Option<Action> {
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
            let legalActions = now_state.legalActions(input);
            for action in legalActions {
                let mut next_state = now_state.clone();
                next_state.advance(input, action);
                next_state.evaluateScore();
                if t == 0 {
                    next_state.first_action_ = action;
                }
                next_beam.push(next_state);
            }
        }

        now_beam = next_beam;
        best_state = now_beam.peek().unwrap();

        if best_state.isDone() {
            break;
        }
    }
    let ans_action = best_state.first_action_;
    if ans_action == !0 {
        return None;
        //panic!("can't find action in beam_search")
    }
    return Some(best_state.first_action_);
}

#[allow(dead_code)]
/// ビーム幅と制限時間(s)を指定してビームサーチで行動を決定する
fn beamSearchActionWithTimeThreshold(
    input: &Input,
    state: &State,
    beam_width: usize,
    time_threshold: f64,
) -> Option<Action> {
    use std::collections::BinaryHeap;
    let timekeeper = TimeKeeper::new(time_threshold);
    let mut now_beam = BinaryHeap::new();
    let mut best_state = state.clone();
    now_beam.push(state.clone());

    for t in 0.. {
        let mut next_beam = BinaryHeap::new();
        for _ in 0..beam_width {
            if timekeeper.isTimeOver() {
                return Some(best_state.first_action_);
            }
            if now_beam.is_empty() {
                break;
            }
            let now_state = now_beam.pop().unwrap();
            let legalActions = now_state.legalActions(input);
            for action in legalActions {
                let mut next_state = now_state.clone();
                next_state.advance(input, action);
                next_state.evaluateScore();
                if t == 0 {
                    next_state.first_action_ = action
                }
                next_beam.push(next_state);
            }
        }

        now_beam = next_beam;
        if now_beam.is_empty() {
            break;
        }
        best_state = now_beam.peek().unwrap().clone();

        if best_state.isDone() {
            break;
        }
    }
    let ans_action = best_state.first_action_;
    if ans_action == !0 {
        return None;
        //panic!("can't find action in beam search")
    }
    Some(ans_action)
}

#[allow(dead_code)]
/// ビーム1本あたりのビーム幅とビームの本数を指定してchokudaiサーチで行動を決定する
fn chokudaiSearchAction(
    input: &Input,
    state: &State,
    beam_width: usize,
    beam_depth: usize,
    beam_number: usize,
) -> Option<Action> {
    use std::collections::BinaryHeap;
    let mut beam = vec![BinaryHeap::new(); beam_depth + 1];
    beam[0].push(state.clone());
    for _ in 0..beam_number {
        // thunderさんのコードだとここで
        // now_beam = &beam[t]
        // next_beam = &beam[t+1]
        // としていて、その方が見やすいのだが、Rustでは二重借用ができない
        for t in 0..beam_depth {
            for _ in 0..beam_width {
                if let Some(now_state) = beam[t].pop() {
                    if now_state.isDone() {
                        beam[t].push(now_state);
                        break;
                    }
                    let legalActions = now_state.legalActions(input);
                    for action in legalActions {
                        let mut next_state = now_state.clone();
                        next_state.advance(input, action);
                        next_state.evaluateScore();
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
            return Some(now_beam.peek().unwrap().first_action_);
        }
    }
    None
}

/// ビーム1本あたりのビーム幅と制限時間(s)を指定してchokudaiサーチで行動を決定する
#[allow(dead_code, non_snake_case)]
fn chokudaiSearchActionWithTimeThreshold(
    input: &Input,
    state: &State,
    beam_width: usize,
    beam_depth: usize,
    time_threshold: f64,
) -> Option<Action> {
    use std::collections::BinaryHeap;
    let timekeeper = TimeKeeper::new(time_threshold);
    let mut beam = vec![BinaryHeap::new(); beam_depth + 1];
    beam[0].push(state.clone());
    loop {
        for t in 0..beam_depth {
            for _ in 0..beam_width {
                if let Some(now_state) = beam[t].pop() {
                    if now_state.isDone() {
                        beam[t].push(now_state);
                        break;
                    }
                    let legalActions = now_state.legalActions(input);
                    for action in legalActions {
                        let mut next_state = now_state.clone();
                        next_state.advance(input, action);
                        next_state.evaluateScore();
                        if t == 0 {
                            next_state.first_action_ = action
                        }
                        beam[t + 1].push(next_state)
                    }
                }
            }
        }
        if timekeeper.isTimeOver() {
            break;
        }
    }
    for t in (0..=beam_depth).rev() {
        if !beam[t].is_empty() {
            return Some(beam[t].peek().unwrap().first_action_);
        }
    }
    None
}
fn main() {
    input! {
        s: (usize, usize),
        tiles: [[usize; TILE_SIZE]; TILE_SIZE],
        ps: [[i32; TILE_SIZE]; TILE_SIZE],
    }
    let timekeeper = TimeKeeper::new(TIME_LIMIT);
    let mut rng = ChaCha20Rng::seed_from_u64(SEED);

    let input = Input { s, tiles, ps };
    let mut state = State::new(&input, !0, input.s);
    state.evaluateScore();
    let mut loop_cnt = 0;
    // 好きな実装を選択しよう！
    // ハイパーパラメータ(ビーム幅など)は適当です。
    //while let Some(action) = greedyAction(&input, &state) {
    //while let Some(action) = beamSearchAction(&input, &state, 3, 3) {
    //while let Some(action) = beamSearchActionWithTimeThreshold(&input, &state, 3, 0.02) {
    //while let Some(action) = chokudaiSearchActionWithTimeThreshold(&input, &state, 3, 3, 0.02) {
    //while let Some(action) = chokudaiSearchAction(&input, &state, 10, 10, 50) {
    while let Some(action) = randomAction(&mut rng, &input, &state) {
        loop_cnt += 1;
        if timekeeper.isTimeOver() {
            break;
        }
        state.advance(&input, action);
        state.evaluateScore();
    }
    state.toString(&input);
    println!("{}", state.output_);
    eprintln!("{} loop", loop_cnt);
    eprintln!("{} ms", timekeeper.time());
}
