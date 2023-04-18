use std::io;
use std::env;
use std::fs::File;

use ndarray::Array2;
//use shakmaty::{Bitboard, board, Board,Square, Color::*};
use shakmaty::{fen::Fen, CastlingMode, Chess, Position,  Role, };
use shakmaty::EnPassantMode::Legal;
use pgn_reader::{Visitor, Skip, RawHeader, BufferedReader, SanPlus};


use uciengine::analysis::*;
use uciengine::uciengine::*;
use async_trait::async_trait;
//use tokio::io::{AsyncReadExt, AsyncWriteExt};

use ndarray::{Axis, Array, Array3};
use ndarray_npy::NpzWriter;

struct LastPosition {
    pos: Chess,
    the_fens: Vec<Fen>,
    games: usize,
}

impl LastPosition {
    fn new() -> LastPosition {
        LastPosition { pos: Chess::default(), 
                        the_fens: Vec::new(),
                        games: 0,}
    }
}

#[async_trait(?Send)]
impl Visitor for LastPosition {
    type Result = Chess;

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        // Support games from a non-standard starting position.
        if key == b"FEN" {
            let pos = Fen::from_ascii(value.as_bytes()).ok()
                .and_then(|f| f.into_position(CastlingMode::Standard).ok());

            if let Some(pos) = pos {
                self.pos = pos;
            }
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }
   

   fn san(&mut self, san_plus: SanPlus, ){
        if let Ok(m) = san_plus.san.to_move(&self.pos) {
            self.pos.play_unchecked(&m);
            self.the_fens.push(Fen::from_position(self.pos.clone(), Legal));
        }
        
    }

   
    fn end_game(&mut self) -> Self::Result {
        self.games += 1;
        ::std::mem::replace(&mut self.pos, Chess::default())
    }
}
    
#[tokio::main]
async fn main() -> io::Result<()> {

    let arg: Vec<String> = env::args().collect();  
    let file = File::open(&arg[1])?;         
    let uncompressed: Box<dyn io::Read> = Box::new(file);
    let mut reader = BufferedReader::new(uncompressed);
    //let duration = start.elapsed().unwrap();
    //println!("Time to read pgn: {:?}", duration);
    
    //sets up visitor under 'last position' struct and utilizes custom methods to collect FEN
    let mut visitor = LastPosition::new();

    //reads everything in the file
    reader.read_all(&mut visitor)?;

    //create an ndarray of size the_fens.len()
    //let fen_count = visitor.the_fens.len();

    //let mut board_3d_arr = Vec::new();
    let mut board_3d_arr = Array3::zeros((12, 8, 8));

    let mut eval_vec_centipawn= Vec::new();
    let mut eval_vec_mate = Vec::new();
    let mut count = 0;   

    let bb_build = [Role::Pawn, Role:: Bishop, Role::Knight, Role::Queen, Role::King, Role::Rook];

    //print out how FEN we have
    println!("FEN count: {}", visitor.the_fens.len());

    while count < visitor.the_fens.len(){
    //while count < 12{
        //for every 100 FEN, provide a progress update
        if count % 100 == 0{

            //print copmpletion percentage based on count and total number of FEN
            println!("***********    {}% complete**********", (count as f32 / visitor.the_fens.len() as f32) * 100.0);
            
        }
        
        //set a start time for the beginning of the loop
        //let start = SystemTime::now();
        
        let pos:Option<Chess> = Fen::from_position(
                                    visitor.pos.clone(), Legal)
                                    .into_position(
                                    CastlingMode::Standard).ok();
        let chess = pos.unwrap();
        let result = eval_move(&visitor.the_fens[count]).await;


        match result {
            Score::Cp(cp) => {
                eval_vec_centipawn.push(cp);
                eval_vec_mate.push(0_i32);
            },
            Score::Mate(mate) => {
                eval_vec_mate.push(mate);
                eval_vec_centipawn.push(0_i32);
            },

        }
        

        //print the current visitor FEN
        
        //let bitboards = bitbuild(chess);

        for role in bb_build.iter(){
        
            let role_bb: &u64 = &chess.our(*role).into();

            let encoded_role_bb = one_hot_encoding(*role_bb);
            board_3d_arr.push(Axis(0), encoded_role_bb.view()).unwrap();
    
        }

        for role in bb_build.iter(){
        
            let role_bb: &u64 = &chess.their(*role).into();

            let encoded_role_bb = one_hot_encoding(*role_bb);
            board_3d_arr.push(Axis(0), encoded_role_bb.view()).unwrap();
    
        }
        
        //print the resulting board3d array
        //println!("{:?}", board_3d_arr);


        count += 1;
    } 
    //use the npzwriter to write the board_3d_arr and the eval_vec to a npz file
    let mut npz = NpzWriter::new(File::create("test.npz")?);
    let eval_cp_ndarray = Array::from_vec(eval_vec_centipawn);
    let eval_mate_ndarray = Array::from_vec(eval_vec_mate);
    
    npz.add_array("board_3d_arr", &board_3d_arr).unwrap();

    npz.add_array("eval_cp", &eval_cp_ndarray).unwrap();
    npz.add_array("eval_mate", &eval_mate_ndarray).unwrap();

    npz.finish().unwrap();
    Ok(())

 }

 
 async fn eval_move(pos: &Fen,) -> Score {
    //provide a start time for the beginning of the function
    //let start = SystemTime::now();

    let engine = UciEngine::new
    (r"C:\Users\tjowaisas\OneDrive - Evolv Technology\Documents\Hobby\Chess\Engine\stockfish_15.1_win_x64_avx2\stockfish-windows-2022-x86-64-avx2.exe");
    
    let fen= pos.to_string();
    
   // let setup_job = GoJob::new()
   // .uci_opt("Hash", 1024)
   // .uci_opt("Threads", 10);
    
   // let _result = engine.check_ready(setup_job).await.unwrap();
    
    let go_job = GoJob::new()
        .pos_fen(fen)
        .go_opt("nodes", 20 * 1000)
        .go_opt("depth", 15);

    let result = engine.go(go_job).await.unwrap();
    let eval = result.ai.score;
    //engine.quit();

    //print the time it took to evaluate the move
    //let duration = start.elapsed().unwrap();
    //println!("Time to evaluate move: {:?}", duration);
    
    return eval

 }
 
 /*
 fn bitbuild(pos: Chess ) -> Vec<u64>{
    
    let bb_build = [Role::Pawn, Role:: Bishop, Role::Knight, Role::Queen, Role::King, Role::Rook];
    let chess = pos.clone();
    let mut bitboards:Vec<u64> = Vec::new();

    for thing in bb_build.iter(){
        
        bitboards.push(chess.our(*thing).into());


    }
    for thing in bb_build.iter(){
        
        bitboards.push(chess.their(*thing).into());
       
    }
    bitboards

}
*/

//thanks chatgpt for the help with this function
fn one_hot_encoding(value: u64) -> Array2<u8> {
    let bits = (0..64).map(|i| ((value >> i) & 1) as u8);
    let mut matrix = Array2::<u8>::zeros((8, 8));
    for (i, bit) in bits.enumerate() {
        let row = i / 8;
        let col = i % 8;
        matrix[[row, col]] = bit;
    }
    matrix
}