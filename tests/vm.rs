extern crate hebb;

use hebb::ir::{LBuilder};
use hebb::lang::{HLexer, HParser};
use hebb::vm::{VMachine};

#[test]
fn test_vm_1() {
  println!();
  //let lexer = HLexer::new("let x = 1; let y = 2; y");
  let lexer = HLexer::new("let x = 1; let y = 2; let x = bot; let t = tee; x");
  //let lexer = HLexer::new("let x = 1; let y = 2; let z = x + y; z");
  let parser = HParser::new(lexer);
  let htree = parser.parse();
  println!("DEBUG: htree: {:?}", htree);
  let mut builder = LBuilder::new();
  //let ltree = builder._htree_to_ltree_lower_pass(htree);
  let ltree = builder.lower(htree);
  builder._debug_dump_vars();
  println!("DEBUG: ltree: {:?}", ltree);
  //let letree = builder._ltree_env_pass(ltree);
  //println!("DEBUG: letree: {:?}", letree);
  let mut vm = VMachine::new();
  vm.eval(ltree);
  vm._debug_dump_ctrl();
}
