import random
# 標準出力のリダイレクトにseeds.txtを指定する

random.seed(2022121292044)  # システス準備を始めた時刻

for _ in range(1000):
    print(random.randint(0, 10000000000))
