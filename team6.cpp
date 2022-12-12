// #pragma GCC target("avx2")
#pragma GCC optimize("O3")
#pragma GCC optimize("unroll-loops")
#include <bits/stdc++.h>

#define rep(i, s, n) for (int i = s; i < n; ++i)
#define rrep(i, s, n) for (int i = s; i >= n; --i)
#define fi first
#define se second
#define all(a) a.begin(), a.end()
#define rall(a) a.rbegin(), a.rend()
#define SUM(a) accumulate(all(a), 0LL)
#define MAX(a) *max_element(all(a))
#define MIN(a) *min_element(all(a))
#define fi first
#define se second
#define len(x) (int)(x).size()
#define dup(x, y) (((x) + (y)-1) / (y))
#define pb push_back
#define eb emplace_back
#define em emplace
#define pq(T) priority_queue<T, vector<T>, greater<T>>
using namespace std;
using namespace chrono;
template <class T>
inline vector<vector<T>> vector2(const size_t &i, const size_t &j, const T &init = T())
{
    return vector<vector<T>>(i, vector<T>(j, init));
}
template <class T>
inline vector<vector<vector<T>>> vector3(const size_t &i, const size_t &j, const int &k, const T &init = T())
{
    return vector<vector<vector<T>>>(i, vector<vector<T>>(j, vector<T>(k, init)));
}
template <class T>
inline vector<vector<vector<vector<T>>>> vector4(const size_t &i, const size_t &j, const size_t &k, const size_t &l, const T &init = T())
{
    return vector<vector<vector<vector<T>>>>(i, vector<vector<vector<T>>>(j, vector<vector<T>>(k, vector<T>(l, init))));
}
template <class T, class U>
bool chmax(T &a, const U &b) { return ((a < b) ? (a = b, true) : false); }
template <class T, class U>
bool chmin(T &a, const U &b) { return ((a > b) ? (a = b, true) : false); }
using ll = long long;
using ld = long double;
using P = pair<int, int>;
using PP = pair<P, P>;
using Pint = pair<P, int>;
using intP = pair<int, P>;
// ================= ここまでテンプレ =================

// 乱数
static uint32_t randXor()
{
    static uint32_t x = 123456789;
    static uint32_t y = 362436069;
    static uint32_t z = 521288629;
    static uint32_t w = 88675123;
    uint32_t t;

    t = x ^ (x << 11);
    x = y;
    y = z;
    z = w;
    return w = (w ^ (w >> 19)) ^ (t ^ (t >> 8));
}

// 0以上1未満の小数をとる乱数
static double rand01()
{
    return (randXor() + 0.5) * (1.0 / UINT_MAX);
}

int dx[] = {1, 0, -1, 0, 0}, dy[] = {0, 1, 0, -1, 0};

char command[] = {'D', 'R', 'U', 'L'};

vector<vector<int>> perms;

// タイルの一片の大きさ
int N;

// 入力を保存しておく構造体
struct INPUT
{

    // start
    int si, sj;

    // t_i,j ,   p_i,j
    vector<vector<int>> t, p;

    // 入力の受け取り
    INPUT() {}
    explicit INPUT(int _N)
    {
        t.assign(N, vector<int>(N));
        p.assign(N, vector<int>(N));
        cin >> si >> sj;
        rep(i, 0, _N) rep(j, 0, _N) cin >> t[i][j];
        rep(i, 0, _N) rep(j, 0, _N) cin >> p[i][j];
    }
};

// 焼きなましで持つ解空間(状態)
struct State
{
    vector<vector<bool>> seen, best_seen, pre_seen;
    vector<vector<char>> root, best_root, pre_root;
    int score, best_score, pre_score;

    State() {}
    explicit State(vector<vector<bool>> _seen, vector<vector<char>> _root, int _score)
    {
        seen = _seen;
        best_seen = _seen;
        root = _root;
        best_root = _root;
        score = _score;
        best_score = _score;
    }

    // ベストスコアを更新
    void update_best()
    {
        if (chmax(best_score, score))
        {
            best_seen = seen;
            best_root = root;
            // cerr << best_score << endl;
        }
    }

    // 今の状態を上書き保存
    void rec_state()
    {
        pre_seen = seen;
        pre_root = root;
        pre_score = score;
    }

    // 保存された状態に戻す
    void roll_back()
    {
        seen = pre_seen;
        root = pre_root;
        score = pre_score;
    }
};

// 焼きなましをする構造体
struct SA
{
    INPUT inp;
    vector<vector<int>> same_tile;
    State S;
    double END_TIME, START_TEMP, END_TEMP;
    SA(INPUT &_inp, vector<vector<int>> &_same_tile, State _S, double _END_TIME, double _START_TEMP, double _END_TEMP)
    {
        inp = _inp;
        same_tile = _same_tile;
        S = _S;
        END_TIME = _END_TIME;
        START_TEMP = _START_TEMP;
        END_TEMP = _END_TEMP;
    }

    // 探索
    void search(auto startClock)
    {
        double time = (duration_cast<microseconds>(system_clock::now() - startClock).count() * 1e-6);
        int iters = 0;
        do
        {
            const double progressRatio = time / END_TIME; // 進捗。開始時が0.0、終了時が1.0
            const double temp = START_TEMP + (END_TEMP - START_TEMP) * progressRatio;

            // 遷移をする前に今の状態を保存
            S.rec_state();

            int deltaScore = translate(40);
            const double probability = exp(deltaScore / temp);

            // cerr << deltaScore << endl;

            if (probability > rand01())
            {
                S.score += deltaScore;
                S.update_best();
            }
            else
            {
                S.roll_back();
            }

            time = (duration_cast<microseconds>(system_clock::now() - startClock).count() * 1e-6);
            iters += 1;
        } while (time < END_TIME);
        cerr << "annealing: " << iters << " iterations" << endl;
    }

    // 長さlenだけ経路を切って新しい経路を作る
    int translate(int len)
    {

        pair<PP, int> start_goal = cut_root(len);

        return unite_root(start_goal.fi.fi.fi, start_goal.fi.fi.se, start_goal.fi.se.fi, start_goal.fi.se.se, start_goal.se);
    }

    // 長さlenの経路を切る
    pair<PP, int> cut_root(int len)
    {

        int x = randXor() % N, y = randXor() % N;

        int s_x = x;
        int s_y = y;
        while (S.root[x][y] == 'X')
        {
            s_x = x = randXor() % N;
            s_y = y = randXor() % N;
        }

        int g_x, g_y, e = 0;
        while (S.root[x][y] != 'X' && len > 0)
        {

            if (S.root[x][y] == 'D')
            {
                S.root[x][y] = 'X';
                ++x;
            }
            else if (S.root[x][y] == 'R')
            {
                S.root[x][y] = 'X';
                ++y;
            }
            else if (S.root[x][y] == 'U')
            {
                S.root[x][y] = 'X';
                --x;
            }
            else
            {
                S.root[x][y] = 'X';
                --y;
            }

            e += inp.p[x][y];
            S.seen[x][y] = false;
            S.seen[x + dx[same_tile[x][y]]][y + dy[same_tile[x][y]]] = false;

            g_x = x;
            g_y = y;
            --len;
        }

        return pair<PP, int>(PP(P(s_x, s_y), P(g_x, g_y)), e);
    }

    // 切った経路をつなげる
    int unite_root(int s_x, int s_y, int g_x, int g_y, int e)
    {
        auto startClock2 = system_clock::now();
        const static double step_TIME = 0.0001; // 終了時間（秒）
        auto rec_seen = S.pre_seen;
        auto rec_root = S.pre_root;
        int ma = -1;

        auto dfs = [&](auto &&dfs, int &cnt, int xx, int yy) -> bool
        {
            double time = (duration_cast<microseconds>(system_clock::now() - startClock2).count() * 1e-6);
            if (time > step_TIME)
            {
                return false;
            }

            if (xx == g_x && yy == g_y)
            {
                if (chmax(ma, cnt))
                {
                    rec_seen = S.seen;
                    rec_root = S.root;
                }
                return true;
            }

            //   vector<int> p(4);
            //   rep(i,0,4) p[i] = i;
            int idx = randXor() % 24;
            //   do{
            //     if(idx == 0) break;
            //     --idx;
            //   }while(next_permutation(all(p)));

            for (int i : perms[idx])
            {
                int nx = xx + dx[i], ny = yy + dy[i];
                if (nx < 0 || nx >= N || ny < 0 || ny >= N)
                    continue;

                if (S.seen[nx][ny])
                    continue;

                cnt += inp.p[nx][ny];
                S.seen[nx][ny] = true;
                S.seen[nx + dx[same_tile[nx][ny]]][ny + dy[same_tile[nx][ny]]] = true;

                S.root[xx][yy] = command[i];
                if (!dfs(dfs, cnt, nx, ny))
                {
                    return false;
                }

                S.root[xx][yy] = 'X';
                cnt -= inp.p[nx][ny];
                S.seen[nx][ny] = false;
                S.seen[nx + dx[same_tile[nx][ny]]][ny + dy[same_tile[nx][ny]]] = false;
            }

            return true;
        };

        int cnt = 0;
        dfs(dfs, cnt, s_x, s_y);
        S.seen = rec_seen;
        S.root = rec_root;
        if (ma == -1)
            ma = e;

        return ma - e;
    }
};

// 入力を渡して解を出力する
void solve(INPUT &inp)
{

    auto same_tile = vector2<int>(N, N, -1);
    rep(i, 0, N) rep(j, 0, N)
    {
        rep(k, 0, 4)
        {
            if (i + dx[k] < 0 || i + dx[k] >= N || j + dy[k] < 0 || j + dy[k] >= N)
                continue;
            if (inp.t[i + dx[k]][j + dy[k]] == inp.t[i][j])
            {
                same_tile[i][j] = k;
                break;
            }
        }
        if (same_tile[i][j] == -1)
            same_tile[i][j] = 4;
    }

    int score = 0;
    string ans = "";

    auto startClock = system_clock::now();
    double time = 0.0;                     // 経過時間（秒）
    const static double syoki_TIME = 0.02; // 終了時間（秒）
    const static double END_TIME = 1.98;   // 終了時間（秒）

    auto seen = vector2<bool>(N, N, false);
    int roop = 0;
    auto root = vector2<char>(N, N, 'X');

    auto dfs = [&](auto &&dfs, int &cnt, string &rec, int x, int y) -> bool
    {
        time = (duration_cast<microseconds>(system_clock::now() - startClock).count() * 1e-6);
        if (time > syoki_TIME)
        {
            if (chmax(score, cnt))
                ans = rec;
            return false;
        }

        cnt += inp.p[x][y];
        seen[x][y] = true;
        seen[x + dx[same_tile[x][y]]][y + dy[same_tile[x][y]]] = true;

        vector<P> nxt;
        rep(i, 0, 4)
        {
            int nx = x + dx[i], ny = y + dy[i];
            if (nx < 0 || nx >= N || ny < 0 || ny >= N)
                continue;
            if (seen[nx][ny])
                continue;
            nxt.eb(pow(abs(nx - N / 2), 2) + pow(abs(ny - N / 2), 2), i);
        }

        bool flag = true;
        sort(rall(nxt));
        for (P i : nxt)
        {
            rec += command[i.se];
            if (!dfs(dfs, cnt, rec, x + dx[i.se], y + dy[i.se]))
            {
                return false;
            }
            rec.pop_back();
            flag = false;
        }

        if (flag)
        {
            if (chmax(score, cnt))
            {
                ans = rec;
                // cerr << score << endl;
            }
            ++roop;
        }

        cnt -= inp.p[x][y];
        seen[x][y] = false;
        seen[x + dx[same_tile[x][y]]][y + dy[same_tile[x][y]]] = false;

        return true;
    };

    int cnt = 0;
    string rec = "";
    dfs(dfs, cnt, rec, inp.si, inp.sj);

    int t_x = inp.si, t_y = inp.sj;
    seen.assign(N, vector<bool>(N, false));

    for (char c : ans)
    {
        root[t_x][t_y] = c;
        seen[t_x][t_y] = true;
        seen[t_x + dx[same_tile[t_x][t_y]]][t_y + dy[same_tile[t_x][t_y]]] = true;
        if (c == 'D')
        {
            ++t_x;
        }
        else if (c == 'R')
        {
            ++t_y;
        }
        else if (c == 'U')
        {
            --t_x;
        }
        else
        {
            --t_y;
        }
    }
    seen[t_x][t_y] = true;
    seen[t_x + dx[same_tile[t_x][t_y]]][t_y + dy[same_tile[t_x][t_y]]] = true;

    State S(seen, root, score);

    SA solver(inp, same_tile, S, END_TIME, 250, 0);

    solver.search(startClock);

    seen = solver.S.seen;
    root = solver.S.root;
    score = solver.S.score;

    ans = "";
    t_x = inp.si, t_y = inp.sj;
    while (root[t_x][t_y] != 'X')
    {
        ans += root[t_x][t_y];
        // cerr << t_x << " " << t_y << endl;
        if (root[t_x][t_y] == 'D')
        {
            ++t_x;
        }
        else if (root[t_x][t_y] == 'R')
        {
            ++t_y;
        }
        else if (root[t_x][t_y] == 'U')
        {
            --t_x;
        }
        else
        {
            --t_y;
        }
    }

    cerr << score << " " << len(ans) << endl;
    cout << ans << endl;
}

int main()
{
    ios::sync_with_stdio(false);
    cin.tie(nullptr);

    vector<int> perm;
    for (int i = 0; i < 4; i++)
    {
        perm.push_back(i);
    }
    do
    {
        perms.push_back(perm);
    } while (next_permutation(perm.begin(), perm.end()));

    N = 50;

    INPUT inp(N);

    solve(inp);

    return 0;
}