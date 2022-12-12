# 時間制限 FirstでPermutation(UDRL)の24パターンでDFS。最良のものをSecondで経路一部破壊で焼きなまし。
first_time_limit = 0.16
second_time_limit = 1.915
min_break_length = 2
max_break_length = 20
path_search_limit_count = 2500
initial_temp = 500
final_temp = 10


prod_env = True


import time

# 時間計測開始
start = time.time()

from math import exp
import random
from itertools import permutations
from collections import deque


def RI(a, b):
    return random.randint(a, b)


# 答えと最大値
final_ans = None
max_score = 0

# 入力
si, sj = map(int, input().split())
t = [list(map(int, input().split())) for _ in range(50)]
p = [list(map(int, input().split())) for _ in range(50)]

num_to_place = [[] for _ in range(2500)]
for i in range(50):
    for j in range(50):
        num_to_place[t[i][j]].append((i, j))

# 移動方向
ti = [1, 0, -1, 0]
tj = [0, 1, 0, -1]
t_str = ["D", "R", "U", "L"]
FourNB = {c: (di, dj) for c, di, dj in zip(t_str, ti, tj)}

first_time_limit -= time.time() - start

indexs = range(4)
lap_time = (1 << 7) - 1
for indexs in permutations(range(4)):
    start_of_loop = time.time()
    cnt = 0

    # キュー（中身は（スコア, 答え, 使えるマス, 位置）　(初期値は最初のマス)）
    used = [0 for _ in range(50)]
    for i, j in num_to_place[t[si][sj]]:
        used[i] |= 1 << j

    que = deque([(p[si][sj], "", used, (si, sj))])

    while que:
        if cnt & lap_time == lap_time:
            elapsed_time = time.time() - start_of_loop
            if elapsed_time > first_time_limit / 24:
                break
        score, ans, used, (i, j) = que.pop()
        if max_score <= score:
            final_ans = ans
            max_score = score

        for idx in indexs:
            ni = i + ti[idx]
            nj = j + tj[idx]
            if 0 <= ni < 50 and 0 <= nj < 50 and (not used[ni] >> nj & 1):
                used_ = used[:]
                for i_, j_ in num_to_place[t[ni][nj]]:
                    used_[i_] |= 1 << j_
                que += (((score + p[ni][nj]), ans + t_str[idx], used_, (ni, nj)),)
        cnt += 1

used = [0 for _ in range(50)]
path = [(si, sj)]
for i, j in num_to_place[t[si][sj]]:
    used[i] |= 1 << j

cur_ans = final_ans
cur_score = max_score
for c in cur_ans:
    di, dj = FourNB[c]
    ni, nj = path[-1][0] + di, path[-1][1] + dj
    path += ((ni, nj),)
    for i, j in num_to_place[t[ni][nj]]:
        used[i] |= 1 << j

aneal_cnt = 0
while True:
    aneal_cnt += 1
    if time.time() - start > second_time_limit:
        break

    br_st = random.randint(1, len(cur_ans) - 1)
    br_ed = min(
        br_st + random.randint(min_break_length, max_break_length), len(cur_ans) - 1
    )
    # startが終点or一つ手前の場合バグるかも？？
    tmp_used = used[:]
    # goal=dict()
    goal_point = path[br_ed]
    subscore = 0
    for idx in range(br_st + 1, br_ed + 1):
        ni, nj = path[idx]
        subscore += p[ni][nj]
        for i, j in num_to_place[t[ni][nj]]:
            tmp_used[i] ^= 1 << j
            # goal[(ni,nj)]=(subscore, idx)

    br_si, br_sj = path[br_st]
    que = deque([(0, "", tmp_used, (br_si, br_sj))])
    cand = []
    time_keeper = 0
    while que:
        time_keeper += 1
        if time_keeper > path_search_limit_count:
            break
        score, ans, t_used, (i, j) = que.pop()

        if (i, j) == goal_point:
            # pathと交差してないかチェックが必要
            # 終点を指定しない探索をしたいが、終点以降の経路と同じタイルを踏んでしまうケースを判定できないので断念
            if ans != cur_ans[br_st : br_st + len(ans)] and goal_point == (i, j):
                cand += ((score, ans, t_used, (i, j)),)
        for idx in indexs:
            ni = i + ti[idx]
            nj = j + tj[idx]
            if 0 <= ni < 50 and 0 <= nj < 50 and (not t_used[ni] >> nj & 1):
                if ans and ans[0] == cur_ans[br_st]:
                    continue
                used_ = t_used[:]
                for i_, j_ in num_to_place[t[ni][nj]]:
                    used_[i_] |= 1 << j_
                que += (((score + p[ni][nj]), ans + t_str[idx], used_, (ni, nj)),)
    # print(path[br_st],cur_ans[br_st:br_ed],[(sc,ans,cur) for sc,ans,_,cur,_ in cand])
    temp = (
        initial_temp
        + (final_temp - initial_temp) * (time.time() - start) / second_time_limit
    )

    if RI(0, 1000) < 1:
        random.shuffle(cand)
    else:
        cand.sort(key=lambda x: -x[0])
    for alt_score, alt_path, t_used, (i, j) in cand:
        delta_score = alt_score - subscore
        trnsp = exp((delta_score) / temp)

        prob = 0
        if delta_score >= 0:
            prob = 1
        elif delta_score / temp <= -10:
            prob = 0
        else:
            prob = trnsp

        if prob > random.random():
            # 経路変更
            # print('----path change---',cur_ans[br_st:br_ed],[alt_score, alt_path, [t_used[:1]], (i, j), max_order_in_path])
            new_ans = cur_ans[:br_st] + alt_path + cur_ans[br_ed:]
            new_path = path[: br_st + 1]
            alt_si, alt_sj = path[-1]

            # 破壊した経路のusedを解除
            for i in range(br_st, br_ed - 1):
                di, dj = FourNB[cur_ans[i]]
                ni, nj = path[i][0] + di, path[i][1] + dj
                for i, j in num_to_place[t[ni][nj]]:
                    used[i] ^= 1 << j

            # 新たな経路を進む
            for c in alt_path:
                di, dj = FourNB[c]
                ni, nj = new_path[-1][0] + di, new_path[-1][1] + dj
                new_path += ((ni, nj),)
                for i, j in num_to_place[t[ni][nj]]:
                    used[i] |= 1 << j

            cur_ans = new_ans
            path = new_path + path[br_ed + 1 :]
            cur_score = cur_score + delta_score

            if max_score < cur_score:
                final_ans = cur_ans
                max_score = cur_score
                # print(path[-1])
            break
# print(max_score,aneal_cnt)
print(final_ans)
