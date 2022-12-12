import subprocess

NUM_OF_TEAMS = 6
NUM_OF_CASES = 1000
# パソコンのプロセス数
max_process = 8  # 要変更
proc_list = []

IN = "tools/in/"
OUT = "tools/out/"
ERR = "tools/err/"
score_list = [[-1 for _ in range(NUM_OF_CASES)] for _ in range(NUM_OF_TEAMS)]


# 以下の形式のfilelist.txtを標準入力で読み込む
# file名 言語 team名
# がNUM_OF_TEAMS行書かれたテキストファイル
for team_num in range(NUM_OF_TEAMS):
    args = input().split()
    filename, lang, team = args

    # 言語ごとにコンパイルなどの準備
    if lang == "Python":
        command = f"pypy {filename}"
    elif lang == "Rust":
        subprocess.run(
            f"cargo build --bin {filename} --release 2>/dev/null", shell=True
        )
        command = f"../target/release/{filename}"
    elif lang == "C++":
        subprocess.run(
            f"g++ -std=gnu++17 -O2 -o ./{team}.out ./{filename} 2>/dev/null", shell=True
        )
        command = f"./{team}.out"
    elif lang == "Crystal":
        subprocess.run(f"crystal build {filename} --release 2>/dev/null", shell=True)
        command = "./" + team  # 拡張子を除いたファイル名

    # 出力フォルダを準備
    subprocess.run(f"rm -rf {OUT}{team}", shell=True)
    subprocess.run(f"mkdir {OUT}{team}", shell=True)
    subprocess.run(f"rm -rf {ERR}{team}", shell=True)
    subprocess.run(f"mkdir {ERR}{team}", shell=True)
    """
    # システムテスト実行
    for i in range(NUM_OF_CASES):
        subprocess.run(
            f"{command} <{IN}{i:04d}.txt >{OUT}{team}/{i:04d}.txt 2>{ERR}{team}/{i:04d}.txt",
            shell=True,
        )
        print(f"\rrunning {i:04d}", end="")
    print()
    """
    # システムテスト実行
    for i in range(NUM_OF_CASES):
        proc = subprocess.Popen(
            f"{command} <{IN}{i:04d}.txt >{OUT}{team}/{i:04d}.txt 2>{ERR}{team}/{i:04d}.txt",
            shell=True,
        )
        proc_list.append(proc)
        print(f"\rrunning {i:04d}", end="")
        if (i + 1) % max_process == 0 or (i + 1) == NUM_OF_CASES:
            for subproc in proc_list:
                subproc.wait()
                # time.sleep(0.1)
        proc_list = []
    print()

    sum_score = 0

    # 出力からビジュアライザを用いて得点計算
    for i in range(NUM_OF_CASES):
        result = subprocess.run(
            f"cargo run --release --bin vis in/{i:04d}.txt out/{team}/{i:04d}.txt",
            shell=True,
            cwd=r"tools/",
            capture_output=True,
            text=True,
        )
        score = int(result.stdout.split()[-1])
        sum_score += score
        score_list[team_num][i] = score
        print(f"{team} {i:04d} score: {score}")
    score_list[team_num].append(sum_score)
    print(f"{team} score: {sum_score}")

result_list = open("result.csv", "w")
print("team, ", end="", file=result_list)
for i in range(NUM_OF_CASES):
    print(f"{i:04d}", end=", ", file=result_list)
print("sum", file=result_list)

for team in range(NUM_OF_TEAMS):
    print(f"team{team+1}, ", end="", file=result_list)
    for case in range(NUM_OF_CASES + 1):
        print(score_list[team][case], ", ", end="", file=result_list)
    print("", file=result_list)
