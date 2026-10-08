//! Hindley-Milner Type Inference System (Algorithm W) with Principal Type Derivation.
//! Implements monomorphic instantiation, polymorphic generalization, and substitution unification.

pub mod elaborator;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Var(String),
    Int,
    Bool,
    String,
    Function(Box<Type>, Box<Type>),
    List(Box<Type>),
    Tuple(Vec<Type>),
    Custom(String, Vec<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScheme {
    pub vars: HashSet<String>,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Var(String),
    LitInt(i64),
    LitBool(bool),
    LitString(String),
    Lambda(String, Box<Term>),
    App(Box<Term>, Box<Term>),
    Let(String, Box<Term>, Box<Term>),
    If(Box<Term>, Box<Term>, Box<Term>),
    Tuple(Vec<Term>),
}

#[derive(Debug, Default, Clone)]
pub struct Substitution {
    pub mappings: HashMap<String, Type>,
}

impl Substitution {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, var: String, ty: Type) {
        self.mappings.insert(var, ty);
    }

    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(name) => {
                if let Some(target) = self.mappings.get(name) {
                    self.apply(target)
                } else {
                    ty.clone()
                }
            }
            Type::Function(param, ret) => Type::Function(
                Box::new(self.apply(param)),
                Box::new(self.apply(ret)),
            ),
            Type::List(elem) => Type::List(Box::new(self.apply(elem))),
            Type::Tuple(elems) => Type::Tuple(elems.iter().map(|e| self.apply(e)).collect()),
            Type::Custom(name, args) => Type::Custom(
                name.clone(),
                args.iter().map(|a| self.apply(a)).collect(),
            ),
            _ => ty.clone(),
        }
    }

    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut result = HashMap::new();
        for (k, v) in &other.mappings {
            result.insert(k.clone(), self.apply(v));
        }
        for (k, v) in &self.mappings {
            result.entry(k.clone()).or_insert_with(|| v.clone());
        }
        Substitution { mappings: result }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TypeEnvironment {
    pub bindings: HashMap<String, TypeScheme>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, scheme: TypeScheme) {
        self.bindings.insert(name, scheme);
    }

    pub fn get(&self, name: &str) -> Option<&TypeScheme> {
        self.bindings.get(name)
    }

    pub fn apply_subst(&self, subst: &Substitution) -> Self {
        let mut new_bindings = HashMap::new();
        for (name, scheme) in &self.bindings {
            let free_vars = scheme.vars.clone();
            let new_ty = subst.apply(&scheme.ty);
            new_bindings.insert(
                name.clone(),
                TypeScheme {
                    vars: free_vars,
                    ty: new_ty,
                },
            );
        }
        TypeEnvironment {
            bindings: new_bindings,
        }
    }

    pub fn free_type_vars(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        for scheme in self.bindings.values() {
            let mut ty_vars = get_free_vars(&scheme.ty);
            for bound in &scheme.vars {
                ty_vars.remove(bound);
            }
            vars.extend(ty_vars);
        }
        vars
    }
}

fn get_free_vars(ty: &Type) -> HashSet<String> {
    match ty {
        Type::Var(name) => {
            let mut set = HashSet::new();
            set.insert(name.clone());
            set
        }
        Type::Function(p, r) => {
            let mut set = get_free_vars(p);
            set.extend(get_free_vars(r));
            set
        }
        Type::List(elem) => get_free_vars(elem),
        Type::Tuple(elems) => {
            let mut set = HashSet::new();
            for e in elems {
                set.extend(get_free_vars(e));
            }
            set
        }
        Type::Custom(_, args) => {
            let mut set = HashSet::new();
            for a in args {
                set.extend(get_free_vars(a));
            }
            set
        }
        _ => HashSet::new(),
    }
}

pub struct HindleyMilnerInference {
    next_id: usize,
}

impl HindleyMilnerInference {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn fresh_var(&mut self) -> Type {
        self.next_id += 1;
        Type::Var(format!("'a{}", self.next_id))
    }

    pub fn instantiate(&mut self, scheme: &TypeScheme) -> Type {
        let mut subst = Substitution::new();
        for var in &scheme.vars {
            subst.insert(var.clone(), self.fresh_var());
        }
        subst.apply(&scheme.ty)
    }

    pub fn generalize(&self, env: &TypeEnvironment, ty: &Type) -> TypeScheme {
        let env_vars = env.free_type_vars();
        let ty_vars = get_free_vars(ty);
        let diff: HashSet<String> = ty_vars.difference(&env_vars).cloned().collect();
        TypeScheme {
            vars: diff,
            ty: ty.clone(),
        }
    }

    pub fn unify(&self, ty1: &Type, ty2: &Type) -> Result<Substitution, String> {
        match (ty1, ty2) {
            (Type::Int, Type::Int) => Ok(Substitution::new()),
            (Type::Bool, Type::Bool) => Ok(Substitution::new()),
            (Type::String, Type::String) => Ok(Substitution::new()),
            (Type::Var(u), t) => self.bind_var(u, t),
            (t, Type::Var(u)) => self.bind_var(u, t),
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                let s1 = self.unify(p1, p2)?;
                let s2 = self.unify(&s1.apply(r1), &s1.apply(r2))?;
                Ok(s2.compose(&s1))
            }
            (Type::List(e1), Type::List(e2)) => self.unify(e1, e2),
            (Type::Tuple(l1), Type::Tuple(l2)) => {
                if l1.len() != l2.len() {
                    return Err(format!("Tuple arity mismatch: {} vs {}", l1.len(), l2.len()));
                }
                let mut current_subst = Substitution::new();
                for (a, b) in l1.iter().zip(l2.iter()) {
                    let s = self.unify(&current_subst.apply(a), &current_subst.apply(b))?;
                    current_subst = s.compose(&current_subst);
                }
                Ok(current_subst)
            }
            (Type::Custom(n1, a1), Type::Custom(n2, a2)) if n1 == n2 && a1.len() == a2.len() => {
                let mut current_subst = Substitution::new();
                for (a, b) in a1.iter().zip(a2.iter()) {
                    let s = self.unify(&current_subst.apply(a), &current_subst.apply(b))?;
                    current_subst = s.compose(&current_subst);
                }
                Ok(current_subst)
            }
            _ => Err(format!("Unification type mismatch: {:?} vs {:?}", ty1, ty2)),
        }
    }

    fn bind_var(&self, var: &str, ty: &Type) -> Result<Substitution, String> {
        if let Type::Var(v) = ty {
            if v == var {
                return Ok(Substitution::new());
            }
        }
        let free = get_free_vars(ty);
        if free.contains(var) {
            return Err(format!("Occurs check failed: type variable {} appears in {:?}", var, ty));
        }
        let mut s = Substitution::new();
        s.insert(var.to_string(), ty.clone());
        Ok(s)
    }

    pub fn infer(&mut self, env: &TypeEnvironment, term: &Term) -> Result<(Substitution, Type), String> {
        match term {
            Term::LitInt(_) => Ok((Substitution::new(), Type::Int)),
            Term::LitBool(_) => Ok((Substitution::new(), Type::Bool)),
            Term::LitString(_) => Ok((Substitution::new(), Type::String)),
            Term::Var(name) => {
                if let Some(scheme) = env.get(name) {
                    let ty = self.instantiate(scheme);
                    Ok((Substitution::new(), ty))
                } else {
                    Err(format!("Unbound identifier: {}", name))
                }
            }
            Term::Lambda(param, body) => {
                let param_ty = self.fresh_var();
                let mut new_env = env.clone();
                new_env.insert(
                    param.clone(),
                    TypeScheme {
                        vars: HashSet::new(),
                        ty: param_ty.clone(),
                    },
                );
                let (subst, body_ty) = self.infer(&new_env, body)?;
                let inferred_param = subst.apply(&param_ty);
                Ok((subst, Type::Function(Box::new(inferred_param), Box::new(body_ty))))
            }
            Term::App(func, arg) => {
                let ret_ty = self.fresh_var();
                let (s1, f_ty) = self.infer(env, func)?;
                let (s2, a_ty) = self.infer(&env.apply_subst(&s1), arg)?;
                let s3 = self.unify(
                    &s2.apply(&f_ty),
                    &Type::Function(Box::new(a_ty), Box::new(ret_ty.clone())),
                )?;
                let final_subst = s3.compose(&s2.compose(&s1));
                Ok((final_subst.clone(), final_subst.apply(&ret_ty)))
            }
            Term::Let(var_name, val_term, body_term) => {
                let (s1, val_ty) = self.infer(env, val_term)?;
                let env1 = env.apply_subst(&s1);
                let scheme = self.generalize(&env1, &val_ty);
                let mut env2 = env1;
                env2.insert(var_name.clone(), scheme);
                let (s2, body_ty) = self.infer(&env2, body_term)?;
                Ok((s2.compose(&s1), body_ty))
            }
            Term::If(cond, then_b, else_b) => {
                let (s1, c_ty) = self.infer(env, cond)?;
                let s2 = self.unify(&c_ty, &Type::Bool)?;
                let s12 = s2.compose(&s1);
                let (s3, t_ty) = self.infer(&env.apply_subst(&s12), then_b)?;
                let s123 = s3.compose(&s12);
                let (s4, e_ty) = self.infer(&env.apply_subst(&s123), else_b)?;
                let s1234 = s4.compose(&s123);
                let s5 = self.unify(&s4.apply(&t_ty), &e_ty)?;
                let final_subst = s5.compose(&s1234);
                Ok((final_subst.clone(), final_subst.apply(&e_ty)))
            }
            Term::Tuple(elems) => {
                let mut current_subst = Substitution::new();
                let mut types = Vec::new();
                for elem in elems {
                    let (s, t) = self.infer(&env.apply_subst(&current_subst), elem)?;
                    current_subst = s.compose(&current_subst);
                    types.push(t);
                }
                let resolved_types = types.into_iter().map(|t| current_subst.apply(&t)).collect();
                Ok((current_subst, Type::Tuple(resolved_types)))
            }
        }
    }
}
