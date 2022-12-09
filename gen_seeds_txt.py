import random
# 標準出力のリダイレクトにseeds.txtを指定する

random.seed(20210325)  # 本番はこの値を変更します

for _ in range(1000):
    print(random.randint(0, 10000000000))
