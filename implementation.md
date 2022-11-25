# AHC002 Walking on Tiles 〜実装編〜
問題文を理解したところでまずやりたいこと、アイデア出しですよね。  
いろんな戦法が浮かびますよね。いろんな戦法が浮かびましたか？  

そのうちのどれかは**ハズレ解法**かもしれません。そのアイデアのうちいくつかを後で**合体**させるとスコアが伸びるかもしれません。

アイデアはメモ書きに留め、まずは状態を表す構造体を実装すべきです。さもなければ、後に解法の乗り換えが不可能になってしまいます（あ、ここも変えなきゃ、あ、やっぱもとに戻したいな、など、考えるだけで胃が痛くなりますね、うぅ）


ところで、この問題設定、[世界四連覇AIエンジニアがゼロから教えるゲーム木探索入門](https://qiita.com/thun-c/items/058743a25c37c87b8aa4)のゲーム例: 数字集め迷路にそっくりじゃありませんか？

## 盤面を表す構造体を作る
入力で与えられるもの
- タイルの配置
- 得点の配置

構造体に必要なもの
- 現在位置
- 踏んだタイル
- 現在の得点
- 最初の行動(*後述)

これをRustで実装すると
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
> そこでわざとオーバーフローを起こし `i + !0 = i - 1` とするテクニックが競プロではよく使われます(業プロでも使われますか？)
> `!0`は`0`のbit否定(pythonでいう`~`チルダ)で、usizeの最大値をとります。
> `0 + !0 = !0`(>N)となるので境界条件の設定が`(i + di) < N`だけで済みます。
> [えびちゃん先生とのやりとり](https://twitter.com/rsk0315_h4x/status/1588542877269098497)も是非ご覧ください



