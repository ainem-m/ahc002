from typing import List, Tuple, Dict, Callable
from time import time
from collections import namedtuple
from heapq import heappop, heappush
import random
import sys


def eprint(*args, **kwargs):
    """標準エラー出力に出力するprint
    """
    print(*args, file=sys.stderr, **kwargs)
    
### 型の定義
Coordinate = namedtuple("Coordinate", "i_ j_")
Action = int
Actions = List[int]
ScoreType = int
Output = str
### 定数
INF = 1000000000
TILE_SIZE = 50
DIJ = [Coordinate(0, -1), Coordinate(0, 1), Coordinate(-1, 0), Coordinate(1, 0)]
DIR = "LRUD"
### 時間制限
TIME_LIMIT = 1.9


class TimeKeeper:
    """時間を管理するクラス
    時間制限を秒単位で指定してインスタンスをつくる
    """
    def __init__(self, time_threshold) -> None:
        self.start_time_ = time()
        self.time_threshold_ = time_threshold
    def isTimeOver(self) -> bool:
        """インスタンスを生成した時から指定した時間制限を超過したか判断する
        """
        return time() - self.start_time_ - self.time_threshold_ >= 0
    def time(self) -> int:
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
    evaluate_score_: 探索上で評価したスコア<br>
    first_action_: 探索木のノートルードで最初に選択した行動<br>
    """
    def __init__(self, END_TURN:int, pos:Coordinate, turn=0, seen=[], steps = [], output="", game_score=0) -> None:
        self.END_TURN_ = END_TURN
        self.turn_ = turn if turn else 0
        self.seen_ = seen if seen else [False]*M
        self.pos_:Coordinate = Coordinate(pos[0], pos[1])
        self.seen_[tiles[pos[0]][pos[1]]] = True
        self.steps_:List[Coordinate] = steps if steps else [pos]
        self.output_:Output = output
        self.game_score_ = game_score if game_score else ps[pos[0]][pos[1]]
        self.evaluate_score_ = self.game_score_
        self.first_action_ = INF
        
    def evaluateScore(self) -> None:
        """
        [どのゲームでも実装する]: 探索用の盤面評価をする
        探索ではゲーム本来のスコアに別の評価値をプラスするといい探索ができるので、ここに工夫の余地がある。
        """
        self.evaluate_score_ = self.game_score_
        
    def isDone(self) -> bool:
        """
        [どのゲームでも実装する]: ゲームの終了判定
        """
        return self.turn_ == self.END_TURN_
    
    def advance(self, action:Action) -> None:
        """
        [どのゲームでも実装する]: 指定したactionでゲームを1ターン進める
        """
        self.pos_ = Coordinate(self.pos_.i_ + DIJ[action].i_, self.pos_.j_ + DIJ[action].j_)
        self.steps_.append((self.pos_.i_, self.pos_.j_))
        self.game_score_ += ps[self.pos_.i_][self.pos_.j_]
        self.seen_[tiles[self.pos_.i_][self.pos_.j_]] = True
        self.turn_ += 1
        self.output_ += DIR[action]
    
    def legalActions(self) -> Actions:
        """
        [どのゲームでも実装する]: 現在の状況でプレイヤーが可能な行動を全て取得する
        """
        actions: Actions = []
        for action in range(4):
            ni = self.pos_.i_ + DIJ[action].i_;
            nj = self.pos_.j_ + DIJ[action].j_;
            if 0<=ni < TILE_SIZE and 0<=nj < TILE_SIZE and  not self.seen_[tiles[ni][nj]]:
                actions.append(action);
        return actions
    
    def __str__(self):
        """
        [実装しなくてもよいが実装すると便利]: 現在のゲーム状況をstrで返す
        """
        res: str = ""
        dic:Dict[str, str] = {
            "LL" : "━━",
            "LU" : "┗━",
            "LD" : "┏━",
            "RR" : "━━",
            "RU" : "┛ ",
            "RD" : "┓ ",
            "UL" : "┓ ",
            "UR" : "┏━",
            "UU" : "┃ ",
            "DL" : "┛ ",
            "DR" : "┗━",
            "DD" : "┃ "}
        path = [["  " for _ in range(TILE_SIZE)] for _ in range(TILE_SIZE)]
        i, j = si, sj
        path[i][j] = "@@"
        for i in range(1, self.turn_):
            h, w = self.steps_[i]
            dir = self.output_[i-1] + self.output_[i]
            path[h][w] = dic[dir]
        # 出力パート
        is_connect_horizontal:Callable[[int, int], bool] = lambda h, w: w+1<TILE_SIZE and tiles[h][w] == tiles[h][w+1]
        is_connect_vertical: Callable[[int, int], bool] = lambda h, w: h+1<TILE_SIZE and tiles[h][w] == tiles[h+1][w]
        for h in range(TILE_SIZE):
            for w in range(TILE_SIZE):
                if not is_connect_vertical(h, w):
                    res += "\x1b[4m"
                if self.seen_[tiles[h][w]]:
                    res += "\x1b[46m"
                res += path[h][w]
                if is_connect_horizontal(h, w):
                    res += " "
                else:
                    res += "\x1b[0m|"
            res += "\n"
        res += f"turn : {self.turn_}\n"
        res += f"score: {self.game_score_}\n"
        return res
    def __lt__(self, other) -> bool:
        """
        [どのゲームでも実装する] : 探索時のソート用に評価を比較する
        pythonではheapqがminheapなのでマイナスをつけている
        """
        # TODOここでotherに型ヒントつけれないのどうしたらいいのか調べる
        return -self.evaluated_score_ < -other.evaluated_score_

State = TileState
def clone(state:State) -> State:
    return State(state.END_TURN_, state.pos_, state.turn_, state.seen_.copy(), state.steps_.copy(), state.output_, state.game_score_)

def randomAction(state:State) -> Action:
    """
    ランダムに行動を決定する
    """
    legal_actions = state.legalActions()
    if not legal_actions:
        return INF
    return random.choice(legal_actions)

def greedyAction(state:State) -> Action:
    """
    貪欲法で行動を決定する
    """
    legal_actions = state.legalActions()
    best_score: ScoreType = -INF
    best_action: Action = INF # pythonだと負のindexが許容されるのでINF
    for action in legal_actions:
        now_state: State = clone(state)
        now_state.advance(action)
        now_state.evaluateScore()
        if now_state.evaluated_score_ > best_score:
            best_score = now_state.evaluated_score_
            best_action = action
    return best_action

def beamSearchAction(state:State, beam_width:int, beam_depth:int) -> Action:
    """
    ビーム1本あたりのビーム幅とビームの本数を指定してchokudaiサーチで行動を決定する
    """
    PriorityQueue = List # pythonにオブジェクトとしてのheapqは存在しないので…
    legal_actions = state.legalActions()
    now_beam:PriorityQueue[State] = []
    best_state:State
    
    heappush(now_beam, state)
    for t in range(beam_depth):
        next_beam:PriorityQueue[State] = []
        for i in range(beam_width):
            if not now_beam: break
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
        best_state = now_beam[0]
        if best_state.isDone(): break
    return best_state.first_action_

def beamSearchActionWithTimeThreshold(state:State, beam_width:int, time_threshold:float) -> Action:
    """
    ビーム幅と制限時間(s)を指定してビームサーチで行動を決定する
    """
    time_keeper = TimeKeeper(time_threshold)
    PriorityQueue = List # pythonにオブジェクトとしてのheapqは存在しないので…
    legal_actions = state.legalActions()
    now_beam:PriorityQueue[State] = []
    best_state:State
    
    heappush(now_beam, state)
    t:int = 0
    while True:
        next_beam:PriorityQueue[State] = []
        for i in range(beam_width):
            if time_keeper.isTimeOver():
                return best_state.first_action_
            if not now_beam: break
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
        if best_state.isDone(): break
    return best_state.first_action_

def chokudaiSearchAction(state:State, beam_width:int, beam_depth:int, beam_number:int) -> Action:
    PriorityQueue = List
    beam:List[PriorityQueue[State]] = [[] for _ in range(beam_depth + 1)]
    heappush(beam[0], state)
    for cnt in range(beam_number):
        for t in range(beam_depth):
            now_beam = beam[t]
            next_beam = beam[t+1]
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
                    if t==0: next_state.first_action_ = action
                    heappush(next_beam, next_state)
    for t in range(beam_depth)[::-1]: # 逆順に
        now_beam = beam[t]
        if now_beam:
            return now_beam[0].first_action_
    
def chokudaiSearchActionWithTimeThreshold(state:State, beam_width:int, beam_depth:int, time_threshold:float) -> Action:
    time_keeper = TimeKeeper(time_threshold)
    PriorityQueue = List
    beam:List[PriorityQueue[State]] = [[] for _ in range(beam_depth + 1)]
    heappush(beam[0], state)
    while True:
        for t in range(beam_depth):
            now_beam = beam[t]
            next_beam = beam[t+1]
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
                    if t==0: next_state.first_action_ = action
                    heappush(next_beam, next_state)
        if time_keeper.isTimeOver():break
    for t in range(beam_depth)[::-1]: # 逆順に
        now_beam = beam[t]
        if now_beam:
            return now_beam[0].first_action_


if __name__ == "__main__":
    si, sj = map(int, input().split())
    tiles:List[List[int]] = [[int(i) for i in range(TILE_SIZE)] for _ in range(TILE_SIZE)]
    ps:List[List[int]] = [[int(i) for i in range(TILE_SIZE)] for _ in range(TILE_SIZE)]
    M:int = 0 # タイルの枚数
    for i in range(TILE_SIZE):
        for j in range(TILE_SIZE):
            M = max(M, tiles[i][j])
    M += 1
    timekeeper = TimeKeeper(TIME_LIMIT)
    state = State(3000, Coordinate(si, sj))
    state.evaluateScore()
    loop_cnt = 0
    while True:
        action = randomAction(state)
        loop_cnt += 1
        if action == INF or timekeeper.isTimeOver():
            break
        state.advance(action)
        state.evaluateScore()
    eprint(state)
    print(state.output_)
    eprint(f"{loop_cnt} loop")
    eprint(f"{timekeeper.time()} ms")