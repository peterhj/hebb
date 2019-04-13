// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::lang::{HExpr};
use crate::stdlib::*;
use crate::vm::{VMBoxCode};

use fixedbitset::{FixedBitSet};
use lazycell::{LazyCell};
use serde::{Serialize, Serializer};
//use serde::ser::{SerializeStruct};
use vertreap::{VertreapMap};

//use std::cell::{RefCell};
use std::collections::{HashMap};
//use std::collections::hash_map::{Entry};
use std::fmt::{self, Debug};
use std::io::{Write, stdout};
use std::iter::{FromIterator};
use std::mem::{size_of};
use std::ops::{Deref};
use std::rc::{Rc};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct LLDepth(pub usize);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct LDBIdx(pub usize);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct LVar(pub u64);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct LHash(pub u64);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct LLabel(pub u64);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LVarSet {
  inner:    FixedBitSet,
}

impl FromIterator<LVar> for LVarSet {
  fn from_iter<I: IntoIterator<Item=LVar>>(var_iter: I) -> LVarSet {
    assert!(size_of::<u64>() <= size_of::<usize>());
    LVarSet{
      inner:    FixedBitSet::from_iter(var_iter.into_iter().map(|var| var.0 as usize)),
    }
  }
}

impl<'v> FromIterator<&'v LVar> for LVarSet {
  fn from_iter<I: IntoIterator<Item=&'v LVar>>(var_iter: I) -> LVarSet {
    assert!(size_of::<u64>() <= size_of::<usize>());
    LVarSet{
      inner:    FixedBitSet::from_iter(var_iter.into_iter().map(|var| var.0 as usize)),
    }
  }
}

impl LVarSet {
  pub fn empty() -> LVarSet {
    LVarSet{
      inner:    FixedBitSet::with_capacity(32),
    }
  }

  pub fn insert(&mut self, var: LVar) {
    assert!(size_of::<u64>() <= size_of::<usize>());
    let v = var.0 as usize;
    if v >= self.inner.len() {
      self.inner.grow(v + 1);
    }
    self.inner.insert(v);
  }

  pub fn union(&self, other: &LVarSet) -> LVarSet {
    LVarSet{
      inner:    FixedBitSet::from_iter(self.inner.union(&other.inner)),
    }
  }

  pub fn difference(&self, other: &LVarSet) -> LVarSet {
    LVarSet{
      inner:    FixedBitSet::from_iter(self.inner.difference(&other.inner)),
    }
  }
}

#[derive(Debug, Serialize)]
//#[derive(Debug, Serialize, Deserialize)]
pub enum LTermPat {
  Cons(LPat, LPat),
  Concat(LPat, LPat),
  Tuple(Vec<LPat>),
  Wild,
  UnitLit,
  BitLit(bool),
  IntLit(i64),
  FloLit(f64),
  Var(LVar),
  Alias(LPat, LVar),
}

#[derive(Debug)]
pub struct LTermPatRef {
  inner:    Rc<LTermPat>,
}

impl Clone for LTermPatRef {
  fn clone(&self) -> LTermPatRef {
    LTermPatRef{inner: self.inner.clone()}
  }
}

impl Deref for LTermPatRef {
  type Target = LTermPat;

  fn deref(&self) -> &LTermPat {
    &*self.inner
  }
}

impl Serialize for LTermPatRef {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    (&*self.inner).serialize(serializer)
  }
}

#[derive(Debug, Serialize)]
//#[derive(Debug, Serialize, Deserialize)]
pub struct LPat {
  pub term: LTermPatRef,
}

pub enum LTermVMExt {
  BcLambda(Vec<LVar>, VMBoxCode),
}

impl Debug for LTermVMExt {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    // TODO
    write!(f, "BcLambda(...)")
  }
}

#[derive(Debug)]
//#[derive(Debug, Serialize)]
pub enum LTerm<E=LExpr, X=LTermVMExt> {
  Break(E),
  Trace(E),
  NoRet,
  NonSmooth,
  Apply(E, Vec<E>),
  TailApply(E, Vec<E>),
  Env,
  DynEnv(LEnv),
  Adj(E),
  AdjDyn(E),
  Lambda(Vec<LVar>, E),
  Let(LVar, E, E),
  Fix(LVar, E),
  Switch(E, E, E),
  Match(E, Vec<(LPat, E)>),
  Cons(E, E),
  Concat(E, E),
  Tuple(Vec<E>),
  UnitLit,
  BitLit(bool),
  IntLit(i64),
  FloLit(f64),
  Lookup(LVar),
  TmpEnvLookup(E, LHash),
  EnvLookup(E, LVar),
  DynEnvLookup(E, String),
  //DynEnvLookupStr(E, String),
  //DynEnvLookup(E, LHash),
  VMExt(X),
}

#[derive(Debug)]
pub struct LTermRef<E=LExpr, X=LTermVMExt> {
  inner:    Rc<LTerm<E, X>>,
}

impl<E, X> Clone for LTermRef<E, X> {
  fn clone(&self) -> LTermRef<E, X> {
    LTermRef{inner: self.inner.clone()}
  }
}

impl<E, X> Deref for LTermRef<E, X> {
  type Target = LTerm<E, X>;

  fn deref(&self) -> &LTerm<E, X> {
    &*self.inner
  }
}

impl<E: Serialize, X: Serialize> Serialize for LTermRef<E, X> where LTerm<E, X>: Serialize {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    (&*self.inner).serialize(serializer)
  }
}

impl<E, X> LTermRef<E, X> {
  pub fn new(term: LTerm<E, X>) -> LTermRef<E, X> {
    LTermRef{inner: Rc::new(term)}
  }
}

#[derive(Clone, Default, Debug)]
pub struct LExprInfo {
  pub env:      Option<LEnv>,
  //pub uses:     Option<LVarSet>,
  pub freeuses: Option<LVarSet>,
}

#[derive(Clone, Debug)]
pub struct LExpr {
  pub gen:      u64,
  pub label:    LLabel,
  pub term:     LTermRef,
  pub info:     LExprInfo,
}

impl LExpr {
  pub fn _pretty_print(&self) {
    LPrettyPrinter{}.print(self.clone());
  }

  pub fn with_gen(&self, new_gen: u64) -> LExpr {
    LExpr{
      gen:      new_gen,
      label:    self.label.clone(),
      term:     self.term.clone(),
      info:     self.info.clone(),
    }
  }
}

#[derive(Debug)]
pub struct LTreeEnvInfo {
  pub gen:  u64,
  pub data: HashMap<LLabel, LEnv>,
}

impl LExpr {
  pub fn _env_info_pass(&self, env: LEnv, info: &mut LTreeEnvInfo) {
    match &*self.term {
      &LTerm::Apply(ref head, ref args) => {
        info.data.insert(self.label.clone(), env.clone());
        head._env_info_pass(env.clone(), info);
        for arg in args.iter() {
          arg._env_info_pass(env.clone(), info);
        }
      }
      &LTerm::Adj(ref sink) => {
        info.data.insert(self.label.clone(), env.clone());
        sink._env_info_pass(env.clone(), info);
      }
      &LTerm::Lambda(ref params, ref body) => {
        info.data.insert(self.label.clone(), env.clone());
        let mut body_env = env;
        for param in params.iter() {
          body_env = body_env.fork_bvar(param.clone());
        }
        body._env_info_pass(body_env, info);
      }
      &LTerm::Let(ref name, ref body, ref rest) => {
        info.data.insert(self.label.clone(), env.clone());
        body._env_info_pass(env.clone(), info);
        let rest_env = env.fork(name.clone(), body.clone());
        rest._env_info_pass(rest_env, info);
      }
      &LTerm::Fix(ref fixname, ref fixbody) => {
        // FIXME
        /*let fix_env = env.fork_fixed(ltree.label.clone(), fixname.clone(), fixbody.term.clone(), Some(env.clone()));
        let fixbody = self._env_info_pass(fix_env, fixbody.clone());
        let mut info = ltree.info;
        info.env = Some(env);
        LExpr{
        }*/
        unimplemented!();
      }
      &LTerm::Switch(ref pred, ref top, ref bot) => {
        info.data.insert(self.label.clone(), env.clone());
        pred._env_info_pass(env.clone(), info);
        top._env_info_pass(env.clone(), info);
        bot._env_info_pass(env.clone(), info);
      }
      /*&LTerm::Match(..) => {
      }*/
      &LTerm::Tuple(ref elems) => {
        info.data.insert(self.label.clone(), env.clone());
        for elem in elems.iter() {
          elem._env_info_pass(env.clone(), info);
        }
      }
      &LTerm::UnitLit   |
      &LTerm::BitLit(_) |
      &LTerm::IntLit(_) |
      &LTerm::FloLit(_) |
      &LTerm::Lookup(_) => {
        info.data.insert(self.label.clone(), env);
      }
      &LTerm::TmpEnvLookup(..) => {
        // FIXME
        unimplemented!();
      }
      &LTerm::EnvLookup(..) => {
        // FIXME
        unimplemented!();
      }
      e => {
        panic!("env info pass: unimplemented ltree expr: {:?}", e);
      }
    }
  }

  pub fn env<'root>(&self, ltree: &'root LTree) -> &'root LEnv {
    let root = ltree.root.clone();
    assert_eq!(self.gen, root.gen);
    let env_info = ltree.info.env.borrow_with(move || {
      let mut env_info = LTreeEnvInfo{
        gen:  root.gen,
        data: HashMap::new(),
      };
      root._env_info_pass(LEnv::default(), &mut env_info);
      env_info
    });
    assert_eq!(self.gen, env_info.gen);
    match env_info.data.get(&self.label) {
      None => panic!(),
      Some(env) => env,
    }
  }

  pub fn maybe_env<'root>(&self, ltree: &'root LTree) -> Option<&'root LEnv> {
    ltree.info.env.borrow()
      .and_then(|info| info.data.get(&self.label))
  }
}

#[derive(Debug)]
pub struct LTreeFreeEnvInfo {
  pub gen:  u64,
  pub data: HashMap<LLabel, LVarSet>,
}

impl LExpr {
  pub fn _free_env_info_pass_init(&self, info: &mut LTreeFreeEnvInfo) {
    match &*self.term {
      &LTerm::Apply(ref head, ref args) => {
        info.data.insert(self.label.clone(), LVarSet::empty());
        head._free_env_info_pass_init(info);
        for arg in args.iter() {
          arg._free_env_info_pass_init(info);
        }
      }
      &LTerm::Adj(ref sink) => {
        info.data.insert(self.label.clone(), LVarSet::empty());
        sink._free_env_info_pass_init(info);
      }
      &LTerm::Lambda(ref _params, ref body) => {
        info.data.insert(self.label.clone(), LVarSet::empty());
        body._free_env_info_pass_init(info);
      }
      &LTerm::Let(ref _name, ref body, ref rest) => {
        info.data.insert(self.label.clone(), LVarSet::empty());
        body._free_env_info_pass_init(info);
        rest._free_env_info_pass_init(info);
      }
      /*&LTerm::Fix(..) => {
      }*/
      &LTerm::Switch(ref pred, ref top, ref bot) => {
        info.data.insert(self.label.clone(), LVarSet::empty());
        pred._free_env_info_pass_init(info);
        top._free_env_info_pass_init(info);
        bot._free_env_info_pass_init(info);
      }
      /*&LTerm::Match(ref query, ref pat_arms) => {
      }*/
      &LTerm::UnitLit   |
      &LTerm::BitLit(_) |
      &LTerm::IntLit(_) |
      &LTerm::FloLit(_) => {
        info.data.insert(self.label.clone(), LVarSet::empty());
      }
      &LTerm::Lookup(ref var) => {
        let mut this_uses = LVarSet::empty();
        this_uses.insert(var.clone());
        info.data.insert(self.label.clone(), this_uses);
      }
      _ => unimplemented!(),
    }
  }

  pub fn _free_env_info_pass_step(&self, info: &mut LTreeFreeEnvInfo) -> bool {
    match &*self.term {
      &LTerm::Apply(ref head, ref args) => {
        let this_free_env = self._free_env(info).clone();
        let mut this_free_env_new = this_free_env.clone();
        let mut changed = head._free_env_info_pass_step(info);
        let head_free_env = head._free_env(info).clone();
        this_free_env_new = this_free_env_new.union(&head_free_env);
        for arg in args.iter() {
          changed |= arg._free_env_info_pass_step(info);
          let arg_free_env = arg._free_env(info).clone();
          this_free_env_new = this_free_env_new.union(&arg_free_env);
        }
        let this_changed = this_free_env != this_free_env_new;
        info.data.insert(self.label.clone(), this_free_env_new);
        changed | this_changed
      }
      &LTerm::Adj(ref sink) => {
        let sink_changed = sink._free_env_info_pass_step(info);
        let sink_free_env = sink._free_env(info).clone();
        let this_free_env = self._free_env(info).clone();
        let this_free_env_new = this_free_env.union(&sink_free_env);
        let this_changed = this_free_env_new != this_free_env;
        info.data.insert(self.label.clone(), this_free_env_new);
        let changed = sink_changed | this_changed;
        changed
      }
      &LTerm::Lambda(ref params, ref body) => {
        // NOTE: Equation: this.free_env U= body.free_env \ bindvars.
        let body_changed = body._free_env_info_pass_step(info);
        let bindvars = LVarSet::from_iter(params.iter());
        let body_free_env = body._free_env(info).clone();
        let nonbind_body_free_env = body_free_env.difference(&bindvars);
        let this_free_env = self._free_env(info).clone();
        let this_free_env_new = this_free_env.union(&nonbind_body_free_env);
        let this_changed = this_free_env_new != this_free_env;
        info.data.insert(self.label.clone(), this_free_env_new);
        let changed = body_changed | this_changed;
        changed
      }
      &LTerm::Let(ref name, ref body, ref rest) => {
        // NOTE: Equation: this.free_env U= body.free_env U (rest.free_env \ {name}).
        let body_changed = body._free_env_info_pass_step(info);
        let rest_changed = rest._free_env_info_pass_step(info);
        let bindvars = LVarSet::from_iter(&[name.clone()]);
        let body_free_env = body._free_env(info).clone();
        let rest_free_env = rest._free_env(info).clone();
        let nonbind_rest_free_env = rest_free_env.difference(&bindvars);
        let this_free_env = self._free_env(info).clone();
        let this_free_env_new = this_free_env.union(&body_free_env.union(&nonbind_rest_free_env));
        let this_changed = this_free_env_new != this_free_env;
        info.data.insert(self.label.clone(), this_free_env_new);
        let changed = body_changed | rest_changed | this_changed;
        changed
      }
      &LTerm::Fix(ref fixname, ref fixbody) => {
        // NOTE: Equation: this.free_env U= fixbody.free_env \ {fixname}.
        let fixbody_changed = fixbody._free_env_info_pass_step(info);
        let bindvars = LVarSet::from_iter(&[fixname.clone()]);
        let fixbody_free_env = fixbody._free_env(info).clone();
        let nonbind_fixbody_free_env = fixbody_free_env.difference(&bindvars);
        let this_free_env = self._free_env(info).clone();
        let this_free_env_new = this_free_env.union(&nonbind_fixbody_free_env);
        let this_changed = this_free_env_new != this_free_env;
        info.data.insert(self.label.clone(), this_free_env_new);
        let changed = fixbody_changed | this_changed;
        changed
      }
      &LTerm::UnitLit   |
      &LTerm::BitLit(_) |
      &LTerm::IntLit(_) |
      &LTerm::FloLit(_) |
      &LTerm::Lookup(_) => {
        false
      }
      _ => unimplemented!(),
    }
  }

  pub fn _free_env_info_pass(&self, free_env_info: &mut LTreeFreeEnvInfo) {
    self._free_env_info_pass_init(free_env_info);
    let _ = self._free_env_info_pass_step(free_env_info);
    loop {
      let changed = self._free_env_info_pass_step(free_env_info);
      if !changed {
        return;
      }
    }
  }

  pub fn _free_env<'root>(&self, info: &'root LTreeFreeEnvInfo) -> &'root LVarSet {
    match info.data.get(&self.label) {
      None => panic!(),
      Some(free_env) => free_env,
    }
  }

  pub fn free_env<'root>(&self, ltree: &'root LTree) -> &'root LVarSet {
    let root = ltree.root.clone();
    assert_eq!(self.gen, root.gen);
    let free_env_info = ltree.info.free_env.borrow_with(move || {
      let mut free_env_info = LTreeFreeEnvInfo{
        gen:  root.gen,
        data: HashMap::new(),
      };
      root._free_env_info_pass(&mut free_env_info);
      free_env_info
    });
    assert_eq!(self.gen, free_env_info.gen);
    match free_env_info.data.get(&self.label) {
      None => panic!(),
      Some(free_env) => free_env,
    }
  }

  pub fn maybe_free_env<'root>(&self, ltree: &'root LTree) -> Option<&'root LVarSet> {
    ltree.info.free_env.borrow()
      .and_then(|info| info.data.get(&self.label))
  }
}

#[derive(Debug)]
pub struct LTreeTypeckInfo {
  pub gen:  u64,
  // FIXME
  //pub data: HashMap<LLabel, ()>,
}

#[derive(Clone, Debug)]
pub struct LTreeInfo {
  pub env:      Rc<LazyCell<LTreeEnvInfo>>,
  pub free_env: Rc<LazyCell<LTreeFreeEnvInfo>>,
  //pub typeck:   Rc<LazyCell<LTreeTypeckInfo>>,
}

impl Default for LTreeInfo {
  fn default() -> LTreeInfo {
    LTreeInfo{
      env:      Rc::new(LazyCell::new()),
      free_env: Rc::new(LazyCell::new()),
      //typeck:   Rc::new(LazyCell::new()),
    }
  }
}

#[derive(Clone, Debug)]
pub struct LTree {
  pub info: LTreeInfo,
  pub root: LExpr,
}

impl LTree {
  pub fn _pretty_print(&self) {
    LTreePrettyPrinter{}.print(self.clone());
  }

  pub fn with_env_info(self) -> LTree {
    self.root.env(&self);
    self
  }

  pub fn with_free_env_info(self) -> LTree {
    self.root.free_env(&self);
    self
  }
}

#[derive(Clone, Debug)]
pub enum LDef {
  BVar,
  Code(LExpr),
  // TODO
  Fixpoint(LLabel, LVar, LTermRef, /*LEnvInfo,*/ Option<LEnv>),
}

/*#[derive(Clone, Default)]
pub struct LEnvNew {
  pub bindings: VertreapMap<LVar, (LLDepth, LHash, LDef)>,
  pub hashes:   VertreapMap<LHash, LVar>,
  pub vars:     VertreapMap<usize, LVar>,
}*/

#[derive(Clone, Default, Debug)]
pub struct LEnv {
  //pub bindings: HashMap<LVar, (usize, Rc<LExpr>)>,
  pub bindings: HashMap<LVar, (LLDepth, LDef)>,
  //pub vars:     Vec<(LVar, usize)>,
  pub vars:     Vec<LVar>,
  pub names:    HashMap<String, LVar>,
}

impl LEnv {
  pub fn _bind_bvar(&mut self, var: LVar) {
    let left_depth = self.vars.len();
    self.bindings.insert(var.clone(), (LLDepth(left_depth), LDef::BVar));
    self.vars.push(var);
  }

  pub fn _bind_free(&mut self, var: LVar, code: LExpr) {
    let left_depth = self.vars.len();
    self.bindings.insert(var.clone(), (LLDepth(left_depth), LDef::Code(code)));
    self.vars.push(var);
  }

  pub fn _bind_fixed(&mut self, label: LLabel, fixvar: LVar, fixbody: LTermRef, env: Option<LEnv>) {
    let left_depth = self.vars.len();
    self.bindings.insert(fixvar.clone(), (LLDepth(left_depth), LDef::Fixpoint(label, fixvar.clone(), fixbody, env)));
    self.vars.push(fixvar);
  }

  pub fn _bind_named(&mut self, name: String, var: LVar, body: LExpr) {
    let left_depth = self.vars.len();
    self.bindings.insert(var.clone(), (LLDepth(left_depth), LDef::Code(body)));
    self.names.insert(name, var.clone());
    self.vars.push(var);
  }

  /*pub fn _lookup_name(&self, name: String) -> (LVar, LExpr) {
    match self.names.get(&name) {
      Some(var) => {
        match self.bindings.get(var) {
          Some(&(_, LDef::BVar)) => {
            // FIXME
            /*let code = LExpr{
              gen:      0,
              label:    label.clone(),
              term:     LTermRef::new(LTerm::Lookup(var.clone())),
              info:     LExprInfo::default(),
            };
            (var.clone(), code.clone())*/
            unimplemented!();
          }
          Some(&(_, LDef::Code(ref code))) => {
            (var.clone(), code.clone())
          }
          Some(&(_, LDef::Fixpoint(ref label, ref fixvar, ref fixbody, ref maybe_env))) => {
            // TODO: in the case of static env info, need to make sure that
            // the returned `LExpr` knows about itself... hence the fixpoint!
            // FIXME: initializing an empty expr info seems wrong.
            let mut info = LExprInfo::default();
            if let &Some(ref env) = maybe_env {
              let fix_env = env.fork_fixed(label.clone(), fixvar.clone(), fixbody.clone(), maybe_env.clone());
              info.env = Some(fix_env.clone());
            }
            let code = LExpr{
              // FIXME: correct gen number.
              gen:      0,
              label:    label.clone(),
              term:     fixbody.clone(),
              info,
            };
            (var.clone(), code)
          }
          None => panic!(),
        }
      }
      None => panic!(),
    }
  }*/

  pub fn fork_bvar(&self, var: LVar) -> LEnv {
    let mut new_env = self.clone();
    new_env._bind_bvar(var);
    new_env
  }

  pub fn fork(&self, /*hash: LHash,*/ var: LVar, body: LExpr) -> LEnv {
    let mut new_env = self.clone();
    new_env._bind_free(var, body);
    new_env
  }

  pub fn fork_fixed(&self, label: LLabel, /*hash: LHash,*/ fixname: LVar, fixbody: LTermRef, env: Option<LEnv>) -> LEnv {
    let mut new_env = self.clone();
    new_env._bind_fixed(label, fixname, fixbody, env);
    new_env
  }
}

/*pub enum LVarKey {
  Fresh,
  Name(String),
  IntLit(i64),
  FloLit(f64),
}*/

#[derive(Debug)]
pub enum LBindKey {
  Fresh,
  Name(String),
  Hash(LHash),
  //IntLit(i64),
  //FloLit(f64),
}

#[derive(Default)]
struct LVarBuilder {
  //n_to_id:  HashMap<String, LVar>,
  h_to_id:  HashMap<LHash, LVar>,
  id_to_k:  HashMap<LVar, LBindKey>,
  id_ctr:   u64,
}

impl LVarBuilder {
  pub fn _debug_dump_vars(&self) {
    println!("DEBUG: vars: {:?}", self.id_to_k);
  }

  //pub fn rev_lookup(&mut self, var: LVar) -> String {
  pub fn rev_lookup(&mut self, var: LVar) -> LHash {
    match self.id_to_k.get(&var) {
      //Some(LBindKey::Name(ref name)) => {
      Some(LBindKey::Hash(ref hash)) => {
        //name.clone()
        hash.clone()
      }
      Some(_) => panic!(),
      None => panic!(),
    }
  }

  //pub fn lookup(&mut self, name: String) -> LVar {
  pub fn lookup(&mut self, hash: LHash) -> LVar {
    match self.h_to_id.get(&hash) {
      Some(var) => var.clone(),
      None => panic!(),
    }
  }

  /*pub fn insert(&mut self, name: String) {
    let _ = self.bind(name);
  }*/

  pub fn fresh(&mut self) -> LVar {
    let &mut LVarBuilder{ref mut id_ctr, ..} = self;
    *id_ctr += 1;
    let id = *id_ctr;
    assert!(id != 0);
    let new_var = LVar(id);
    new_var
  }

  //pub fn bind(&mut self, name: String) -> (String, LVar, Option<LVar>) {
  pub fn bind(&mut self, hash: LHash) -> (LHash, LVar, Option<LVar>) {
    let &mut LVarBuilder{
      //ref mut n_to_id,
      ref mut h_to_id,
      ref mut id_to_k,
      ref mut id_ctr} = self;
    *id_ctr += 1;
    let id = *id_ctr;
    assert!(id != 0);
    let new_var = LVar(id);
    //let old_var = n_to_id.insert(name.clone(), new_var.clone());
    let old_var = h_to_id.insert(hash.clone(), new_var.clone());
    //id_to_k.insert(new_var.clone(), LBindKey::Name(name.clone()));
    id_to_k.insert(new_var.clone(), LBindKey::Hash(hash.clone()));
    //(name, new_var, old_var)
    (hash, new_var, old_var)
  }

  //pub fn unbind(&mut self, name: String, new_var: LVar, old_var: Option<LVar>) {
  pub fn unbind(&mut self, hash: LHash, new_var: LVar, old_var: Option<LVar>) {
    let pop_var = if let Some(old_var) = old_var {
      //self.n_to_id.insert(name, old_var)
      self.h_to_id.insert(hash, old_var)
    } else {
      //self.n_to_id.remove(&name)
      self.h_to_id.remove(&hash)
    };
    match pop_var {
      Some(pop_var) => assert_eq!(pop_var, new_var),
      None => panic!(),
    }
  }
}

#[derive(Default)]
struct LHashBuilder {
  n_to_id:  HashMap<String, LHash>,
  id_to_n:  HashMap<LHash, String>,
  id_ctr:   u64,
}

impl LHashBuilder {
  pub fn rev_lookup(&self, hash: LHash) -> String {
    match self.id_to_n.get(&hash) {
      Some(name) => name.clone(),
      None => panic!(),
    }
  }

  pub fn lookup(&mut self, name: String) -> LHash {
    let &mut LHashBuilder{
      ref mut n_to_id,
      ref mut id_to_n,
      ref mut id_ctr} = self;
    n_to_id.entry(name.clone()).or_insert_with(|| {
      *id_ctr += 1;
      let id = *id_ctr;
      assert!(id != 0);
      let new_hash = LHash(id);
      id_to_n.insert(new_hash.clone(), name);
      new_hash
    }).clone()
  }
}

#[derive(Default)]
struct LLabelBuilder {
  id_ctr:   u64,
}

impl LLabelBuilder {
  pub fn fresh(&mut self) -> LLabel {
    self.id_ctr += 1;
    let id = self.id_ctr;
    assert!(id != 0);
    LLabel(id)
  }
}

pub struct LBuilder {
  gen_ctr:  u64,
  labels:   LLabelBuilder,
  hashes:   LHashBuilder,
  vars:     LVarBuilder,
}

impl LBuilder {
  pub fn new() -> LBuilder {
    let labels = LLabelBuilder::default();
    let hashes = LHashBuilder::default();
    let vars = LVarBuilder::default();
    LBuilder{
      gen_ctr: 0,
      labels,
      hashes,
      vars,
    }
  }

  pub fn _debug_dump_vars(&self) {
    self.vars._debug_dump_vars();
  }

  pub fn _alloc_name(&mut self, name: &str) -> (LHash, LVar, LLabel) {
    let hash = self.hashes.lookup(name.to_string());
    let (_, var, _) = self.vars.bind(hash.clone());
    //println!("DEBUG: LBuilder: insert: {:?} {:?} {:?}", name, hash, var);
    let label = self.labels.fresh();
    (hash, var, label)
  }

  pub fn _gen(&self) -> u64 {
    self.gen_ctr
  }

  pub fn _make_bclambda(&mut self, bc: VMBoxCode) -> LExpr {
    LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::VMExt(LTermVMExt::BcLambda(vec![], bc))), info: LExprInfo::default()}
  }

  pub fn _alloc_bclambda(&mut self, name: &str, bc: VMBoxCode) -> (LHash, LVar, LLabel, LExpr) {
    let (hash, var, label) = self._alloc_name(name);
    let bclam = self._make_bclambda(bc);
    (hash, var, label, bclam)
  }

  pub fn _include_stdlib_and_lower(&mut self, htree: Rc<HExpr>) -> LExpr {
    fn bind(gen: u64, label: LLabel, var: LVar, body: LExpr, rest: LExpr) -> LExpr {
      LExpr{gen, label: label, term: LTermRef::new(LTerm::Let(var, body, rest)), info: LExprInfo::default()}
    }
    fn bind_next<B: Iterator<Item=(&'static str, Box<dyn Fn() -> VMBoxCode>)>, R: FnOnce() -> Rc<HExpr>>(this: &mut LBuilder, mut bindings: B, rest: R) -> LExpr {
      match bindings.next() {
        None => this._htree_to_ltree_lower_pass((rest)()),
        Some((name, bc_builder)) => {
          let (_, var, label, bc) = this._alloc_bclambda(name, bc_builder());
          let ltree = (bind_next)(this, bindings, rest);
          let ltree = (bind)(this._gen(), label, var, bc, ltree);
          ltree
        }
      }
    }
    let stdlib_bindings: Vec<(&'static str, Box<dyn Fn() -> VMBoxCode>)> = vec![
      ("eq",  Box::new(|| make_bc_eq())),
      ("add", Box::new(|| make_bc_add())),
      ("sub", Box::new(|| make_bc_sub())),
      ("mul", Box::new(|| make_bc_mul())),
      ("div", Box::new(|| make_bc_div())),
      ("neg", Box::new(|| make_bc_neg())),
      ("pi100", Box::new(|| make_bc_pi100())),
    ];
    let ltree = bind_next(
        self,
        stdlib_bindings.into_iter(),
        move || htree
    );
    ltree
  }

  pub fn lower_with_stdlib(&mut self, htree: Rc<HExpr>) -> LTree {
    self.gen_ctr += 1;
    let ltree = self._include_stdlib_and_lower(htree);
    LTree{
      info: LTreeInfo::default(),
      root: ltree,
    }
  }

  pub fn _htree_to_ltree_lower_pass_pat(&mut self, htree: Rc<HExpr>) -> LPat {
    // FIXME
    unimplemented!();
  }

  pub fn _htree_to_ltree_lower_pass(&mut self, htree: Rc<HExpr>) -> LExpr {
    // TODO
    match &*htree {
      &HExpr::Lambda(ref args, ref body) => {
        // TODO
        let bvars: Vec<_> = args.iter().map(|arg| {
          let a = self._htree_to_ltree_lower_pass(arg.clone());
          match &*a.term {
            // FIXME: allow wildcard pattern in param position.
            &LTerm::Lookup(ref v) => v.clone(),
            _ => panic!(),
          }
        }).collect();
        let body = self._htree_to_ltree_lower_pass(body.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Lambda(bvars, body)), info: LExprInfo::default()}
      }
      &HExpr::Apply(ref lhs, ref args) => {
        let lhs = self._htree_to_ltree_lower_pass(lhs.clone());
        let args: Vec<_> = args.iter().map(|arg| {
          self._htree_to_ltree_lower_pass(arg.clone())
        }).collect();
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Apply(lhs, args)), info: LExprInfo::default()}
      }
      &HExpr::Tuple(ref elems) => {
        let elems: Vec<_> = elems.iter().map(|arg| {
          self._htree_to_ltree_lower_pass(arg.clone())
        }).collect();
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Tuple(elems)), info: LExprInfo::default()}
      }
      &HExpr::Adj(ref sink) => {
        let sink = self._htree_to_ltree_lower_pass(sink.clone());
        /*let sink_var = match &*sink.term {
          &LTerm::Lookup(ref v) => v.clone(),
          _ => panic!(),
        };*/
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Adj(sink)), info: LExprInfo::default()}
      }
      &HExpr::AdjDyn(ref body) => {
        // TODO
        let body = self._htree_to_ltree_lower_pass(body.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::AdjDyn(body)), info: LExprInfo::default()}
      }
      &HExpr::Let(ref lhs, ref body, ref rest, ref maybe_attrs) => {
        let attrs = maybe_attrs.clone().unwrap_or_default();
        match &**lhs {
          &HExpr::Ident(ref lhs) => {
            // FIXME: desugar "let rec" to "let fix in".
            match attrs.rec {
              false => {
                let lhs_hash = self.hashes.lookup(lhs.to_string());
                let (lhs_hash, lhs_var, lhs_oldvar) = self.vars.bind(lhs_hash);
                let body = self._htree_to_ltree_lower_pass(body.clone());
                let rest = self._htree_to_ltree_lower_pass(rest.clone());
                self.vars.unbind(lhs_hash, lhs_var.clone(), lhs_oldvar);
                LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Let(lhs_var, body, rest)), info: LExprInfo::default()}
              }
              true  => {
                let lhs_hash = self.hashes.lookup(lhs.to_string());
                let (lhs_hash, lhs_var, lhs_oldvar) = self.vars.bind(lhs_hash);
                let (fixname_hash, fixname_var, fixname_oldvar) = self.vars.bind(lhs_hash.clone());
                let fixbody = self._htree_to_ltree_lower_pass(body.clone());
                let fix = LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Fix(fixname_var.clone(), fixbody)), info: LExprInfo::default()};
                self.vars.unbind(fixname_hash, fixname_var, fixname_oldvar);
                let rest = self._htree_to_ltree_lower_pass(rest.clone());
                self.vars.unbind(lhs_hash, lhs_var.clone(), lhs_oldvar);
                LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Let(lhs_var, fix, rest)), info: LExprInfo::default()}
              }
            }
          }
          &HExpr::Apply(ref lhs, ref args) => {
            // FIXME: desugar "let rec" to "let fix in".
            let lhs_hash = match &**lhs {
              &HExpr::Ident(ref name) => {
                self.hashes.lookup(name.to_string())
              }
              _ => panic!(),
            };
            let (name, name_var, name_oldvar) = self.vars.bind(lhs_hash);
            let mut args_ = vec![];
            let mut args_vars = vec![];
            let mut args_oldvars = vec![];
            for arg in args.iter() {
              let (arg_, arg_var, arg_oldvar) = match &**arg {
                &HExpr::Ident(ref name) => {
                  let arg_hash = self.hashes.lookup(name.to_string());
                  self.vars.bind(arg_hash)
                }
                _ => panic!(),
              };
              args_.push(arg_);
              args_vars.push(arg_var);
              args_oldvars.push(arg_oldvar);
            }
            let body = self._htree_to_ltree_lower_pass(body.clone());
            for ((arg_, arg_var), arg_oldvar) in args_.iter().zip(args_vars.iter()).zip(args_oldvars.iter()).rev() {
              self.vars.unbind(arg_.clone(), arg_var.clone(), arg_oldvar.clone());
            }
            let lam = LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Lambda(args_vars, body)), info: LExprInfo::default()};
            let rest = self._htree_to_ltree_lower_pass(rest.clone());
            self.vars.unbind(name, name_var.clone(), name_oldvar);
            LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Let(name_var, lam, rest)), info: LExprInfo::default()}
          }
          _ => {
            panic!();
          }
        }
      }
      &HExpr::Switch(ref pred, ref top, ref bot) => {
        let pred = self._htree_to_ltree_lower_pass(pred.clone());
        let top = self._htree_to_ltree_lower_pass(top.clone());
        let bot = self._htree_to_ltree_lower_pass(bot.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Switch(pred, top, bot)), info: LExprInfo::default()}
      }
      &HExpr::Eq(ref lhs, ref rhs) => {
        let op_name = "eq".to_string();
        let op_hash = self.hashes.lookup(op_name);
        let op_var = self.vars.lookup(op_hash);
        let op = LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Lookup(op_var)), info: LExprInfo::default()};
        let lhs = self._htree_to_ltree_lower_pass(lhs.clone());
        let rhs = self._htree_to_ltree_lower_pass(rhs.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Apply(op, vec![lhs, rhs])), info: LExprInfo::default()}
      }
      &HExpr::Add(ref lhs, ref rhs) => {
        let op_name = "add".to_string();
        let op_hash = self.hashes.lookup(op_name);
        let op_var = self.vars.lookup(op_hash);
        let op = LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Lookup(op_var)), info: LExprInfo::default()};
        let lhs = self._htree_to_ltree_lower_pass(lhs.clone());
        let rhs = self._htree_to_ltree_lower_pass(rhs.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Apply(op, vec![lhs, rhs])), info: LExprInfo::default()}
      }
      &HExpr::Sub(ref lhs, ref rhs) => {
        let op_name = "sub".to_string();
        let op_hash = self.hashes.lookup(op_name);
        let op_var = self.vars.lookup(op_hash);
        let op = LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Lookup(op_var)), info: LExprInfo::default()};
        let lhs = self._htree_to_ltree_lower_pass(lhs.clone());
        let rhs = self._htree_to_ltree_lower_pass(rhs.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Apply(op, vec![lhs, rhs])), info: LExprInfo::default()}
      }
      &HExpr::Mul(ref lhs, ref rhs) => {
        let op_name = "mul".to_string();
        let op_hash = self.hashes.lookup(op_name);
        let op_var = self.vars.lookup(op_hash);
        let op = LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Lookup(op_var)), info: LExprInfo::default()};
        let lhs = self._htree_to_ltree_lower_pass(lhs.clone());
        let rhs = self._htree_to_ltree_lower_pass(rhs.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Apply(op, vec![lhs, rhs])), info: LExprInfo::default()}
      }
      &HExpr::Div(ref lhs, ref rhs) => {
        let op_name = "div".to_string();
        let op_hash = self.hashes.lookup(op_name);
        let op_var = self.vars.lookup(op_hash);
        let op = LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Lookup(op_var)), info: LExprInfo::default()};
        let lhs = self._htree_to_ltree_lower_pass(lhs.clone());
        let rhs = self._htree_to_ltree_lower_pass(rhs.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Apply(op, vec![lhs, rhs])), info: LExprInfo::default()}
      }
      &HExpr::Neg(ref arg) => {
        let op_name = "neg".to_string();
        let op_hash = self.hashes.lookup(op_name);
        let op_var = self.vars.lookup(op_hash);
        let op = LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Lookup(op_var)), info: LExprInfo::default()};
        let arg = self._htree_to_ltree_lower_pass(arg.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Apply(op, vec![arg])), info: LExprInfo::default()}
      }
      &HExpr::NoRet => {
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::NoRet), info: LExprInfo::default()}
      }
      &HExpr::NonSmooth => {
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::NonSmooth), info: LExprInfo::default()}
      }
      &HExpr::UnitLit => {
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::UnitLit), info: LExprInfo::default()}
      }
      &HExpr::NilTupLit => {
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Tuple(vec![])), info: LExprInfo::default()}
      }
      &HExpr::BotLit => {
        // TODO: special var key for literal constants.
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::BitLit(false)), info: LExprInfo::default()}
      }
      &HExpr::TeeLit => {
        // TODO: special var key for literal constants.
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::BitLit(true)), info: LExprInfo::default()}
      }
      &HExpr::IntLit(x) => {
        // TODO: special var key for literal constants.
        //let var = self.vars.int_lit(x);
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::IntLit(x)), info: LExprInfo::default()}
      }
      &HExpr::Ident(ref name) => {
        let name_hash = self.hashes.lookup(name.to_string());
        let name_var = self.vars.lookup(name_hash);
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::Lookup(name_var)), info: LExprInfo::default()}
      }
      &HExpr::PathLookup(ref lhs, ref rhs_name) => {
        /*let (name, name_var, name_oldvar) = self.vars.bind(name.to_string());
        //let body = self._htree_to_ltree_lower_pass(body.clone());
        let env = LExpr{label: self.labels.fresh(), term: LTermRef::new(LTerm::Env), info: LExprInfo::default()};
        //let rest = self._htree_to_ltree_lower_pass(rest.clone());
        self.vars.unbind(name, name_var.clone(), name_oldvar);
        LExpr{label: self.labels.fresh(), term: LTermRef::new(LTerm::Let(name_var, body, rest)), info: LExprInfo::default()}*/
        // TODO
        let lhs = self._htree_to_ltree_lower_pass(lhs.clone());
        //LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::DynEnvLookup(lhs, rhs_name.clone())), info: LExprInfo::default()}
        let rhs_hash = self.hashes.lookup(rhs_name.clone());
        LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(LTerm::TmpEnvLookup(lhs, rhs_hash)), info: LExprInfo::default()}
      }
      _ => {
        // TODO
        unimplemented!();
      }
    }
  }

  pub fn lower(&mut self, htree: Rc<HExpr>) -> LTree {
    self.gen_ctr += 1;
    let ltree = self._htree_to_ltree_lower_pass(htree);
    LTree{
      info: LTreeInfo::default(),
      root: ltree,
    }
  }

  pub fn _ltree_normalize_pass_kont(&mut self, ltree: LExpr, kont: &mut dyn FnMut(&mut Self, LExpr) -> LExpr) -> LExpr {
    // TODO
    match &*ltree.term {
      &LTerm::Apply(ref head, ref args) => {
        // FIXME: normalize the head.
        // FIXME: normalize non-1 args.
        let saved_head = head.clone();
        match args.len() {
          1 => {
            self._ltree_normalize_pass_kont_name(args[0].clone(), &mut |this, e| {
              let new_apply_expr = LExpr{
                gen:    this._gen(),
                label:  this.labels.fresh(),
                term:   LTermRef::new(LTerm::Apply(
                    saved_head.clone(),
                    vec![e],
                )),
                info:   LExprInfo::default(),
              };
              kont(this, new_apply_expr)
            })
          }
          _ => unimplemented!(),
        }
      }
      &LTerm::Adj(ref sink) => {
        self._ltree_normalize_pass_kont_name(sink.clone(), &mut |this, sink| {
          let new_adj_expr = LExpr{
            gen:    this._gen(),
            label:  this.labels.fresh(),
            term:   LTermRef::new(LTerm::Adj(sink)),
            info:   LExprInfo::default(),
          };
          kont(this, new_adj_expr)
        })
      }
      &LTerm::Lambda(ref bvars, ref body) => {
        let new_lambda_expr = LExpr{
          gen:    self._gen(),
          label:  self.labels.fresh(),
          term:   LTermRef::new(LTerm::Lambda(
              bvars.clone(),
              self._ltree_normalize_pass_kont_term(body.clone()),
          )),
          info:   LExprInfo::default(),
        };
        kont(self, new_lambda_expr)
      }
      &LTerm::VMExt(LTermVMExt::BcLambda(..)) => {
        kont(self, ltree)
      }
      &LTerm::Let(ref name, ref body, ref rest) => {
        let saved_name = name.clone();
        let saved_rest = rest.clone();
        self._ltree_normalize_pass_kont(body.clone(), &mut |this, e| {
          LExpr{
            gen:    this._gen(),
            label:  this.labels.fresh(),
            term:   LTermRef::new(LTerm::Let(
                saved_name.clone(),
                e,
                this._ltree_normalize_pass_kont(saved_rest.clone(), kont),
            )),
            info:   LExprInfo::default(),
        })
      }
      &LTerm::Switch(ref pred, ref top, ref bot) => {
        let saved_top = top.clone();
        let saved_bot = bot.clone();
        self._ltree_normalize_pass_kont_name(pred.clone(), &mut |this, e| {
          let new_top = this._ltree_normalize_pass_kont_term(saved_top.clone());
          let new_bot = this._ltree_normalize_pass_kont_term(saved_bot.clone());
          let new_switch_expr = LExpr{
            gen:    this._gen(),
            label:  this.labels.fresh(),
            term:   LTermRef::new(LTerm::Switch(
                e,
                new_top,
                new_bot,
            )),
            info:   LExprInfo::default(),
          };
          kont(this, new_switch_expr)
        })
      }
      &LTerm::Tuple(ref args) => {
        // FIXME: should really use `kont_name` below; solution to this is same
        // as for normalizing lambda applications.
        let new_tuple_expr = LExpr{
          gen:    self._gen(),
          label:  self.labels.fresh(),
          term:   LTermRef::new(LTerm::Tuple(
              args.iter().map(|arg| self._ltree_normalize_pass_kont_term(arg.clone())).collect(),
          )),
          info:   LExprInfo::default(),
        };
        kont(self, new_tuple_expr)
      }
      &LTerm::BitLit(_) => {
        kont(self, ltree)
      }
      &LTerm::IntLit(_) => {
        kont(self, ltree)
      }
      &LTerm::FloLit(_) => {
        kont(self, ltree)
      }
      &LTerm::Lookup(_) => {
        kont(self, ltree)
      }
      e => panic!("normalize: unimplemented lexpr term: {:?}", e),
    }
  }

  pub fn _ltree_normalize_pass_kont_term(&mut self, ltree: LExpr) -> LExpr {
    self._ltree_normalize_pass_kont(ltree, &mut |this, new_ltree| new_ltree)
  }

  /*pub fn _ltree_normalize_pass_kont_names(&mut self, mut ltrees: Vec<LExpr>, kont: &mut dyn FnMut(&mut Self, Vec<LExpr>) -> LExpr) -> LExpr {
    if ltrees.len() == 0 {
      vec![]
    } else if ltrees.len() == 1 {
      let ltree = ltrees.pop().unwrap();
      vec![self._ltree_normalize_pass_kont_name(ltree, &mut |this, e| e)]
    } else {
      unimplemented!();
    }
  }*/

  pub fn _ltree_normalize_pass_kont_name(&mut self, ltree: LExpr, kont: &mut dyn FnMut(&mut Self, LExpr) -> LExpr) -> LExpr {
    self._ltree_normalize_pass_kont(ltree, &mut |this, new_ltree| {
      match &*new_ltree.term {
        &LTerm::BitLit(_) => {
          kont(this, new_ltree)
        }
        &LTerm::IntLit(_) => {
          kont(this, new_ltree)
        }
        &LTerm::FloLit(_) => {
          kont(this, new_ltree)
        }
        &LTerm::Lookup(_) => {
          kont(this, new_ltree)
        }
        &LTerm::Tuple(ref elems) => {
          if elems.is_empty() {
            kont(this, new_ltree)
          } else {
            let new_var = this.vars.fresh();
            LExpr{
              gen:    this._gen(),
              label:  this.labels.fresh(),
              term:   LTermRef::new(LTerm::Let(
                  new_var.clone(),
                  new_ltree,
                  {
                    let new_var_expr = LExpr{
                      gen:    this._gen(),
                      label:  this.labels.fresh(),
                      term:   LTermRef::new(LTerm::Lookup(new_var)),
                      info:   LExprInfo::default(),
                    };
                    kont(this, new_var_expr)
                  }
              )),
              info:   LExprInfo::default(),
            }
          }
        }
        _ => {
          let new_var = this.vars.fresh();
          LExpr{
            gen:    this._gen(),
            label:  this.labels.fresh(),
            term:   LTermRef::new(LTerm::Let(
                new_var.clone(),
                new_ltree,
                {
                  let new_var_expr = LExpr{
                    gen:    this._gen(),
                    label:  this.labels.fresh(),
                    term:   LTermRef::new(LTerm::Lookup(new_var)),
                    info:   LExprInfo::default(),
                  };
                  kont(this, new_var_expr)
                }
            )),
            info:   LExprInfo::default(),
          }
        }
      }
    })
  }

  pub fn normalize(&mut self, ltree: LTree) -> LTree {
    self.gen_ctr += 1;
    let root = self._ltree_normalize_pass_kont_term(ltree.root);
    LTree{
      info: LTreeInfo::default(),
      root,
    }
  }

  pub fn _ltree_dyn_adjoint_pass(&mut self, ltree: LExpr) -> LExpr {
    match &*ltree.term {
      &LTerm::Let(ref name, ref body, ref rest) => {
        let body = self._ltree_dyn_adjoint_pass(body.clone());
        let rest = self._ltree_dyn_adjoint_pass(rest.clone());
        LExpr{
          gen:      self._gen(),
          label:    ltree.label.clone(),
          term:     LTermRef::new(LTerm::Let(
              name.clone(),
              body,
              rest,
          )),
          info:     ltree.info,
        }
      }
      &LTerm::IntLit(x) => {
        LExpr{
          gen:      self._gen(),
          label:    ltree.label.clone(),
          term:     LTermRef::new(LTerm::IntLit(x)),
          info:     ltree.info,
        }
      }
      &LTerm::Lookup(ref lookup_var) => {
        LExpr{
          gen:      self._gen(),
          label:    ltree.label.clone(),
          term:     LTermRef::new(LTerm::Lookup(lookup_var.clone())),
          info:     ltree.info,
        }
      }
      &LTerm::Adj(ref body) => {
        // TODO
        panic!();
        /*let orig_env = match ltree.info.env {
          Some(ref env) => env.clone(),
          None => panic!(),
        };
        let mut shadow_env = orig_env.clone();
        let mut shadow = vec![];
        // FIXME: only need to shadow freevars, not all bindings in scope.
        for (fvar_var, _) in orig_env.bindings.iter() {
          let fvar_name = self.vars.rev_lookup(fvar_var.clone());
          let (shadow_hash, shadow_var, shadow_oldvar) = self.vars.bind(fvar_name);
          let shadow_name = self.hashes.rev_lookup(shadow_hash.clone());
          // FIXME: next line is where we bind the adjoint expr;
          // for now, we set the adjoint to be any old literal.
          let shadow_code = LExpr{label: self.labels.fresh(), term: LTermRef::new(LTerm::IntLit(42424242)), info: LExprInfo::default()};
          shadow_env._bind_named(shadow_name.clone(), shadow_var.clone(), shadow_code);
          shadow.push((shadow_hash, shadow_var, shadow_oldvar));
        }
        for (shadow_hash, shadow_var, shadow_oldvar) in shadow.into_iter().rev() {
          self.vars.unbind(shadow_hash, shadow_var, shadow_oldvar);
        }
        let mut info = LExprInfo::default();
        info.env = Some(shadow_env);
        LExpr{
          label:    self.labels.fresh(),
          term:     LTermRef::new(LTerm::Env),
          info,
        }*/
      }
      &LTerm::AdjDyn(ref root) => {
        // TODO
        let root = self._ltree_dyn_adjoint_pass(root.clone());
        let root_env = match root.info.env {
          Some(ref env) => env.clone(),
          None => panic!(),
        };
        let mut shadow_env = root_env.clone();
        let mut shadow = vec![];
        // FIXME: do the correct traversal order (requires freevars).
        // FIXME: only need to shadow freevars, not all bindings in scope.
        for (fvar_var, &(_, ref fvar_def)) in root_env.bindings.iter() {
          let fvar_hash = self.vars.rev_lookup(fvar_var.clone());
          let (fvar_hash, shadow_var, shadow_oldvar) = self.vars.bind(fvar_hash);
          let fvar_name = self.hashes.rev_lookup(fvar_hash.clone());
          // FIXME: next line is where we bind the adjoint expr;
          // for now, we set the adjoint to be any old literal.
          let adj_term = match &*root.term {
            &LTerm::Lookup(ref root_var) => {
              if root_var == fvar_var {
                Some(LTerm::IntLit(1))
              } else {
                None
              }
            }
            _ => {
              // FIXME: if the IR is properly normalized, root should always be
              // a var.
              println!("WARNING: adj dyn: root is not a var");
              None
            }
          }.unwrap_or({
            /*match fvar_def {
              &LDef::Code(ref ltree) => {
                // TODO
                //unimplemented!();
              }
            }*/
            LTerm::IntLit(42424242)
          });
          let shadow_code = LExpr{gen: self._gen(), label: self.labels.fresh(), term: LTermRef::new(adj_term), info: LExprInfo::default()};
          shadow_env._bind_named(fvar_name, shadow_var.clone(), shadow_code);
          shadow.push((fvar_hash, shadow_var, shadow_oldvar));
        }
        for (fvar_hash, shadow_var, shadow_oldvar) in shadow.into_iter().rev() {
          self.vars.unbind(fvar_hash, shadow_var, shadow_oldvar);
        }
        LExpr{
          gen:      self._gen(),
          label:    ltree.label.clone(),
          term:     LTermRef::new(LTerm::DynEnv(shadow_env)),
          info:     ltree.info,
        }
      }
      &LTerm::DynEnvLookup(ref target, ref name) => {
        // TODO
        let target = self._ltree_dyn_adjoint_pass(target.clone());
        LExpr{
          gen:      self._gen(),
          label:    ltree.label.clone(),
          term:     LTermRef::new(LTerm::DynEnvLookup(target, name.clone())),
          info:     ltree.info,
        }
      }
      _ => unimplemented!(),
    }
  }

  pub fn _ltree_strip_info_pass(&mut self, ltree: LExpr) -> LExpr {
    // TODO
    unimplemented!();
  }

  pub fn _ltree_adjoint_transform_rec(&mut self, sink: LVar, ltree: LExpr) -> LExpr {
    // TODO
    match &*ltree.term {
      &LTerm::Apply(ref head, ref args) => {
        // TODO
        unimplemented!();
      }
      &LTerm::Lambda(ref bvars, ref body) => {
        // TODO
        let body = self._ltree_adjoint_transform_rec(sink.clone(), body.clone());
        unimplemented!();
        LExpr{
          gen:      self._gen(),
          label:    ltree.label,
          term:     LTermRef::new(LTerm::Lambda(
              bvars.clone(),
              body,
          )),
          info:     ltree.info,
        }
      }
      &LTerm::Let(ref name, ref body, ref rest) => {
        // TODO
        let body = self._ltree_adjoint_transform_rec(sink.clone(), body.clone());
        let rest = self._ltree_adjoint_transform_rec(sink.clone(), rest.clone());
        unimplemented!();
        LExpr{
          gen:      self._gen(),
          label:    ltree.label,
          term:     LTermRef::new(LTerm::Let(
              name.clone(),
              body,
              rest,
          )),
          info:     ltree.info,
        }
      }
      &LTerm::Fix(ref fixname, ref fixbody) => {
        // TODO
        let fixbody = self._ltree_adjoint_transform_rec(sink.clone(), fixbody.clone());
        unimplemented!();
        LExpr{
          gen:      self._gen(),
          label:    ltree.label,
          term:     LTermRef::new(LTerm::Fix(
              fixname.clone(),
              fixbody,
          )),
          info:     ltree.info,
        }
      }
      &LTerm::IntLit(_) => {
        LExpr{
          gen:      self._gen(),
          label:    ltree.label,
          term:     LTermRef::new(LTerm::IntLit(0)),
          info:     ltree.info,
        }
      }
      &LTerm::Lookup(ref lookup_var) => {
        // TODO
        unimplemented!();
        LExpr{
          gen:      self._gen(),
          label:    ltree.label,
          term:     LTermRef::new(LTerm::Lookup(lookup_var.clone())),
          info:     ltree.info,
        }
      }
      &LTerm::Adj(_) => {
        // TODO
        unimplemented!();
      }
      _ => unimplemented!(),
    }
  }

  pub fn _ltree_adjoint_transform_rooted(&mut self, lroot: LExpr, ltree: LExpr) -> LExpr {
    // TODO
    match &*ltree.term {
      &LTerm::Adj(_) => {
        // TODO
        unimplemented!();
        /*self._ltree_adjoint_transform_rec(_, lroot);*/
      }
      _ => unimplemented!(),
    }
  }

  pub fn _ltree_adjoint_transform(&mut self, ltree: LExpr) -> LExpr {
    self._ltree_adjoint_transform_rooted(ltree.clone(), ltree)
  }
}

pub enum LPrettyPrinterVerbosity {
  V0,
  V1,
  V2,
  V3,
}

pub struct LPrettyPrinter {
  //v: LPrettyPrinterVerbosity,
}

impl LPrettyPrinter {
  fn _write<W: Write>(&mut self, ltree: LExpr, indent: usize, indent_first: bool, writer: &mut W) -> bool {
    // TODO
    if indent_first {
      for _ in 0 .. indent {
        write!(writer, " ").unwrap();
      }
    }
    match &*ltree.term {
      &LTerm::Apply(ref head, ref args) => {
        // TODO
        let apply_toks = "apply ";
        write!(writer, "{}", apply_toks).unwrap();
        self._write(head.clone(), indent + apply_toks.len() + 2, false, writer);
        writeln!(writer, "").unwrap();
        for arg in args.iter() {
          for _ in 0 .. indent + apply_toks.len() {
            write!(writer, " ").unwrap();
          }
          self._write(arg.clone(), indent + apply_toks.len() + 2, false, writer);
          writeln!(writer, "").unwrap();
        }
        false
      }
      &LTerm::Adj(ref sink) => {
        if let Some(ref freeuses) = ltree.info.freeuses {
          write!(writer, "# INFO: freeuses: [").unwrap();
          let mut first_it = true;
          for v in freeuses.inner.ones() {
            if first_it {
              write!(writer, "${}", v).unwrap();
              first_it = false;
            } else {
              write!(writer, ", ${}", v).unwrap();
            }
          }
          writeln!(writer, "]").unwrap();
          for _ in 0 .. indent {
            write!(writer, " ").unwrap();
          }
        }
        let adj_toks = format!("adj ");
        write!(writer, "{}", adj_toks).unwrap();
        let sink_indent = indent + adj_toks.len();
        self._write(sink.clone(), sink_indent, false, writer)
      }
      &LTerm::Lambda(ref params, ref body) => {
        // FIXME
        write!(writer, "\\").unwrap();
        for (param_idx, param) in params.iter().enumerate() {
          if param_idx == 0 {
            write!(writer, "${}", param.0).unwrap();
          } else {
            write!(writer, ", ${}", param.0).unwrap();
          }
        }
        write!(writer, ". <lam.body>").unwrap();
        false
      }
      &LTerm::VMExt(LTermVMExt::BcLambda(..)) => {
        // FIXME
        write!(writer, "<bclam>").unwrap();
        false
      }
      &LTerm::Let(ref name, ref body, ref rest) => {
        // TODO
        if let Some(ref freeuses) = ltree.info.freeuses {
          write!(writer, "# INFO: freeuses: [").unwrap();
          let mut first_it = true;
          for v in freeuses.inner.ones() {
            if first_it {
              write!(writer, "${}", v).unwrap();
              first_it = false;
            } else {
              write!(writer, ", ${}", v).unwrap();
            }
          }
          writeln!(writer, "]").unwrap();
          for _ in 0 .. indent {
            write!(writer, " ").unwrap();
          }
        }
        let let_toks = format!("let ${} = ", name.0);
        write!(writer, "{}", let_toks).unwrap();
        let body_indent = indent + let_toks.len();
        if self._write(body.clone(), body_indent, false, writer) {
          writeln!(writer, "").unwrap();
        }
        writeln!(writer, " in").unwrap();
        self._write(rest.clone(), indent, true, writer)
      }
      &LTerm::Switch(ref query, ref top, ref bot) => {
        let switch_toks = format!("switch ");
        write!(writer, "{}", switch_toks).unwrap();
        let query_indent = indent + switch_toks.len();
        if self._write(query.clone(), query_indent, false, writer) {
          writeln!(writer, "").unwrap();
        }
        writeln!(writer, " then").unwrap();
        self._write(top.clone(), indent + 4, true, writer);
        writeln!(writer, " |").unwrap();
        self._write(bot.clone(), indent + 4, true, writer)
      }
      &LTerm::Tuple(ref _elems) => {
        // FIXME
        write!(writer, "<tup>").unwrap();
        false
      }
      &LTerm::BitLit(x) => {
        // TODO
        write!(writer, "{}", if x { "tee" } else { "bot" }).unwrap();
        false
      }
      &LTerm::IntLit(x) => {
        // TODO
        write!(writer, "{}", x).unwrap();
        false
      }
      &LTerm::FloLit(x) => {
        // TODO
        write!(writer, "{:.0}f", x).unwrap();
        false
      }
      &LTerm::Lookup(ref lookup_var) => {
        // TODO
        write!(writer, "${}", lookup_var.0).unwrap();
        false
      }
      e => {
        panic!("pretty print: unimplemented ltree expr: {:?}", e);
      }
    }
  }

  pub fn write_to<W: Write>(&mut self, ltree: LExpr, writer: &mut W) {
    self._write(ltree, 0, true, writer);
    writeln!(writer, "").unwrap();
  }

  pub fn print(&mut self, ltree: LExpr) {
    let w = stdout();
    self.write_to(ltree, &mut w.lock());
  }
}

pub enum LTreePrettyPrinterVerbosity {
  V0,
  V1,
  V2,
  V3,
}

pub struct LTreePrettyPrinter {
  //v: LTreePrettyPrinterVerbosity,
}

impl LTreePrettyPrinter {
  fn _write<W: Write>(&mut self, lroot: &LTree, ltree: LExpr, indent: usize, indent_first: bool, writer: &mut W) -> bool {
    // TODO
    if indent_first {
      for _ in 0 .. indent {
        write!(writer, " ").unwrap();
      }
    }
    match &*ltree.term {
      &LTerm::Apply(ref head, ref args) => {
        // TODO
        let apply_toks = "apply ";
        write!(writer, "{}", apply_toks).unwrap();
        self._write(lroot, head.clone(), indent + apply_toks.len() + 2, false, writer);
        writeln!(writer, "").unwrap();
        for arg in args.iter() {
          for _ in 0 .. indent + apply_toks.len() {
            write!(writer, " ").unwrap();
          }
          self._write(lroot, arg.clone(), indent + apply_toks.len() + 2, false, writer);
          writeln!(writer, "").unwrap();
        }
        false
      }
      &LTerm::Adj(ref sink) => {
        if let Some(ref free_env) = ltree.maybe_free_env(lroot) {
          write!(writer, "# INFO: free env: [").unwrap();
          let mut first_it = true;
          for v in free_env.inner.ones() {
            if first_it {
              write!(writer, "${}", v).unwrap();
              first_it = false;
            } else {
              write!(writer, ", ${}", v).unwrap();
            }
          }
          writeln!(writer, "]").unwrap();
          for _ in 0 .. indent {
            write!(writer, " ").unwrap();
          }
        }
        let adj_toks = format!("adj ");
        write!(writer, "{}", adj_toks).unwrap();
        let sink_indent = indent + adj_toks.len();
        self._write(lroot, sink.clone(), sink_indent, false, writer)
      }
      &LTerm::Lambda(ref params, ref body) => {
        // FIXME
        write!(writer, "\\").unwrap();
        for (param_idx, param) in params.iter().enumerate() {
          if param_idx == 0 {
            write!(writer, "${}", param.0).unwrap();
          } else {
            write!(writer, ", ${}", param.0).unwrap();
          }
        }
        write!(writer, ". <lam.body>").unwrap();
        false
      }
      &LTerm::VMExt(LTermVMExt::BcLambda(..)) => {
        // FIXME
        write!(writer, "<bclam>").unwrap();
        false
      }
      &LTerm::Let(ref name, ref body, ref rest) => {
        // TODO
        if let Some(ref free_env) = ltree.maybe_free_env(lroot) {
          write!(writer, "# INFO: free env: [").unwrap();
          let mut first_it = true;
          for v in free_env.inner.ones() {
            if first_it {
              write!(writer, "${}", v).unwrap();
              first_it = false;
            } else {
              write!(writer, ", ${}", v).unwrap();
            }
          }
          writeln!(writer, "]").unwrap();
          for _ in 0 .. indent {
            write!(writer, " ").unwrap();
          }
        }
        let let_toks = format!("let ${} = ", name.0);
        write!(writer, "{}", let_toks).unwrap();
        let body_indent = indent + let_toks.len();
        if self._write(lroot, body.clone(), body_indent, false, writer) {
          writeln!(writer, "").unwrap();
        }
        writeln!(writer, " in").unwrap();
        self._write(lroot, rest.clone(), indent, true, writer)
      }
      &LTerm::Switch(ref query, ref top, ref bot) => {
        let switch_toks = format!("switch ");
        write!(writer, "{}", switch_toks).unwrap();
        let query_indent = indent + switch_toks.len();
        if self._write(lroot, query.clone(), query_indent, false, writer) {
          writeln!(writer, "").unwrap();
        }
        writeln!(writer, " then").unwrap();
        self._write(lroot, top.clone(), indent + 4, true, writer);
        writeln!(writer, " |").unwrap();
        self._write(lroot, bot.clone(), indent + 4, true, writer)
      }
      &LTerm::Tuple(ref _elems) => {
        // FIXME
        write!(writer, "<tup>").unwrap();
        false
      }
      &LTerm::BitLit(x) => {
        // TODO
        write!(writer, "{}", if x { "tee" } else { "bot" }).unwrap();
        false
      }
      &LTerm::IntLit(x) => {
        // TODO
        write!(writer, "{}", x).unwrap();
        false
      }
      &LTerm::FloLit(x) => {
        // TODO
        write!(writer, "{:.0}f", x).unwrap();
        false
      }
      &LTerm::Lookup(ref lookup_var) => {
        // TODO
        write!(writer, "${}", lookup_var.0).unwrap();
        false
      }
      e => {
        panic!("pretty print: unimplemented ltree expr: {:?}", e);
      }
    }
  }

  pub fn write_to<W: Write>(&mut self, ltree: LTree, writer: &mut W) {
    self._write(&ltree, ltree.root.clone(), 0, true, writer);
    writeln!(writer, "").unwrap();
  }

  pub fn print(&mut self, ltree: LTree) {
    let w = stdout();
    self.write_to(ltree, &mut w.lock());
  }
}
