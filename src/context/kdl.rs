use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use kdl::{KdlDocument, KdlValue};

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

fn pairs<I>(mut iter: I) -> impl Iterator<Item = (I::Item, I::Item)>
where
    I: Iterator,
{
    std::iter::from_fn(move || Some((iter.next()?, iter.next()?)))
}

fn parse_env_set(node: &kdl::KdlNode) -> Result<EnvSet> {
    let args = node
        .iter_children()
        .map(|c| {
            let k = c.name().to_string();
            let v = c.get(0).context("Env value required")?.to_string();
            Ok((k, v))
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

impl Into<String> for context::Context {
    fn into(self) -> String {}
}
