extern crate hebb;

use hebb::ir::{LBuilder};
use hebb::lang::{HLexer, HParser};

#[test]
fn test_ir_1() {
  println!();
  //let lexer = HLexer::new("let x = 1; let y = 2; let z = x + y; z");
  let lexer = HLexer::new("let x = 1; let y = 2; let z = x; z");
  let parser = HParser::new(lexer);
  let htree = parser.parse();
  println!("DEBUG: htree: {:?}", htree);
  let mut builder = LBuilder::new();
  //let ltree = builder._htree_to_ltree_lower_pass(htree);
  let ltree = builder.lower(htree);
  //builder._debug_dump_vars();
  let ltree = ltree.with_env_info();
  let ltree = ltree.with_free_env_info();
  println!("DEBUG: ltree pretty printed:");
  ltree._pretty_print();
}

#[test]
fn test_ir_2() {
  println!();
  //let lexer = HLexer::new("let x = 1; let y = 2; let z = x + y; z");
  let lexer = HLexer::new("let x = 1; let y = let w = 2 in w; let z = x[y[y]]; y");
  let parser = HParser::new(lexer);
  let htree = parser.parse();
  println!("DEBUG: htree: {:?}", htree);
  let mut builder = LBuilder::new();
  let ltree = builder.lower(htree);
  let ltree = ltree.with_env_info();
  let ltree = ltree.with_free_env_info();
  //println!("DEBUG: ltree: {:?}", ltree);
  println!("DEBUG: ltree, pretty printed:");
  ltree._pretty_print();
  let ltree = builder.normalize(ltree);
  let ltree = ltree.with_env_info();
  let ltree = ltree.with_free_env_info();
  //println!("DEBUG: a-normalized ltree: {:?}", ltree);
  println!("DEBUG: a-normalized ltree, pretty printed:");
  ltree._pretty_print();
}

#[test]
fn test_ir_adj() {
  println!();
  let lexer = HLexer::new("let x = 1; let y = let w = x in w; let z = adj y; z");
  let parser = HParser::new(lexer);
  let htree = parser.parse();
  println!("DEBUG: htree: {:?}", htree);
  let mut builder = LBuilder::new();
  let ltree = builder.lower(htree);
  //let ltree = builder.lower_with_stdlib(htree);
  let ltree = ltree.with_env_info();
  let ltree = ltree.with_free_env_info();
  //println!("DEBUG: ltree: {:?}", ltree);
  println!("DEBUG: ltree, pretty printed:");
  ltree._pretty_print();
  let ltree = builder.normalize(ltree);
  let ltree = ltree.with_env_info();
  let ltree = ltree.with_free_env_info();
  //println!("DEBUG: a-normalized ltree: {:?}", ltree);
  println!("DEBUG: a-normalized ltree, pretty printed:");
  ltree._pretty_print();
}
