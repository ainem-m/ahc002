START_TIME = Time.utc
INF        = 10i64 ** 9 # 焼きなまし用

module G
  class_property scores : Array(Array(Int32)) = Array.new(50) { [0] * 50 }
  class_property tiles : Array(Array(Int32)) = Array.new(50) { [0] * 50 }
end

# 入力読み込み
si, sj = read_line.split.map(&.to_i)

board = Board.new(si, sj)

50.times do |i|
  read_line.split.map(&.to_i).each_with_index do |t, j|
    G.tiles[i][j] = t
  end
end

50.times do |i|
  read_line.split.map(&.to_i).each_with_index do |s, j|
    G.scores[i][j] = s
  end
end

# 移動方向は「」状になる8パターン保持する
U    = {-1, 0, 'U'}
D    = {1, 0, 'D'}
R    = {0, 1, 'R'}
L    = {0, -1, 'L'}
URLD = [U, R, L, D]
DRLU = [D, R, L, U]
ULRD = [U, L, R, D]
DLRU = [D, L, R, U]
RUDL = [R, U, D, L]
LUDR = [L, U, D, R]
RDUL = [R, D, U, L]
LDUR = [L, D, U, R]

DIR_PATTERNS = [
  URLD,
  DRLU,
  ULRD,
  DLRU,
  RUDL,
  LUDR,
  RDUL,
  LDUR,
]

# まずは8パターンDFSして初期解を作る
solvers = [] of Solver

DIR_PATTERNS.each do |dirs|
  solver = Solver.new(si, sj, board, Time.utc, dirs)
  solver.dfs_first_ans
  solvers << solver
end

# 最もスコアの高い初期解で焼きなまし
solvers.sort_by! { |s| -s.score }

best_solver = solvers[0]
puts best_solver.do_simurated_annearing

# 盤面
struct Board
  @done : Array(Bool) = [false] * 2500

  def initialize(@si : Int32, @sj : Int32)
  end

  property si, sj, done
end

# ソルバー
class Solver
  @score = 0
  @path = ""
  @dfs_time_limit : Float64 = 0.02
  @simulated_annealing_time_limit : Float64 = 1.98
  @best_score = 0
  @best_path = ""

  def initialize(@si : Int32, @sj : Int32, @board : Board, @start_time : Time, @dirs : Array(Tuple(Int32, Int32, Char)))
    @i = @si
    @j = @sj
    @gi = @si
    @gj = @si
    @dir_dict = Hash(Char, Tuple(Int32, Int32, Char)).new
    @dir_dict['U'] = U
    @dir_dict['D'] = D
    @dir_dict['R'] = R
    @dir_dict['L'] = L
    @origin_board = @board
  end

  property score, path, best_score, best_path, board

  # DFS発動
  def dfs_first_ans
    self.dfs_route(@board, @i, @j, 'S', @score, @path, 0)
    @path = @path.lchop
    @best_score = @score
    @best_path = @path
    @score
  end

  # 焼きなまし発動
  def do_simurated_annearing
    self.simulated_annealing
    @best_path
  end

  # 移動可能か確認
  private def can_move?(board : Board, i2 : Int32, j2 : Int32)
    0 <= i2 < 50 && 0 <= j2 < 50 && !board.done[G.tiles[i2][j2]]
  end

  # 盤面を更新しながら通過した点を記録し、通過した点の配列を返す
  private def move_log_and_update_board
    cells = [] of Tuple(Int32, Int32, Int32)
    i2, j2 = @si, @sj
    @board.done[G.tiles[i2][j2]] = true
    @path.chars.each_with_index do |d, di|
      i2 += @dir_dict[d][0]
      j2 += @dir_dict[d][1]
      @board.done[G.tiles[i2][j2]] = true
      cells << {i2, j2, di}
    end
    @gi = i2
    @gj = j2
    cells
  end

  # 削除した経路を通過していないことにして、新しい盤面と削除された部分のスコアを返す
  private def mid_change_board(sdi : Int32, gdi : Int32, board : Board, done_state : Bool, cells : Array(Tuple(Int32, Int32)))
    removed_score = 0
    chars = @path.chars
    sdi.upto(gdi - 1) do |ii|
      cell = cells[ii]
      i = cell[0]
      j = cell[1]
      board.done[G.tiles[i][j]] = done_state
      removed_score += G.scores[i][j]
    end
    {board, removed_score}
  end

  # 焼きなまし
  # 経路の一部を破壊してDFSで再度繋ぐ
  private def simulated_annealing
    temp_start = 100 # 温度関数に使用
    temp_end = 10    # 温度関数に使用
    loop_times = 0   # ループ回数の記録 デバッグ用
    changed = true
    position_of_path = [{-1, -1}] * 2500
    i = @si
    j = @sj
    path_end = 0
    @path.chars.each_with_index do |d, di|
      i += @dir_dict[d][0]
      j += @dir_dict[d][1]
      position_of_path[di] = {i, j}
      @gi = i
      @gj = j
      path_end = di
    end
    self.move_log_and_update_board
    while (Time.utc - START_TIME).total_seconds <= @simulated_annealing_time_limit
      loop_times += 1
      sdi = Random.rand(0..(path_end - 1)) # ここから先をちぎる
      start_i, start_j = position_of_path[sdi]
      next if start_i == @si && start_j == @sj
      next if start_i == @gi && start_j == @gj
      next if sdi + 1 >= path_end
      next if start_i == -1 || start_j == -1
      # ちぎるゾーンの終点
      break_range = Random.rand(5..25)
      gdi = Random.rand((sdi + 1)..(sdi + break_range))
      next if gdi >= path_end - 1
      goal_i, goal_j = position_of_path[gdi]
      next if goal_i == @si && goal_j == @sj
      next if goal_i == @gi && goal_j == @gj
      next if goal_i == -1 || goal_j == -1
      next if G.tiles[start_i][start_j] == G.tiles[goal_i][goal_j]
      front_path = @path[..sdi]
      back_path = @path[gdi..].lchop
      old_done = board.done.clone
      board, removed_score = mid_change_board(sdi, gdi, @board, false, position_of_path)
      remain_score = @score - removed_score
      board.done[G.tiles[goal_i][goal_j]] = false
      # ちぎった区間をDFS
      reconnect_path_and_score = self.reconnect(board, start_i, start_j, goal_i, goal_j, 'S', 0, "", true, DIR_PATTERNS.sample)
      changed = false if reconnect_path_and_score.nil?
      @board.done = old_done
      unless reconnect_path_and_score.nil?
        mid_path, mid_score, new_board = reconnect_path_and_score
        new_score = remain_score + mid_score
        # 温度関数
        temp = temp_start + (temp_end - temp_start) * (Time.utc - START_TIME).total_seconds / @simulated_annealing_time_limit
        next if @score == new_score
        # 遷移確率関数
        prob = Math.exp((new_score - @score) / temp)
        rand_mod = ((Random.rand(Int32::MIN..Int32::MAX) % INF) / INF)
        force_next = prob > rand_mod
        if new_score > @score || force_next
          @score = new_score
          @path = front_path + mid_path + back_path
          @board = board
          path_end = @path.size - 1
          di = sdi + 1
          new_i = start_i
          new_j = start_j
          "#{mid_path}#{back_path}".chars.each_with_index do |mp, mi|
            new_i += @dir_dict[mp][0]
            new_j += @dir_dict[mp][1]
            @board.done[G.tiles[new_i][new_j]] = true
            position_of_path[di + mi] = {new_i, new_j}
          end
          @board.done[G.tiles[start_i][start_j]] = true
          @board.done[G.tiles[goal_i][goal_j]] = true
          prev_goal_i = goal_i
          prev_goal_j = goal_j
          changed = true
          if @score > @best_score
            @best_score = @score
            @best_path = @path
          end
        else
          changed = false
        end
      end
    end
    @path
  end

  # ちぎった区間をDFSで再接続
  private def reconnect(board : Board, start_i : Int32, start_j : Int32, goal_i : Int32, goal_j : Int32, dir_char : Char, current_score : Int32, current_path : String, is_start : Bool, dirs : Array(Tuple(Int32, Int32, Char)))
    stack = Deque(Tuple(Board, Int32, Int32, Char, Int32, String)).new
    stack << {board, start_i, start_j, dir_char, current_score, current_path}
    start_time = Time.utc
    done = board.done.clone
    origin_i = start_i
    origin_j = start_j
    until stack.empty?
      break if (Time.utc - start_time).total_seconds > 0.01
      board, start_i, start_j, dir_char, current_score, current_path = stack.pop
      if start_i == goal_i && start_j == goal_j && 0 <= start_i < 50 && 0 <= start_j < 50
        board.done = done
        board.done[G.tiles[start_i][start_j]] = true
        return {current_path, current_score, board}
      end
      i = origin_i
      j = origin_j
      current_path.chars.each do |d|
        i += @dir_dict[d][0]
        j += @dir_dict[d][1]
        done[G.tiles[i][j]] = true
      end
      done[G.tiles[start_i][start_j]] = true
      current_score += G.scores[start_i][start_j]
      go_next = false
      dirs.each do |dir|
        i2, j2, d_char = start_i + dir[0], start_j + dir[1], dir[2]
        next if dir_char == 'U' && d_char == 'D'
        next if dir_char == 'D' && d_char == 'U'
        next if dir_char == 'R' && d_char == 'L'
        next if dir_char == 'L' && d_char == 'R'
        if 0 <= i2 < 50 && 0 <= j2 < 50 && !done[G.tiles[i2][j2]]
          stack << {board, i2, j2, d_char, current_score, current_path + d_char}
          go_next = true
        end
      end
      unless go_next
        i = origin_i
        j = origin_j
        current_path.chars.each do |d|
          i += @dir_dict[d][0]
          j += @dir_dict[d][1]
          done[G.tiles[i][j]] = false
        end
      end
    end
    return nil
  end

  # 初期解をDFS スコティさんのを移植
  private def dfs_route(board : Board, i2 : Int32, j2 : Int32, dir_char : Char, current_score : Int32, current_path : String, renzoku_dir : Int32)
    if !self.can_move?(board, i2, j2)
      if current_score > @score
        @score = current_score
        @path = current_path
        @board = board
      end
      return
    end
    if (Time.utc - @start_time).total_seconds > @dfs_time_limit
      if current_score > @score
        @score = current_score
        @path = current_path
        @board = board
      end
      return
    end
    board.done[G.tiles[i2][j2]] = true
    current_score += G.scores[i2][j2]
    current_path += dir_char
    @dirs.each do |dir|
      i3, j3, d_char = i2 + dir[0], j2 + dir[1], dir[2]
      next if dir_char == 'U' && d_char == 'D'
      next if dir_char == 'D' && d_char == 'U'
      next if dir_char == 'R' && d_char == 'L'
      next if dir_char == 'L' && d_char == 'R'
      renzoku_d = renzoku_dir
      renzoku_d += 1 if dir_char == d_char
      renzoku_d = 0 if dir_char != d_char
      next if renzoku_d > 10
      self.dfs_route(board, i3, j3, d_char, current_score, current_path, renzoku_d)
    end
    current_path = current_path.rchop
    board.done[G.tiles[i2][j2]] = false
    current_score -= G.scores[i2][j2]
    return
  end
end
