use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

use crate::context::{self, EnvSet, Envs, TmuxPane, TmuxWindow};

fn parse_required_str(node: &kdl::KdlNode, key: &str) -> Result<String> {
    Ok(node
        .get(key)
        .and_then(KdlValue::as_string)
        .context("[Config] invalid config")?
        .to_string())
}

fn parse_optional_str(node: &kdl::KdlNode, key: &str) -> Option<String> {
    node.get(key)
        .and_then(KdlValue::as_string)
        .map(str::to_string)
}

fn parse_window(node: &kdl::KdlNode) -> Result<TmuxWindow> {
    let name = parse_required_str(node, "name")?;
    let cmd = parse_optional_str(node, "cmd");

    Ok(TmuxWindow { name, cmd })
}

fn parse_pane(node: &kdl::KdlNode) -> Result<TmuxPane> {
    let name = parse_required_str(node, "name")?;

    let windows = node
        .iter_children()
        .map(parse_window)
        .collect::<Result<_>>()?;

    Ok(TmuxPane { name, windows })
}

fn parse_env_set(node: &kdl::KdlNode) -> Result<EnvSet> {
    let args = node
        .iter_children()
        .map(|c| {
            let k = c.name().to_string();
            let v = c
                .get(0)
                .and_then(KdlValue::as_string)
                .context("Env set value should be valud string")?;
            Ok((k, v.to_string()))
        })
        .collect::<Result<HashMap<_, _>>>()?;
    Ok(EnvSet(args))
}

impl TryFrom<&str> for context::Context {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let doc: KdlDocument = value.parse()?;
        let ctx_config = doc
            .get("ctx")
            .ok_or(anyhow!("[Config] `ctx` config required"))?;

        let name = parse_required_str(ctx_config, "name")?;
        let root = parse_required_str(ctx_config, "root").map(PathBuf::try_from)??;
        let panes = ctx_config
            .iter_children()
            .map(parse_pane)
            .collect::<Result<_>>()?;

        let env_node = doc.get("env");
        let active_env = env_node.and_then(|n| parse_optional_str(n, "active"));
        let env_sets = env_node
            .map(|en| {
                en.iter_children()
                    .map(|n| {
                        let name = parse_required_str(n, "name")?;
                        let env_set = parse_env_set(n)?;
                        Ok((name, env_set))
                    })
                    .collect::<Result<HashMap<_, _>>>()
            })
            .unwrap_or(Ok(HashMap::new()))?;

        return Ok(Self {
            name,
            root,
            panes,
            active_env,
            env_sets,
        });
    }
}

fn build_children_doc(ch: &mut Vec<KdlNode>) -> KdlDocument {
    let mut children_doc = KdlDocument::new();
    children_doc.nodes_mut().append(ch);
    children_doc
}

fn build_window(window: &TmuxWindow) -> KdlNode {
    let mut node = KdlNode::new("window");
    node["name"] = KdlValue::String(window.name.to_string());
    if let Some(cmd) = &window.cmd {
        node["cmd"] = KdlValue::String(cmd.to_string());
    }

    node
}

fn build_pane(pane: &TmuxPane) -> KdlNode {
    let mut node = KdlNode::new("pane");
    node["name"] = KdlValue::String(pane.name.to_string());
    let mut children = pane.windows.iter().map(build_window).collect();
    node.set_children(build_children_doc(&mut children));

    node
}

fn build_set((name, set): (&String, &EnvSet)) -> KdlNode {
    let mut node = KdlNode::new("set");
    node["name"] = KdlValue::String(name.to_string());
    let mut children = set
        .0
        .iter()
        .map(|(k, v)| {
            let mut n = KdlNode::new(k.to_string());
            n.entries_mut().push(KdlEntry::new(v.to_string()));
            n
        })
        .collect();
    node.set_children(build_children_doc(&mut children));
    node
}

fn build_context(ctx: &context::Context) -> KdlDocument {
    let mut ctx_node = KdlNode::new("ctx");
    ctx_node["name"] = KdlValue::String(ctx.name.to_string());
    ctx_node["root"] = KdlValue::String(ctx.root.to_string_lossy().to_string());
    let mut panes = ctx.panes.iter().map(build_pane).collect();
    ctx_node.set_children(build_children_doc(&mut panes));

    let mut env_node = KdlNode::new("env");
    if let Some(active_env) = &ctx.active_env {
        env_node["active"] = KdlValue::String(active_env.to_string());
    };
    let mut envs = ctx.env_sets.iter().map(build_set).collect();
    env_node.set_children(build_children_doc(&mut envs));

    let mut doc = KdlDocument::new();
    doc.nodes_mut().push(ctx_node);
    doc.nodes_mut().push(env_node);
    doc
}

impl Into<String> for context::Context {
    fn into(self) -> String {
        let mut node = build_context(&self);
        node.autoformat_no_comments();
        node.to_string()
    }
}
