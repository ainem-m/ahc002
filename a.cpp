// (https://qiita.com/thun-c/items/058743a25c37c87b8aa4)
// を参考にしています。thunderさんに多大なる感謝を…

// Copyright [2021] <Copyright Eita Aoki (Thunder) >
#include <string>
#include <vector>
#include <sstream>
#include <iostream>
#include <utility>
#include <random>
#include <assert.h>
#include <math.h>
#include <chrono>
#include <queue>
#include <algorithm>
#include <string>
std::random_device rnd;
std::mt19937 mt(rnd());

// 型の定義
using Action = int;
using Actions = std::vector<int>;
using ScoreType = int64_t;
using Output = std::string;

// 定数
constexpr const ScoreType INF = 1000000000LL;
const int TILE_SIZE = 50;
const int DIJ[4][2] = {{0, -1}, {0, 1}, {-1, 0}, {1, 0}};
const std::string DIR = "LRUD";

// 好みで変更する
const int64_t TIME_LIMIT = 1900;
const int SEED = 20210325;

// 時間を管理するクラス
class TimeKeeper
{
private:
    std::chrono::high_resolution_clock::time_point start_time_;
    int64_t time_threshold_;

public:
    // 時間制限をミリ秒単位で指定してインスタンスをつくる。
    TimeKeeper(const int64_t &time_threshold)
        : start_time_(std::chrono::high_resolution_clock::now()),
          time_threshold_(time_threshold)
    {
    }

    // インスタンス生成した時から指定した時間制限を超過したか判断する。
    bool isTimeOver() const
    {
        auto diff = std::chrono::high_resolution_clock::now() - this->start_time_;
        return std::chrono::duration_cast<std::chrono::milliseconds>(diff).count() >= time_threshold_;
    }
};

/// END_TURN_: 探索を終了するターン<br>
/// turn_: 現在のターン<br>
/// seen_: タイルを踏んだかどうか<br>
/// pos_: 現在位置<br>
/// output_: 経路の出力<br>
/// steps_: 移動経路の座標<br>
/// game_score_: 得点(実際の得点)<br>
/// evaluated_score_: 探索上で評価したスコア<br>
/// first_action_: 探索木のノートルードで最初に選択した行動<br>
class TileState
{
private:
    // 現在位置を保持する。
    struct Position
    {
        int i_;
        int j_;
        Position(const int i = 0, const int j = 0) : i_(i), j_(j) {}
    };

    int END_TURN_;
    int turn_;
    std::vector<std::vector<bool>> seen_;
    Position pos_;

public:
    Output output_;
    std::vector<Position> steps_;
    int game_score_;
    ScoreType evaluated_score_;
    Action first_action_;
    TileState() {}

    // TODO
    // コンストラクタ全然わからないのでrustのnewを載せてます
    TileState(const int end_turn, const int si, const int sj) : END_TURN_(end_turn),
                                                                turn_(0),
                                                                seen_(false, std::vector<bool>(M)),
                                                                pos_(Position{si, sj}),
                                                                output_(""),
                                                                steps_(steps_),
                                                                game_score_(0),
                                                                evaluated_score_(0),
                                                                first_action_(-1){
                                                                    /*
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
                                                                        let evaluated_score_ = game_score_;
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
                                                                  } */

                                                                    // [どのゲームでも実装する] : 探索用の盤面評価をする
                                                                    void
                                                                        evaluateScore(){
                                                                            this->evaluated_score_ = this->game_score_; // 探索ではゲーム本来のスコアに別の評価値をプラスするといい探索ができるので、ここに工夫の余地がある。
}

// [どのゲームでも実装する] : ゲームの終了判定
bool
isDone() const
{
    return this->turn_ == END_TURN_;
}

// [どのゲームでも実装する] : 指定したactionでゲームを1ターン進める
// TODO
void advance(const Action &action)
{
    this->pos_.i_ += DIJ[action][0];
    this->pos_.j_ += DIJ[action][1];
    /*
    self.steps_.push((self.pos_.i_, self.pos_.j_));
    self.game_score_ += input.ps[self.pos_.i_][self.pos_.j_];
    self.seen_[input.tiles[self.pos_.i_][self.pos_.j_]] = true;
    self.turn_ += 1;
    self.output_.push(DIR[action]);*/
    auto &point = this->points_[this->pos_.i_][this->pos_.j_];
    if (point > 0)
    {
        this->game_score_ += point;
        point = 0;
    }
    this->turn_++;
}

// [どのゲームでも実装する] : 現在の状況でプレイヤーが可能な行動を全て取得する
// TODO tilesはどこから受け取りますか？
// pythonだとスコープがガバガバなのでif __name__ == "__main__":内の変数を受け取れる
// rustだと無理なのでInput構造体から受け取っています
Actions legalActions() const
{
    Actions actions;
    for (Action action = 0; action < 4; action++)
    {
        int ty = this->pos_.i_ + DIJ[action][0];
        int tx = this->pos_.j_ + DIJ[action][1];
        if (ty >= 0 && ty < h_ && tx >= 0 && tx < w_ && !this->seen_[input.tiles[ty][tx]])
        {
            actions.emplace_back(action);
        }
    }
    return actions;
}

// [実装しなくてもよいが実装すると便利] : 現在のゲーム状況を文字列にする
// TODO これ実装の仕方全く検討もつかなくて途方にくれています…
std::string toString() const
{
    std::stringstream ss;
    ss << "turn:\t" << this->turn_ << "\n";
    ss << "score:\t" << this->game_score_ << "\n";
    /*
        let mut path = vec ![vec !["  "; TILE_SIZE]; TILE_SIZE];
        // 移動経路に罫線を引く
        let(i, j) = input.s;
        path[i][j] = "@@";
        for
            i in 1..self.turn_
            {
                let(h, w) = self.steps_[i];
                let mut dir = String::new ();
                dir.push(self.output_.chars().nth(i - 1).unwrap());
                dir.push(self.output_.chars().nth(i).unwrap());
                // 直前の移動方向 + 今回の移動方向によって引く罫線を決定
                path[h][w] = match dir.as_str()
                {
                    "LL" = > "━━",
                    "LU" = > "┗━",
                    "LD" = > "┏━",
                    "RR" = > "━━",
                    "RU" = > "┛ ",
                    "RD" = > "┓ ",
                    "UL" = > "┓ ",
                    "UR" = > "┏━",
                    "UU" = > "┃ ",
                    "DL" = > "┛ ",
                    "DR" = > "┗━",
                    "DD" = > "┃ ",
                    _ = > unreachable !(),
                }
            }
        // 出力パート
        let isConnectHorizontal =
            | h : usize,
            w : usize | w + 1 < TILE_SIZE && input.tiles[h][w] == input.tiles[h][w + 1];
        let isConnectVertical =
            | h : usize,
            w : usize | h + 1 < TILE_SIZE && input.tiles[h][w] == input.tiles[h + 1][w];
        for
            h in 0..TILE_SIZE
            {
            for
                w in 0..TILE_SIZE
                {
                    if
                        !isConnectVertical(h, w)
                        {
                            // 下のタイルとつながっていなかったら下線を引く
                            eprint !("\x1b[4m");
                        }
                    if self
                        .seen_[input.tiles[h][w]]
                        {
                            // 踏んだタイルなら色を塗る
                            eprint !("\x1b[46m");
                        }
                    eprint !("{}", path[h][w]);
                    if isConnectHorizontal (h, w)
                    {
                        // 右のタイルと繋がっていたら文字修飾を引き継いで空白を出力
                        eprint !(" ")
                    }
                    else
                    {
                        // 右のタイルと繋がっていなかったら修飾をリセットして|を出力
                        eprint !("\x1b[0m");
                        eprint !("|");
                    }
                }
            eprintln !();
            }
    }
    */
}
}
;

// [どのゲームでも実装する] : 探索時のソート用に評価を比較する
bool operator<(const TileState &maze_1, const TileState &maze_2)
{
    return maze_1.evaluated_score_ < maze_2.evaluated_score_;
}
using State = TileState;

// ランダムに行動を決定する
Action randomAction(const State &state)
{
    auto legal_actions = state.legalActions();
    return legal_actions[mt() % (legal_actions.size())];
}

// 貪欲法で行動を決定する
Action greedyAction(const State &state)
{
    auto legal_actions = state.legalActions();
    ScoreType best_score = -INF;
    Action best_action = -1;
    for (const auto action : legal_actions)
    {
        State now_state = state;
        now_state.advance(action);
        now_state.evaluateScore();
        if (now_state.evaluated_score_ > best_score)
        {
            best_score = now_state.evaluated_score_;
            best_action = action;
        }
    }
    return best_action;
}

// ビーム幅と深さを指定してビームサーチで行動を決定する
Action beamSearchAction(const State &state, const int beam_width, const int beam_depth)
{
    auto legal_actions = state.legalActions();
    std::priority_queue<State> now_beam;
    State best_state;

    now_beam.push(state);
    for (int t = 0; t < beam_depth; t++)
    {
        std::priority_queue<State> next_beam;
        for (int i = 0; i < beam_width; i++)
        {
            if (now_beam.empty())
                break;
            State now_state = now_beam.top();
            now_beam.pop();
            auto legal_actions = now_state.legalActions();
            for (const auto &action : legal_actions)
            {
                State next_state = now_state;
                next_state.advance(action);
                next_state.evaluateScore();
                if (t == 0)
                    next_state.first_action_ = action;
                next_beam.push(next_state);
            }
        }

        now_beam = next_beam;
        best_state = now_beam.top();

        if (best_state.isDone())
        {
            break;
        }
    }
    return best_state.first_action_;
}

// ビーム幅と制限時間(ms)を指定してビームサーチで行動を決定する
Action beamSearchActionWithTimeThreshold(const State &state, const int beam_width, const int64_t time_threshold)
{
    auto time_keeper = TimeKeeper(time_threshold);
    auto legal_actions = state.legalActions();
    std::prioriti_queue<State> now_beam;
    State best_state;

    now_beam.push(state);
    for (int t = 0;; t++)
    {
        std::prioriti_queue<State> next_beam;
        for (int i = 0; i < beam_width; i++)
        {
            if (time_keeper.isTimeOver())
            {
                return best_state.first_action_;
            }
            if (now_beam.empty())
                break;
            State now_state = now_beam.top();
            now_beam.pop();
            auto legal_actions = now_state.legalActions();
            for (const auto &action : legal_actions)
            {
                State next_state = now_state;
                next_state.advance(action);
                next_state.evaluateScore();
                if (t == 0)
                    next_state.first_action_ = action;
                next_beam.push(next_state);
            }
        }

        now_beam = next_beam;
        best_state = now_beam.top();

        if (best_state.isDone())
        {
            break;
        }
    }
    return best_state.first_action_;
}

// ビーム1本あたりのビーム幅とビームの本数を指定してchokudaiサーチで行動を決定する
Action chokudaiSearchAction(const State &state, const int beam_width, const int beam_depth, const int beam_number)
{
    auto beam = std::vector<std::prioriti_queue<State>>(beam_depth + 1);
    for (int t = 0; t < beam_depth + 1; t++)
    {
        beam[t] = std::prioriti_queue<State>();
    }
    beam[0].push(state);
    for (int cnt = 0; cnt < beam_number; cnt++)
    {
        for (int t = 0; t < beam_depth; t++)
        {
            auto &now_beam = beam[t];
            auto &next_beam = beam[t + 1];
            for (int i = 0; i < beam_width; i++)
            {
                if (now_beam.empty())
                    break;
                State now_state = now_beam.top();
                if (now_state.isDone())
                {
                    break;
                }
                now_beam.pop();
                auto legal_actions = now_state.legalActions();
                for (const auto &action : legal_actions)
                {
                    State next_state = now_state;
                    next_state.advance(action);
                    next_state.evaluateScore();
                    if (t == 0)
                        next_state.first_action_ = action;
                    next_beam.push(next_state);
                }
            }
        }
    }
    for (int t = beam_depth; t >= 0; t--)
    {
        const auto &now_beam = beam[t];
        if (!now_beam.empty())
        {
            return now_beam.top().first_action_;
        }
    }

    return -1;
}

// ビーム1本あたりのビーム幅と制限時間(ms)を指定してchokudaiサーチで行動を決定する
Action chokudaiSearchActionWithTimeThreshold(const State &state, const int beam_width, const int beam_depth, const int64_t time_threshold)
{
    auto time_keeper = TimeKeeper(time_threshold);
    auto beam = std::vector<std::prioriti_queue<State>>(beam_depth + 1);
    for (int t = 0; t < beam_depth + 1; t++)
    {
        beam[t] = std::prioriti_queue<State>();
    }
    beam[0].push(state);
    for (;;)
    {
        for (int t = 0; t < beam_depth; t++)
        {
            auto &now_beam = beam[t];
            auto &next_beam = beam[t + 1];
            for (int i = 0; i < beam_width; i++)
            {
                if (now_beam.empty())
                    break;
                State now_state = now_beam.top();
                if (now_state.isDone())
                {
                    break;
                }
                now_beam.pop();
                auto legal_actions = now_state.legalActions();
                for (const auto &action : legal_actions)
                {
                    State next_state = now_state;
                    next_state.advance(action);
                    next_state.evaluateScore();
                    if (t == 0)
                        next_state.first_action_ = action;
                    next_beam.push(next_state);
                }
            }
        }
        if (time_keeper.isTimeOver())
        {
            break;
        }
    }
    for (int t = beam_depth; t >= 0; t--)
    {
        const auto &now_beam = beam[t];
        if (!now_beam.empty())
        {
            return now_beam.top().first_action_;
        }
    }

    return -1;
}
#include <iostream>
#include <functional>

int main()
{
    using std::cout;
    using std::endl;
    auto mt = std::mt19937(SEED);
    /*
    input !
    {
    s:
        (usize, usize),
            tiles : [[usize; TILE_SIZE]; TILE_SIZE],
                    ps : [[i32; TILE_SIZE]; TILE_SIZE],
    }
    let timekeeper = TimeKeeper::new (TIME_LIMIT);
    let mut rng = ChaCha20Rng::seed_from_u64(SEED);

    let input = Input{s, tiles, ps};
    let mut state = State::new (&input, !0, input.s);
    state.evaluateScore();
    let mut loop_cnt = 0;
    // 好きな実装を選択しよう！
    // ハイパーパラメータ(ビーム幅など)は適当です。
    // while let Some(action) = greedyAction(&input, &state) {
    // while let Some(action) = beamSearchAction(&input, &state, 3, 3) {
    // while let Some(action) = beamSearchActionWithTimeThreshold(&input, &state, 3, 0.02) {
    // while let Some(action) = chokudaiSearchActionWithTimeThreshold(&input, &state, 3, 3, 0.02) {
    // while let Some(action) = chokudaiSearchAction(&input, &state, 10, 10, 50) {
    while
        let Some(action) = randomAction(&mut rng, &input, &state)
        {
            loop_cnt += 1;
            if timekeeper
                .isTimeOver()
                {
                    break;
                }
            state.advance(&input, action);
            state.evaluateScore();
        }
    state.toString(&input);
    println !("{}", state.output_);
    eprintln !("{} loop", loop_cnt);
    eprintln !("{} ms", timekeeper.time());
    */
}