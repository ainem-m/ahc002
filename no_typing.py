from time import time
from collections import namedtuple
from heapq import heappop, heappush
import random
import sys

# [世界四連覇AIエンジニアがゼロから教えるゲーム木探索入門]
# (https://qiita.com/thun-c/items/058743a25c37c87b8aa4)
# を参考にしています。thunderさんに多大なる感謝を…
# Copyright [2021] <Copyright Eita Aoki (Thunder) >

### 型の定義
# あれ、どっちがiでどっちがjだっけ？にならないようにnamedtupleを使う
# 添字でもフィールド名でも呼び出せて便利！
# >>> coor = Coordinate(1, 2)
# >>> print(coor[0]) # 1
# >>> print(coor.i_) # 1
# >>> i, j = coor
# >>> print(i, j) # 1 2
Coordinate = namedtuple("Coordinate", "i_ j_")

### 定数
INF = 1000000000
TILE_SIZE = 50
DIJ = [Coordinate(0, -1), Coordinate(0, 1), Coordinate(-1, 0), Coordinate(1, 0)]
DIR = "LRUD"

### 好みで変更する
TIME_LIMIT = 1.9
SEED = 20210325


def eprint(*args, **kwargs):
    """標準エラー出力に出力するprint"""
    print(*args, file=sys.stderr, **kwargs)


class TimeKeeper:
    """時間を管理するクラス
    時間制限を秒単位で指定してインスタンスをつくる
    """

    def __init__(self, time_threshold):
        self.start_time_ = time()
        self.time_threshold_ = time_threshold

    def isTimeOver(self):
        """インスタンスを生成した時から指定した時間制限を超過したか判断する
        returns: bool
        """
        return time() - self.start_time_ - self.time_threshold_ >= 0

    def time(self):
        """経過時間をミリ秒単位で返す
        returns: int
        """
        return int((time() - self.start_time_) * 1000)


class TileState:
    """
    END_TURN_: 探索を終了するターン<br>
    turn_: 現在のターン<br>
    seen_: タイルを踏んだかどうか<br>
    pos_: 現在位置<br>
    output_: 経路の出力<br>
    steps_: 移動経路の座標<br>
    game_score_: 得点(実際の得点)<br>
    evaluated_score_: 探索上で評価したスコア<br>
    first_action_: 探索木のノートルードで最初に選択した行動<br>
    """

    def __init__(
        self,
        end_turn,
        pos,
        turn=0,
        seen=[],
        steps=[],
        output="",
        game_score=0,
        first_action=INF,
    ):
        self.END_TURN_ = end_turn
        self.turn_ = turn if turn else 0
        self.seen_ = seen if seen else [False] * M
        self.pos_: Coordinate = Coordinate(pos[0], pos[1])
        self.seen_[tiles[pos[0]][pos[1]]] = True
        self.steps_ = steps if steps else [pos]
        self.output_ = output
        self.game_score_ = game_score if game_score else ps[pos[0]][pos[1]]
        self.evaluated_score_ = 0
        self.first_action_ = first_action

    def evaluateScore(self):
        """
        [どのゲームでも実装する]: 探索用の盤面評価をする
        探索ではゲーム本来のスコアに別の評価値をプラスするといい探索ができるので、ここに工夫の余地がある。
        """
        self.evaluated_score_ = self.game_score_

    def isDone(self):
        """
        [どのゲームでも実装する]: ゲームの終了判定
        returns: bool
        """
        return self.turn_ == self.END_TURN_

    def advance(self, action):
        """
        action: int
        [どのゲームでも実装する]: 指定したactionでゲームを1ターン進める
        """
        self.pos_ = Coordinate(
            self.pos_.i_ + DIJ[action].i_, self.pos_.j_ + DIJ[action].j_
        )
        self.steps_.append((self.pos_.i_, self.pos_.j_))
        self.game_score_ += ps[self.pos_.i_][self.pos_.j_]
        self.seen_[tiles[self.pos_.i_][self.pos_.j_]] = True
        self.turn_ += 1
        self.output_ += DIR[action]

    def legalActions(self):
        """
        [どのゲームでも実装する]: 現在の状況でプレイヤーが可能な行動を全て取得する
        returns: List[int]
        """
        actions = []
        for action in range(4):
            ni = self.pos_.i_ + DIJ[action].i_
            nj = self.pos_.j_ + DIJ[action].j_
            if (
                0 <= ni < TILE_SIZE
                and 0 <= nj < TILE_SIZE
                and not self.seen_[tiles[ni][nj]]
            ):
                actions.append(action)
        return actions

    def __str__(self):
        """
        [実装しなくてもよいが実装すると便利]: 現在のゲーム状況をstrで返す
        returns: str
        """
        res: str = ""
        # key: 直前の移動方向+今回の移動方向
        # value: 罫線
        dic = {
            "LL": "━━",
            "LU": "┗━",
            "LD": "┏━",
            "RR": "━━",
            "RU": "┛ ",
            "RD": "┓ ",
            "UL": "┓ ",
            "UR": "┏━",
            "UU": "┃ ",
            "DL": "┛ ",
            "DR": "┗━",
            "DD": "┃ ",
        }
        path = [["  " for _ in range(TILE_SIZE)] for _ in range(TILE_SIZE)]
        i, j = si, sj
        path[i][j] = "@@"
        # 移動経路を読み込み、罫線を引く
        for i in range(1, self.turn_):
            h, w = self.steps_[i]
            dir = self.output_[i - 1] + self.output_[i]
            path[h][w] = dic[dir]
        # 出力パート
        isConnectHorizontal = (
            lambda h, w: w + 1 < TILE_SIZE and tiles[h][w] == tiles[h][w + 1]
        )
        isConnectVertical = (
            lambda h, w: h + 1 < TILE_SIZE and tiles[h][w] == tiles[h + 1][w]
        )
        for h in range(TILE_SIZE):
            for w in range(TILE_SIZE):
                if not isConnectVertical(h, w):
                    # 下のタイルと繋がっていなかったら下線を引く
                    res += "\x1b[4m"
                if self.seen_[tiles[h][w]]:
                    # 踏んだタイルなら色を塗る
                    res += "\x1b[46m"
                res += path[h][w]
                if isConnectHorizontal(h, w):
                    # 右のタイルと繋がっていたら文字修飾を引き継いで空白を出力
                    res += " "
                else:
                    # 右のタイルと繋がっていなかったら修飾をリセットして|を出力
                    res += "\x1b[0m|"
            res += "\n"
        res += f"turn : {self.turn_}\n"
        res += f"score: {self.game_score_}\n"
        res += f"legal_actions: {self.legalActions()}\n"
        return res

    def __lt__(self, other):
        """
        [どのゲームでも実装する] : 探索時のソート用に評価を比較する
        pythonではheapqがminheapなのでevaluated_score_の値に
        マイナスをつけている
        """
        # TODOここでotherに型ヒントつけれないのどうしたらいいのか調べる
        return -self.evaluated_score_ < -other.evaluated_score_


State = TileState


def clone(state):
    """
    pythonではdeepcopyが信用できないので代わりのもの
    Stateを受け取り、複製したStateを返す
    returns: State
    """
    return State(
        state.END_TURN_,
        state.pos_,
        state.turn_,
        state.seen_.copy(),
        state.steps_.copy(),
        state.output_,
        state.game_score_,
        state.first_action_,
    )


def randomAction(state):
    """
    ランダムに行動を決定する
    returns: int
    """
    legal_actions = state.legalActions()
    if not legal_actions:
        return INF
    return random.choice(legal_actions)


def greedyAction(state: State):
    """
    貪欲法で行動を決定する
    returns: int
    """
    legal_actions = state.legalActions()
    best_score = -INF
    best_action = INF  # pythonだと負のindexが許容されるのでINF
    for action in legal_actions:
        now_state: State = clone(state)
        now_state.advance(action)
        now_state.evaluateScore()
        if now_state.evaluated_score_ > best_score:
            best_score = now_state.evaluated_score_
            best_action = action
    return best_action


def beamSearchAction(state, beam_width, beam_depth):
    """
    ビーム幅と深さを指定してビームサーチで行動を決定する
    returns: int
    """
    now_beam = []

    heappush(now_beam, state)
    for t in range(beam_depth):
        next_beam = []
        for i in range(beam_width):
            if not now_beam:
                break
            now_state = heappop(now_beam)
            legal_actions = now_state.legalActions()
            if not legal_actions:
                heappush(next_beam, now_state)
                break
            for action in legal_actions:
                next_state = clone(now_state)
                next_state.advance(action)
                next_state.evaluateScore()
                if t == 0:
                    next_state.first_action_ = action
                heappush(next_beam, next_state)
        now_beam = next_beam
        best_state = now_beam[0]
        if best_state.isDone():
            break
    return best_state.first_action_


def beamSearchActionWithTimeThreshold(state, beam_width, time_threshold):
    """
    ビーム幅と制限時間(s)を指定してビームサーチで行動を決定する
    returns: int
    """
    time_keeper = TimeKeeper(time_threshold)
    legal_actions = state.legalActions()
    now_beam = []
    best_state: State

    heappush(now_beam, state)
    t: int = 0
    while True:
        next_beam = []
        for i in range(beam_width):
            if time_keeper.isTimeOver():
                return best_state.first_action_
            if not now_beam:
                break
            now_state = heappop(now_beam)
            legal_actions = now_state.legalActions()
            for action in legal_actions:
                next_state = clone(now_state)
                next_state.advance(action)
                next_state.evaluateScore()
                if t == 0:
                    next_state.first_action_ = action
                heappush(next_beam, next_state)
        now_beam = next_beam
        if not now_beam:
            break
        best_state = now_beam[0]
        t += 1
        if best_state.isDone():
            break
    return best_state.first_action_


def chokudaiSearchAction(state, beam_width, beam_depth, beam_number):
    """ビーム1本あたりのビーム幅とビームの本数を指定してchokudaiサーチで行動を決定する"""
    beam = [[] for _ in range(beam_depth + 1)]
    heappush(beam[0], state)
    for cnt in range(beam_number):
        for t in range(beam_depth):
            now_beam = beam[t]
            next_beam = beam[t + 1]
            for i in range(beam_width):
                if not now_beam:
                    break
                now_state = now_beam[0]
                if now_state.isDone():
                    break
                heappop(now_beam)
                legal_actions = now_state.legalActions()
                if not legal_actions:
                    heappush(now_beam, now_state)
                    break
                for action in legal_actions:
                    next_state = clone(now_state)
                    next_state.advance(action)
                    next_state.evaluateScore()
                    if t == 0:
                        next_state.first_action_ = action
                    heappush(next_beam, next_state)
    for t in range(beam_depth + 1)[::-1]:  # 逆順に
        now_beam = beam[t]
        if now_beam:
            return now_beam[0].first_action_
    return INF


def chokudaiSearchActionWithTimeThreshold(
    state, beam_width, beam_depth, time_threshold
):
    """
    ビーム1本あたりのビーム幅と制限時間(s):floatを指定してchokudaiサーチで行動を決定する
    returns: int
    """
    time_keeper = TimeKeeper(time_threshold)
    beam = [[] for _ in range(beam_depth + 1)]
    heappush(beam[0], state)
    while True:
        for t in range(beam_depth):
            now_beam = beam[t]
            next_beam = beam[t + 1]
            for i in range(beam_width):
                if not now_beam:
                    break
                now_state = now_beam[0]
                if now_state.isDone():
                    break
                heappop(now_beam)
                legal_actions = now_state.legalActions()
                for action in legal_actions:
                    next_state = clone(now_state)
                    next_state.advance(action)
                    next_state.evaluateScore()
                    if t == 0:
                        next_state.first_action_ = action
                    heappush(next_beam, next_state)
        if time_keeper.isTimeOver():
            break
    for t in range(beam_depth + 1)[::-1]:  # 逆順に
        now_beam = beam[t]
        if now_beam:
            return now_beam[0].first_action_
    return INF


if __name__ == "__main__":
    random.seed(20210325)
    si, sj = map(int, input().split())
    tiles = [[int(i) for i in input().split()] for _ in range(TILE_SIZE)]
    ps = [[int(i) for i in input().split()] for _ in range(TILE_SIZE)]
    M = 0  # タイルの枚数
    for i in range(TILE_SIZE):
        for j in range(TILE_SIZE):
            M = max(M, tiles[i][j])
    M += 1
    timekeeper = TimeKeeper(TIME_LIMIT)
    state = State(INF, Coordinate(si, sj))
    state.evaluateScore()
    loop_cnt = 0
    while True:
        # 好きな実装を選択しよう！
        # ハイパーパラメータ(ビーム幅など)は適当です。
        action = randomAction(state)
        # action = greedyAction(state)
        # action = beamSearchAction(state, 3, 3)
        # action = beamSearchActionWithTimeThreshold(state, 10, 0.01)
        # action = chokudaiSearchAction(state, 10, 10, 50)
        # action = chokudaiSearchActionWithTimeThreshold(state, 10, 10, 0.02)
        loop_cnt += 1
        if action == INF or timekeeper.isTimeOver():
            # 手詰まりになったか時間切れの場合
            # 手詰まりになった後の行動は工夫の余地あり
            break
        state.advance(action)
        state.evaluateScore()
    eprint(state)
    print(state.output_)
    eprint(f"{loop_cnt} loop")
    eprint(f"{timekeeper.time()} ms")
