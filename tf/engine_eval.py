import chess
import chess.engine
import chess.svg
import random
import numpy
import time
import os

sf_path = r"C:\Users\tjowaisas\OneDrive - Evolv Technology\Documents\Hobby\Chess\Engine\stockfish_15.1_win_x64_avx2\stockfish-windows-2022-x86-64-avx2.exe"
lc0_path= r"C:\Users\tjowaisas\OneDrive - Evolv Technology\Documents\Hobby\Chess\Engine\Leela\lc0.exe"

# this function will create our x (board)
def random_board(max_depth=30):
  board = chess.Board()
  depth = random.randrange(0, max_depth)

  for _ in range(depth):
    all_moves = list(board.legal_moves)
    random_move = random.choice(all_moves)
    board.push(random_move)
    if board.is_game_over():
      break
  return board


# this function will create our f(x) (score)
def stockfish(board, depth):
  # depth 10 takes .2s
  # depth 20 takes .7s
  # depth 25 takes 3.4s
  # depth 30 takes 14.3s
  with chess.engine.SimpleEngine.popen_uci(sf_path) as sf:
    result = sf.analyse(board, chess.engine.Limit(depth=depth))
    score = result['score'].white().score()
    return score

squares_index = {
  'a': 0,
  'b': 1,
  'c': 2,
  'd': 3,
  'e': 4,
  'f': 5,
  'g': 6,
  'h': 7
}

# example: h3 -> 17
def square_to_index(square):
  letter = chess.square_name(square)
  return 8 - int(letter[1]), squares_index[letter[0]]

def split_dims(board):
  # this is the 3d matrix
  board3d = numpy.zeros((14, 8, 8), dtype=numpy.int8)

  # here we add the pieces's view on the matrix
  for piece in chess.PIECE_TYPES:
    for square in board.pieces(piece, chess.WHITE):
      idx = numpy.unravel_index(square, (8, 8))
      board3d[piece - 1][7 - idx[0]][idx[1]] = 1
    for square in board.pieces(piece, chess.BLACK):
      idx = numpy.unravel_index(square, (8, 8))
      board3d[piece + 5][7 - idx[0]][idx[1]] = 1

  # add attacks and valid moves too
  # so the network knows what is being attacked
  aux = board.turn
  board.turn = chess.WHITE
  for move in board.legal_moves:
      i, j = square_to_index(move.to_square)
      board3d[12][i][j] = 1
  board.turn = chess.BLACK
  for move in board.legal_moves:
      i, j = square_to_index(move.to_square)
      board3d[13][i][j] = 1
  board.turn = aux

  return board3d

db_size = 5000

#print(sf_array)
#print(split_array)

#print(sf_array.ndim)
split_array = numpy.zeros([db_size, 14, 8, 8])
sf_array = numpy.zeros([db_size,1])

for x in range(db_size):

  print("Producing board " + str(x) + " of " + str(db_size))
  board = random_board()
  sf = stockfish(board, 10)

  while sf is None:
    board = random_board()
    sf = stockfish(board, 10)

  #print(sf)
  sf_array[x] = sf
  
  if numpy.isnan(sf_array[x]):
    boardsvg = chess.svg.board(board, size=300, coordinates=True)
    with open('temp.svg', 'w') as outputfile:
      outputfile.write(boardsvg)
      time.sleep(0.1)
      os.startfile('temp.svg')
  #print(sf_array[x])

  split = split_dims(board)
  split_array[x] = split

numpy.savez('dataset.npz', board=split_array, eval=sf_array, )

#print(sf_array)
#print(split_array)
"""
board = random_board()

boardsvg = chess.svg.board(board, size=300, coordinates=True)
with open('temp.svg', 'w') as outputfile:
  outputfile.write(boardsvg)
  time.sleep(0.1)
  os.startfile('temp.svg')
"""
