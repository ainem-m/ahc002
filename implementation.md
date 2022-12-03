# AHC002 Walking on Tiles 〜実装編〜
問題文を理解したところでまずやりたいこと、アイデア出しですよね。  
いろんな戦法が浮かびますよね。いろんな戦法が浮かびましたか？  

そのうちのどれかは**ハズレ解法**かもしれません。そのアイデアのうちいくつかを後で**合体**させるとスコアが伸びるかもしれません。

アイデアはメモ書きに留め、まずは状態を表す構造体など、後半を見据えた実装をすべきです。さもなければ、後に解法の乗り換えが不可能になってしまいます（あ、ここも変えなきゃ、あ、やっぱもとに戻したいな、など、考えるだけで胃が痛くなりますね、うぅ）

ところで、この問題設定、[世界四連覇AIエンジニアがゼロから教えるゲーム木探索入門](https://qiita.com/thun-c/items/058743a25c37c87b8aa4)のゲーム例: 数字集め迷路にそっくりじゃありませんか？
ということで[thunderさん](https://twitter.com/thun_c)から許可を頂いてサンプルコードのアルゴリズム部分にゲーム木探索の実装を使わせていただきました！



## 盤面を表す構造体を作る
この辺、我流なのでもし設計時点での便利な方法なんてございましたらこっそり教えて下さい。

入力で与えられるもの
- タイルの配置
- 得点の配置
これらはゲーム中変化することがないので盤面でもつ必要はないですね。

構造体に必要なもの
- 現在位置
- 踏んだタイル
- 現在の得点
- 最初の行動


これをRustで実装すると

```Rust
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
		// タイルの枚数(input.tilesの最大値)
        let M_ = input
            .tiles
            .iter()
            .map(|t| t.iter().max().unwrap())
            .max()
            .unwrap()
            + 1;
		// タイルを踏んだかどうか
        let mut seen_ = vec![false; M_];
		// 現在位置
        let pos_ = Position {
            i_: pos.0,
            j_: pos.1,
        };
		// 現在位置は踏んでおく
        seen_[input.tiles[pos_.i_][pos_.j_]] = true;
		// 移動経路(座標)
        let steps_ = vec![(pos_.i_, pos_.j_)];
		// 得点(実際の得点)
        let game_score_ = input.ps[pos_.i_][pos_.j_];
		// 探索上で評価したスコア
        let evaluated_score_ = 0;
        Self {
            END_TURN_: end_turn, // 終了するターン(サンプルでは使用していない)
            turn_: 0, // 現在のターン
            seen_,
            pos_,
            steps_,
            output_: String::new(), // 出力用移動経路
            game_score_,
            evaluated_score_,
            first_action_: !0, // 探索木のルートノードで
        }
    }
```


## ビジュアライザからわかること

`tools/src/bin/lib.rs`
```rust
pub fn compute_score_detail(input: &Input, out: &Output) -> (i32, String, Vec<usize>, Vec<(usize, usize)>) {
	let mut used = vec![0; N * N];  //本当はタイルの総数Mで十分だが、N*Nで足りなくなることはない
	let (mut i, mut j) = input.s; // 現在位置、最初はスタート地点
	used[input.tiles[i][j]] = 1; // スタート地点を踏む
	let mut score = input.ps[i][j]; // スタート地点の得点も含む
	let mut steps = vec![(i, j)]; // 通った経路
	let mut err = String::new();
	for c in out.chars() { // 出力を一文字ずつ見ていく
		let (di, dj) = match c { // cがLRUDのうちどれかによって値を変える
			'L' => (0, !0), // Lなら(0, !0)
			'R' => (0, 1), // 以下略
			'U' => (!0, 0),
			'D' => (1, 0),
			_ => {
				return (0, "Illegal output".to_owned(), used, steps);
			}
		};
		i += di; // 現在地点から移動
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
```
わりと素直な実装な気がします。

> ##### !0について
> Rustのusize型は実行環境によってu32かu64として扱われる**符号なし**の整数型です。
> Rustでは配列のindexは必ずusize型にしなければいけない決まりがあり、困ってしまうのが、グリッド探索をしたい今回のような場合です。
> (`[(1, 0), (0, 1), (-1, 0), (0, -1)]`のような配列を作れない)
> そこでわざとオーバーフローを起こして
> ```i + !0 = i - 1 + std::usize::MAX + 1 = i - 1```
> と、うまく計算するテクニックが競プロではよく使われます(業プロでも使われますか？)
> `!0`は`0`のbit否定(pythonでいう`~`チルダ)で、usizeの最大値をとります。
> `0 + !0 = !0`(>N)となるので境界条件の設定が`(i + di) < N`だけで済みます。
> [えびちゃん先生とのやりとり](https://twitter.com/rsk0315_h4x/status/1588542877269098497)も是非ご覧ください


## 自前のビジュアライザ
<img src = "to_string.png" width = 100% alt="今回の力作">


## 乱択

## 貪欲法

## ビームサーチ

## ビームサーチ（時間設定）

## chokudaiサーチ

## chokudaiサーチ(時間設定)
たすけてー

## 焼きなましについて
今回の問題は「どう焼くか」を考えることが点数に結びつくと考えたため、焼きなましについては実装されていません！(焼きなまし法の理解については[Introduction to Heuristics Contest 解説](https://img.atcoder.jp/intro-heuristics/editorial.pdf)や[ナップサック問題を様々な解法で解く](https://colab.research.google.com/drive/1-ou5jI6xCebLn-5cEIOeUBRO02aBz0nC?usp=sharing)を、焼きなましの改善については[焼きなましのコツ](https://shindannin.hatenadiary.com/entry/2021/03/06/115415)をオススメします)