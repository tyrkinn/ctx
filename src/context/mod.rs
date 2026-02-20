mod kdl;

use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct TmuxWindow {
    pub name: String,
    pub cmd: Option<String>,
}

#[derive(Debug)]
pub struct TmuxPane {
    pub name: String,
    pub windows: Vec<TmuxWindow>,
}

#[derive(Debug)]
pub struct EnvSet(pub HashMap<String, String>);

type Envs = HashMap<String, EnvSet>;

#[derive(Debug)]
pub struct Context {
    pub name: String,
    pub root: PathBuf,
    pub panes: Vec<TmuxPane>,
    pub active_env: Option<String>,
    pub env_sets: HashMap<String, EnvSet>,
}
