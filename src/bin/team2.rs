#![allow(non_snake_case)]

// [世界四連覇AIエンジニアがゼロから教えるゲーム木探索入門]
// (https://qiita.com/thun-c/items/058743a25c37c87b8aa4)
// を参考にしています。thunderさんに多大なる感謝を…
// Copyright [2021] <Copyright Eita Aoki (Thunder) >

use num::ToPrimitive;
use proconio::input;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::{cmp::Reverse, collections::BinaryHeap, f32::MIN};

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
const START_TEMP: f64 = 500.0;
const END_TEMP: f64 = 10.0;

// 評価関数を切りかえる用途
const USE_TARGET_DISTANCE: usize = 1;
const USE_GAME_SCORE: usize = 2;

// 好みで変更する
const TIME_LIMIT: f64 = 1.988;
const SEED: u64 = 20210325;
const VIEW_POINTS: bool = false; // デバッグの時得点を表示するかどうか

const fn calc_center(i: usize, j: usize) -> (usize, usize) {
    (i * TILE_SIZE / 8, j * TILE_SIZE / 8)
}

// 16個のブロックの中心座標
const CENTER: [(usize, usize); 16] = [
    calc_center(1, 1),
    calc_center(1, 3),
    calc_center(1, 5),
    calc_center(1, 7),
    calc_center(3, 1),
    calc_center(3, 3),
    calc_center(3, 5),
    calc_center(3, 7),
    calc_center(5, 1),
    calc_center(5, 3),
    calc_center(5, 5),
    calc_center(5, 7),
    calc_center(7, 1),
    calc_center(7, 3),
    calc_center(7, 5),
    calc_center(7, 7),
];

/**

盤面を16分割して、この順番でまわる

 0 ←  1    2 ←  3
 ↓    ↑    ↓    ↑
 4    5 ←  6    7
 ↓              ↑
 8    9 → 10   11
 ↓    ↑    ↓    ↑
12 → 13   14 → 15

**/
// 16個のブロックの次のブロックの中心
const TARGET: [(usize, usize); 16] = [
    //0        1           2          3           4           5           6           7
    CENTER[4], CENTER[0], CENTER[6], CENTER[2], CENTER[8], CENTER[1], CENTER[5], CENTER[3],
    //8          9          10           11         12          13          14          15
    CENTER[12], CENTER[10], CENTER[14], CENTER[7], CENTER[13], CENTER[9], CENTER[15], CENTER[11],
];

// 時間を管理するクラス
struct TimeKeeper {
    start_time_: f64,
    time_threshold_: f64,
}
impl TimeKeeper {
    // 時間制限を秒単位で指定してインスタンスをつくる。
    pub fn new(time_threshold: f64) -> Self {
        TimeKeeper {
            start_time_: Self::get_time(),
            time_threshold_: time_threshold,
        }
    }
    // インスタンス生成した時から指定した時間制限を超過したか判断する。
    pub fn isTimeOver(&self) -> bool {
        Self::get_time() - self.start_time_ - self.time_threshold_ >= 0.
    }
    // 経過時間をミリ秒単位で返す
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

// 入力で与えられる情報をまとめた構造体
// s: 開始位置
// tiles: タイルの位置
// points: 座標ごとの得点
pub struct Input {
    pub s: (usize, usize),
    pub tiles: Vec<Vec<usize>>,
    pub points: Vec<Vec<i32>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
// 位置を表す構造体
struct Position {
    i: usize,
    j: usize,
}
impl Position {
    pub fn new((i, j): (usize, usize)) -> Self {
        Position { i, j }
    }
}

#[derive(Clone)]
// END_TURN_: 探索を終了するターン<br>
// turn: 現在のターン<br>
// seen: タイルを踏んだかどうか<br>
// pos: 現在位置<br>
// goal: 目標位置
// output: 経路の出力<br>
// steps: 移動経路の座標<br>
// game_score: 得点(実際の得点)<br>
// evaluated_score: 探索上で評価したスコア<br>
// first_action_: 探索木のルートノードで最初に選択した行動<br>
struct TileState {
    END_TURN_: usize,
    turn: usize,
    seen: Vec<bool>,
    pos: Position,
    pub goal: Position,
    pub output: Output,
    pub steps: Vec<(usize, usize)>,
    pub game_score: i32,
    pub evaluated_score: ScoreType,
    pub first_action_: Action,
}

impl TileState {
    pub fn new(
        input: &Input,
        end_turn: usize,
        pos_init: (usize, usize),
        goal: (usize, usize),
    ) -> Self {
        let M_ = input
            .tiles
            .iter()
            .map(|t| t.iter().max().unwrap())
            .max()
            .unwrap()
            + 1;
        let mut seen = vec![false; M_];
        let pos = Position {
            i: pos_init.0,
            j: pos_init.1,
        };
        let goal = Position {
            i: goal.0,
            j: goal.1,
        };
        seen[input.tiles[pos.i][pos.j]] = true;
        let steps = vec![(pos.i, pos.j)];
        let game_score = input.points[pos.i][pos.j];
        let evaluated_score = 0;
        Self {
            END_TURN_: end_turn,
            turn: 0,
            seen,
            pos,
            goal,
            steps,
            output: String::from("*"),
            game_score,
            evaluated_score,
            first_action_: !0,
        }
    }

    // 現在地と目標座標のマンハッタン距離
    pub fn goalDistance(&self) -> i32 {
        (self.pos.i as i32 - self.goal.i as i32).abs()
            + (self.pos.j as i32 - self.goal.j as i32).abs()
    }

    // [どのゲームでも実装する]: 探索用の盤面評価をする
    // 探索ではゲーム本来のスコアに別の評価値をプラスするといい探索ができるので、ここに工夫の余地がある。
    pub fn evaluateScore(&mut self, flag: usize) {
        match flag {
            // ゲームスコアをそのまま使う
            USE_GAME_SCORE => {
                self.evaluated_score = self.game_score;
            }
            // 現在地と目標座標のマンハッタン距離
            USE_TARGET_DISTANCE => {
                self.evaluated_score = self.goalDistance();
            }
            _ => {
                self.evaluated_score = self.game_score;
            }
        }
    }

    // NOTE: スコアを積算するとなぜかあわないので、しょうがなく実装. たくさん呼ぶと実行回数が減る
    // スコア更新
    pub fn updateGameScore(&mut self, input: &Input) {
        self.game_score = self.steps.iter().map(|&(i, j)| input.points[i][j]).sum();
    }

    // [どのゲームでも実装する]: ゲームの終了判定
    pub fn isDone(&self) -> bool {
        self.turn == self.END_TURN_
    }

    // [どのゲームでも実装する]: 指定したactionでゲームを1ターン進める
    pub fn advance(&mut self, input: &Input, action: Action) {
        self.pos.i = self.pos.i.wrapping_add(DIJ[action].0);
        self.pos.j = self.pos.j.wrapping_add(DIJ[action].1);
        self.steps.push((self.pos.i, self.pos.j));
        self.game_score += input.points[self.pos.i][self.pos.j];
        self.seen[input.tiles[self.pos.i][self.pos.j]] = true;
        self.turn += 1;
        self.output.push(DIR[action]);
    }

    // [どのゲームでも実装する]: 現在の状況でプレイヤーが可能な行動を全て取得する
    pub fn legalActions(&self, input: &Input) -> Actions {
        let mut actions: Actions = vec![];
        for action in 0..4 {
            let ni = self.pos.i.wrapping_add(DIJ[action].0);
            let nj = self.pos.j.wrapping_add(DIJ[action].1);
            if ni < TILE_SIZE && nj < TILE_SIZE && !self.seen[input.tiles[ni][nj]] {
                actions.push(action);
            }
        }
        actions
    }

    // [実装しなくてもよいが実装すると便利]: 現在のゲーム状況を標準エラー出力に出力する
    pub fn toString(&self, input: &Input) {
        let mut path = vec![vec!["  "; TILE_SIZE]; TILE_SIZE];
        let string: Vec<Vec<String>> = input
            .points
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
        // NOTE: この添字どうにかしたい
        for i in 2..self.output.len() {
            let (h, w) = self.steps[i - 1];
            let mut dir = String::new();
            dir.push(self.output.chars().nth(i - 1).unwrap());
            dir.push(self.output.chars().nth(i).unwrap());
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

        eprint!("   ");
        for w in 0..TILE_SIZE {
            eprint!("{:2} ", w);
        }
        eprintln!();
        for h in 0..TILE_SIZE {
            eprint!("{:2} ", h);
            for w in 0..TILE_SIZE {
                if !isConnectVertical(h, w) {
                    // 下のタイルとつながっていなかったら下線を引く
                    eprint!("\x1b[4m");
                }
                if self.seen[input.tiles[h][w]] {
                    // 踏んだタイルなら色を塗る
                    eprint!("\x1b[46m");
                }
                eprint!("{}", path[h][w]);
                if isConnectHorizontal(h, w) {
                    // 右のタイルと繋がっていたら文字修飾を引き継いで空白を出力
                    eprint!(" ");
                } else {
                    // 右のタイルと繋がっていなかったら修飾をリセットして|を出力
                    eprint!("\x1b[0m");
                    eprint!("|");
                }
            }
            eprintln!();
        }
        eprintln!("score: {}", self.game_score);
    }
}

// [どのゲームでも実装する] : 探索時のソート用に評価を比較する
impl Ord for TileState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.evaluated_score.cmp(&other.evaluated_score)
    }
}
impl PartialEq for TileState {
    fn eq(&self, other: &Self) -> bool {
        self.evaluated_score == other.evaluated_score
    }
}
impl Eq for TileState {} // ここは空でOK
impl PartialOrd for TileState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.evaluated_score.partial_cmp(&other.evaluated_score)
    }
}

type State = TileState;

fn dfs(input: &Input, state: &mut State, move_limit: i32, timekeeper: &TimeKeeper) {
    let mut stack = vec![(state.clone(), 0_i32)];
    let start = timekeeper.time();

    while let Some((now_state, move_cnt)) = stack.pop() {
        if timekeeper.isTimeOver() {
            break;
        }
        // 再構築完了
        if now_state.pos == state.goal {
            if *state < now_state {
                *state = now_state;
            }
            continue;
        }
        // 移動上限 or 残り回数で辿りつけない
        if move_cnt == move_limit || now_state.goalDistance() > move_limit - move_cnt {
            continue;
        }
        now_state
            .legalActions(&input)
            .iter()
            .map(|&action| {
                let mut next_state = now_state.clone();
                next_state.advance(&input, action);
                next_state.evaluateScore(USE_GAME_SCORE);
                next_state
            })
            .for_each(|next_state| stack.push((next_state, move_cnt + 1)));
    }
}

fn dfs4sa(input: &Input, state: &mut State, move_limit: i32, timekeeper: &TimeKeeper) {
    let mut stack = vec![(state.clone(), 0_i32)];
    let start = timekeeper.time();

    while let Some((now_state, move_cnt)) = stack.pop() {
        if timekeeper.isTimeOver() {
            break;
        }
        // ゴールに達したらすぐにリターンする（スコアによらず）
        if now_state.pos == state.goal {
            *state = now_state;
            return;
        }
        // 移動上限 or 残り回数で辿りつけない
        if move_cnt == move_limit || now_state.goalDistance() > move_limit - move_cnt {
            continue;
        }
        let mut rng = thread_rng();
        let mut legal_actions_random = now_state.legalActions(&input);
        legal_actions_random.shuffle(&mut rng);
        legal_actions_random
            .iter()
            .map(|&action| {
                let mut next_state = now_state.clone();
                next_state.advance(&input, action);
                next_state.evaluateScore(USE_GAME_SCORE);
                next_state
            })
            .for_each(|next_state| stack.push((next_state, move_cnt + 1)));
    }
}

// reset_length は(8,9,10 のどれか)
fn climb(
    input: &Input,
    rng: &mut ChaCha20Rng,
    state: &mut State,
    reset_length: usize,
    timekeeper: &TimeKeeper,
    cnt: usize,
) {
    // NOTE: -2はおそらく、マスの大きさを考慮したマージン
    // 破壊する区間 [start, end)
    let start = rng.gen_range(1, state.steps.len() - reset_length - 1);
    let end = start + reset_length;
    let (si, sj) = state.steps[start - 1];
    let (ei, ej) = state.steps[end - 1];

    // 再構築する盤面
    let mut change_state = state.clone();
    change_state.pos = Position::new((si, sj));
    change_state.goal = Position::new((ei, ej));

    // 盤面リセット (探索履歴, ゲームスコア)
    for i in start..end {
        let (pi, pj) = state.steps[i];
        change_state.seen[input.tiles[pi][pj]] = false;
        change_state.game_score -= input.points[pi][pj] as i32;
    }

    // 破壊する開始位置までを保存しておく
    change_state.output = String::from(&change_state.output[..start]);
    change_state.steps = state.steps[..start].to_vec();
    change_state.evaluateScore(USE_GAME_SCORE);

    // 破壊した部分をdfsで再構築
    dfs(
        &input,
        &mut change_state,
        (reset_length * 2) as i32,
        &timekeeper,
    );

    // 破壊してない部分をくっつける
    change_state.output += &state.output[end..];
    change_state.steps.extend_from_slice(&state.steps[end..]);
    change_state.evaluateScore(USE_GAME_SCORE);

    // スコア更新(これ大事) → スコア計算のバグとったので、多分不要
    // change_state.updateGameScore(input);

    // 破壊した部分をdfsで再構築した結果が良かったら更新
    if state.game_score < change_state.game_score {
        *state = change_state;
        eprintln!(
            "update {} : {} ms, {} times, score:{}",
            start,
            timekeeper.time(),
            cnt,
            state.game_score
        );
    }

    //
}

// reset_length は(8,9,10 のどれか)
fn simulatedAnnealing(
    input: &Input,
    rng: &mut ChaCha20Rng,
    state: &mut State,
    reset_length: usize,
    timekeeper: &TimeKeeper,
    cnt: usize,
) {
    // NOTE: -2はおそらく、マスの大きさを考慮したマージン
    // 破壊する区間 [start, end)
    let start = rng.gen_range(1, state.steps.len() - reset_length - 1);
    let end = start + reset_length;
    let (si, sj) = state.steps[start - 1];
    let (ei, ej) = state.steps[end - 1];

    // 再構築する盤面
    let mut change_state = state.clone();
    change_state.pos = Position::new((si, sj));
    change_state.goal = Position::new((ei, ej));

    // 盤面リセット (探索履歴, ゲームスコア)
    for i in start..end {
        let (pi, pj) = state.steps[i];
        change_state.seen[input.tiles[pi][pj]] = false;
        change_state.game_score -= input.points[pi][pj] as i32;
    }

    // 破壊する開始位置までを保存しておく
    change_state.output = String::from(&change_state.output[..start]);
    change_state.steps = state.steps[..start].to_vec();
    change_state.evaluateScore(USE_GAME_SCORE);

    // 破壊した部分をdfsで再構築
    dfs4sa(
        &input,
        &mut change_state,
        (reset_length * 2) as i32,
        &timekeeper,
    );

    // 破壊してない部分をくっつける
    change_state.output += &state.output[end..];
    change_state.steps.extend_from_slice(&state.steps[end..]);
    change_state.evaluateScore(USE_GAME_SCORE);

    // TODO:
    // --------------------------------------------------
    let delta_score = (change_state.game_score - state.game_score) as f64;
    let temp = START_TEMP
        + (END_TEMP - START_TEMP) * timekeeper.time().to_f64().unwrap() / (TIME_LIMIT * 1000.0);
    let prob = if delta_score >= 0.0 {
        1.0
    } else if delta_score / temp <= -10.0 {
        0.0
    } else {
        (delta_score / temp).exp()
    };
    if prob > rng.gen_range(0.0, 1.0) {
        *state = change_state;

        // eprintln!(
        //     "update {} : {} ms, {} times, score:{}",
        //     start,
        //     timekeeper.time(),
        //     cnt,
        //     state.game_score
        // );

        //
    }
    // --------------------------------------------------

    // // 破壊した部分をdfsで再構築した結果が良かったら更新
    // if state.game_score < change_state.game_score {
    //     *state = change_state;
    //     eprintln!(
    //         "update {} : {} ms, {} times, score:{}",
    //         start,
    //         timekeeper.time(),
    //         cnt,
    //         state.game_score
    //     );
    // }

    //
}

fn bfs(input: &Input, state: &mut State, timekeeper: &TimeKeeper) {
    let mut heap = BinaryHeap::new();
    heap.push(Reverse(state.clone()));

    while let Some(Reverse(now_state)) = heap.pop() {
        // 100ms探索して盤面をつなげられなかったら、局所解と判断してstep2に移行
        if timekeeper.time() > 50 {
            break;
        }
        if now_state.evaluated_score > state.evaluated_score {
            *state = now_state.clone();
        }
        if now_state.evaluated_score <= 2 {
            *state = now_state.clone();
            break;
        }
        now_state
            .legalActions(&input)
            .iter()
            .map(|&action| {
                let mut next_state = now_state.clone();
                next_state.advance(&input, action);
                next_state.evaluateScore(USE_TARGET_DISTANCE);
                next_state
            })
            .for_each(|next_state| heap.push(Reverse(next_state)));
    }
}

// 目的地の座標を返す
fn target_ij((i, j): (usize, usize)) -> (usize, usize) {
    let block = (4 * i / TILE_SIZE) * 4 + (4 * j / TILE_SIZE);
    TARGET[block]
}

// step1: 盤面を16分割して、各ブロックの中心座標を通りながら、ぐるっと回る
fn step1(input: &Input, state: &mut State, timekeeper: &TimeKeeper) {
    state.evaluateScore(USE_TARGET_DISTANCE);
    for _ in 0..16 {
        if timekeeper.isTimeOver() {
            break;
        }
        bfs(input, state, timekeeper);
        let next_goal = target_ij((state.pos.i, state.pos.j));
        state.goal.i = next_goal.0;
        state.goal.j = next_goal.1;
        state.evaluateScore(USE_TARGET_DISTANCE);
    }
}

// step2: ランダムに経路を破壊して、破壊した部分をdfsで再構築して、山登り
fn step2(
    input: &Input,
    rng: &mut ChaCha20Rng,
    state: &mut State,
    timekeeper: &TimeKeeper,
) -> State {
    let mut best_state = state.clone();
    let mut cnt = 0_usize;
    while !timekeeper.isTimeOver() {
        let reset_length = rng.gen_range(10, 15);
        simulatedAnnealing(input, rng, state, reset_length, timekeeper, cnt);
        if best_state.game_score < state.game_score {
            best_state = state.clone();
        }
        cnt += 1;
    }
    eprintln!("climb {} times", cnt);
    best_state

    //
}

fn main() {
    input! {
        s: (usize, usize),
        tiles: [[usize; TILE_SIZE]; TILE_SIZE],
        points: [[i32; TILE_SIZE]; TILE_SIZE],
    }

    let timekeeper = TimeKeeper::new(TIME_LIMIT);
    let mut rng = ChaCha20Rng::seed_from_u64(SEED);
    let input = Input { s, tiles, points };

    // 盤面の初期化
    let mut state = State::new(&input, std::usize::MAX, input.s, target_ij(input.s));
    state.evaluateScore(USE_TARGET_DISTANCE);

    // step1: 盤面を16分割して、各ブロックの中心座標を通りながら、ぐるっと回る
    // eprintln!("step1 start : {} ms", timekeeper.time());
    step1(&input, &mut state, &timekeeper);
    // eprintln!("score: {}", state.game_score);
    // eprintln!("step1 finish: {} ms\n", timekeeper.time());

    // step2: ランダムに経路を破壊して、破壊した部分をdfsで再構築して、山登り
    // eprintln!("step2 start : {} ms", timekeeper.time());
    state.evaluateScore(USE_GAME_SCORE);
    let best_state = step2(&input, &mut rng, &mut state, &timekeeper);
    // eprintln!("score: {}", state.game_score);
    // eprintln!("step2 finish: {} ms\n", timekeeper.time());

    println!("{}", &best_state.output[1..]);

    // // debug
    // state.updateGameScore(&input);
    // state.toString(&input);
    // eprintln!("{} ms", timekeeper.time());
    // eprintln!("score: {}", state.game_score);
    // eprintln!("out : {}", state.output.len());
    // eprintln!("step: {}", state.steps.len());

    //
}
