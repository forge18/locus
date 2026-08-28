//! Renderer-independent workflow graph data and authoring validation.
//!
//! The canvas and the compiled workflow deliberately use different shapes. This
//! module owns the authored shape so positions, named handles, and loop-back
//! intent survive a `graph` JSONB round trip without depending on a renderer.

use std::collections::{BTreeMap, BTreeSet};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use thiserror::Error;

const GRAPH_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NodeKind {
    Goal,
    Agent,
    Task,
    Loop,
    Condition,
    Gate,
    Verify,
}

impl NodeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Goal => "Goal",
            Self::Agent => "Agent",
            Self::Task => "Task",
            Self::Loop => "Loop",
            Self::Condition => "Condition",
            Self::Gate => "Gate",
            Self::Verify => "Verify",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "goal" => Some(Self::Goal),
            "agent" => Some(Self::Agent),
            "task" => Some(Self::Task),
            "loop" => Some(Self::Loop),
            "condition" => Some(Self::Condition),
            "gate" => Some(Self::Gate),
            "verify" => Some(Self::Verify),
            _ => None,
        }
    }

    pub const fn handles(self) -> (&'static [&'static str], &'static [&'static str]) {
        match self {
            Self::Goal => (&["approved"], &["start"]),
            Self::Agent | Self::Task => (&["in"], &["out"]),
            Self::Condition => (&["in"], &["true", "false"]),
            Self::Loop => (&["in"], &["body", "exit"]),
            Self::Gate => (&["in"], &["pass", "reject"]),
            Self::Verify => (&["in"], &["passed", "failed"]),
        }
    }

    pub const fn input_handles(self) -> &'static [&'static str] {
        match self {
            Self::Goal => &["approved"],
            Self::Agent | Self::Task | Self::Loop | Self::Condition | Self::Gate | Self::Verify => {
                &["in"]
            }
        }
    }

    pub const fn output_handles(self) -> &'static [&'static str] {
        match self {
            Self::Goal => &["start"],
            Self::Agent | Self::Task => &["out"],
            Self::Loop => &["body", "exit"],
            Self::Condition => &["true", "false"],
            Self::Gate => &["pass", "reject"],
            Self::Verify => &["passed", "failed"],
        }
    }
}

impl Serialize for NodeKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NodeKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).ok_or_else(|| de::Error::custom(format!("unknown node kind `{value}`")))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphPosition {
    pub x: f64,
    pub y: f64,
}

impl GraphPosition {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WorkflowNode {
    pub id: String,
    pub kind: NodeKind,
    pub position: GraphPosition,
    #[serde(default = "empty_object")]
    pub data: Value,
    #[serde(rename = "loop", default, skip_serializing_if = "Option::is_none")]
    pub loop_id: Option<String>,
}

fn empty_object() -> Value {
    Value::Object(serde_json::Map::new())
}

impl WorkflowNode {
    pub fn new(
        id: impl Into<String>,
        kind: NodeKind,
        position: GraphPosition,
        data: Value,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            position,
            data,
            loop_id: None,
        }
    }

    pub fn with_loop(mut self, loop_id: impl Into<String>) -> Self {
        self.loop_id = Some(loop_id.into());
        self
    }

    pub fn text(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(Value::as_str)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    #[serde(rename = "sourceHandle")]
    pub source_handle: String,
    pub target: String,
    #[serde(rename = "targetHandle")]
    pub target_handle: String,
    #[serde(rename = "loopBack", default, skip_serializing_if = "Option::is_none")]
    pub loop_back: Option<bool>,
}

impl GraphEdge {
    pub fn loop_back(&self) -> bool {
        self.loop_back.unwrap_or(false)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WorkflowGraph {
    pub version: u32,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<GraphEdge>,
}

impl WorkflowGraph {
    pub const fn version() -> u32 {
        GRAPH_VERSION
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("workflow graph is serializable")
    }

    pub fn from_value(value: &Value) -> Result<Self, String> {
        let graph: Self =
            serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
        if graph.version != GRAPH_VERSION {
            return Err(format!(
                "unsupported workflow graph version: {}",
                graph.version
            ));
        }
        Ok(graph)
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
        Self::from_value(&value)
    }

    pub fn validation_errors(&self) -> Vec<GraphValidationError> {
        validate_graph(self)
    }

    pub fn validate(&self) -> Result<(), Vec<GraphValidationError>> {
        let errors = self.validation_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn serialize_graph(graph: &WorkflowGraph) -> String {
    graph.to_json()
}

pub fn deserialize_graph(json: &str) -> Result<WorkflowGraph, String> {
    WorkflowGraph::from_json(json)
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum GraphValidationError {
    #[error("graph version must be {expected}, got {actual}")]
    UnsupportedVersion { expected: u32, actual: u32 },
    #[error("node `{node}` has an empty id")]
    EmptyNodeId { node: String },
    #[error("node `{node}` is duplicated")]
    DuplicateNode { node: String },
    #[error("node `{node}` has an unsupported kind")]
    UnsupportedNodeKind { node: String },
    #[error("edge `{edge}` has an empty id")]
    EmptyEdgeId { edge: String },
    #[error("edge `{edge}` is duplicated")]
    DuplicateEdge { edge: String },
    #[error("edge `{edge}` references missing node `{node}`")]
    MissingEdgeNode { edge: String, node: String },
    #[error("edge `{edge}` from node `{node}` has no source handle")]
    MissingSourceHandle { edge: String, node: String },
    #[error("edge `{edge}` to node `{node}` has no target handle")]
    MissingTargetHandle { edge: String, node: String },
    #[error("edge `{edge}` uses unresolved source handle `{handle}` on node `{node}`")]
    UnresolvedSourceHandle {
        edge: String,
        node: String,
        handle: String,
    },
    #[error("edge `{edge}` uses unresolved target handle `{handle}` on node `{node}`")]
    UnresolvedTargetHandle {
        edge: String,
        node: String,
        handle: String,
    },
    #[error("undeclared cycle through node `{node}`")]
    Cycle { node: String },
    #[error("Verify node `{node}` has no command")]
    MissingVerifyCommand { node: String },
    #[error("workflow has no Verify node; offending node is `<workflow>`")]
    MissingVerifyNode,
    #[error("goal node `{node}` is unreachable")]
    UnreachableGoal { node: String },
    #[error("Loop node `{node}` has no termination condition")]
    NonTerminatingLoop { node: String },
    #[error("Agent node `{node}` combines builder and tester roles for definition `{agent}`")]
    RoleContamination { node: String, agent: String },
    #[error("Agent node `{node}` widens `{capability}` beyond its definition")]
    PermissionWidened { node: String, capability: String },
}

pub fn validate_graph(graph: &WorkflowGraph) -> Vec<GraphValidationError> {
    let mut errors = Vec::new();
    if graph.version != GRAPH_VERSION {
        errors.push(GraphValidationError::UnsupportedVersion {
            expected: GRAPH_VERSION,
            actual: graph.version,
        });
    }

    let mut nodes = BTreeMap::new();
    for node in &graph.nodes {
        if node.id.trim().is_empty() {
            errors.push(GraphValidationError::EmptyNodeId {
                node: "<empty>".into(),
            });
            continue;
        }
        if nodes.insert(node.id.clone(), node).is_some() {
            errors.push(GraphValidationError::DuplicateNode {
                node: node.id.clone(),
            });
        }
    }

    let mut edge_ids = BTreeSet::new();
    let mut adjacency: BTreeMap<&str, Vec<&str>> =
        nodes.keys().map(|id| (id.as_str(), Vec::new())).collect();
    for edge in &graph.edges {
        if edge.id.trim().is_empty() {
            errors.push(GraphValidationError::EmptyEdgeId {
                edge: "<empty>".into(),
            });
        } else if !edge_ids.insert(edge.id.clone()) {
            errors.push(GraphValidationError::DuplicateEdge {
                edge: edge.id.clone(),
            });
        }
        let Some(source) = nodes.get(&edge.source) else {
            errors.push(GraphValidationError::MissingEdgeNode {
                edge: edge.id.clone(),
                node: edge.source.clone(),
            });
            continue;
        };
        let Some(target) = nodes.get(&edge.target) else {
            errors.push(GraphValidationError::MissingEdgeNode {
                edge: edge.id.clone(),
                node: edge.target.clone(),
            });
            continue;
        };
        if edge.source_handle.trim().is_empty() {
            errors.push(GraphValidationError::MissingSourceHandle {
                edge: edge.id.clone(),
                node: source.id.clone(),
            });
        } else if !source
            .kind
            .output_handles()
            .contains(&edge.source_handle.as_str())
        {
            errors.push(GraphValidationError::UnresolvedSourceHandle {
                edge: edge.id.clone(),
                node: source.id.clone(),
                handle: edge.source_handle.clone(),
            });
        }
        if edge.target_handle.trim().is_empty() {
            errors.push(GraphValidationError::MissingTargetHandle {
                edge: edge.id.clone(),
                node: target.id.clone(),
            });
        } else if !target
            .kind
            .input_handles()
            .contains(&edge.target_handle.as_str())
        {
            errors.push(GraphValidationError::UnresolvedTargetHandle {
                edge: edge.id.clone(),
                node: target.id.clone(),
                handle: edge.target_handle.clone(),
            });
        }
        if !edge.loop_back() {
            adjacency
                .entry(edge.source.as_str())
                .or_default()
                .push(edge.target.as_str());
        }
    }

    let verifies: Vec<_> = graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Verify)
        .collect();
    if verifies.is_empty() {
        errors.push(GraphValidationError::MissingVerifyNode);
    }
    for verify in verifies {
        if verify
            .text("command")
            .is_none_or(|command| command.trim().is_empty())
        {
            errors.push(GraphValidationError::MissingVerifyCommand {
                node: verify.id.clone(),
            });
        }
    }

    errors.extend(validate_cycles(&nodes, &adjacency));
    errors.extend(validate_goals(&graph.nodes, &adjacency));
    errors.extend(validate_loops(graph));
    errors.extend(validate_roles_and_permissions(&graph.nodes));
    errors
}

fn validate_cycles<'a>(
    nodes: &BTreeMap<String, &WorkflowNode>,
    adjacency: &BTreeMap<&'a str, Vec<&'a str>>,
) -> Vec<GraphValidationError> {
    let mut errors = Vec::new();
    let mut state = BTreeMap::<&str, u8>::new();
    let mut path = Vec::new();
    for id in nodes.keys().map(String::as_str) {
        if state.get(id).copied().unwrap_or(0) == 0 {
            visit_cycle(id, adjacency, &mut state, &mut path, &mut errors);
        }
    }
    errors
}

fn visit_cycle<'a>(
    id: &'a str,
    adjacency: &BTreeMap<&'a str, Vec<&'a str>>,
    state: &mut BTreeMap<&'a str, u8>,
    path: &mut Vec<&'a str>,
    errors: &mut Vec<GraphValidationError>,
) {
    state.insert(id, 1);
    path.push(id);
    for next in adjacency.get(id).into_iter().flatten() {
        match state.get(next).copied().unwrap_or(0) {
            1 => errors.push(GraphValidationError::Cycle {
                node: (*next).into(),
            }),
            0 => visit_cycle(next, adjacency, state, path, errors),
            _ => {}
        }
    }
    path.pop();
    state.insert(id, 2);
}

fn validate_goals(
    nodes: &[WorkflowNode],
    adjacency: &BTreeMap<&str, Vec<&str>>,
) -> Vec<GraphValidationError> {
    let mut incoming = BTreeSet::new();
    for targets in adjacency.values() {
        incoming.extend(targets.iter().copied());
    }
    let roots: Vec<_> = nodes
        .iter()
        .filter(|node| !incoming.contains(node.id.as_str()))
        .map(|node| node.id.as_str())
        .collect();
    let mut reachable = BTreeSet::new();
    let mut pending = roots;
    while let Some(id) = pending.pop() {
        if !reachable.insert(id) {
            continue;
        }
        pending.extend(adjacency.get(id).into_iter().flatten().copied());
    }
    nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Goal && !reachable.contains(node.id.as_str()))
        .map(|node| GraphValidationError::UnreachableGoal {
            node: node.id.clone(),
        })
        .collect()
}

fn validate_loops(graph: &WorkflowGraph) -> Vec<GraphValidationError> {
    let mut errors = Vec::new();
    for loop_node in graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Loop)
    {
        let members: BTreeSet<_> = graph
            .nodes
            .iter()
            .filter(|node| node.loop_id.as_deref() == Some(loop_node.id.as_str()))
            .map(|node| node.id.as_str())
            .collect();
        let max_iterations = loop_node
            .data
            .get("max_iterations")
            .or_else(|| loop_node.data.get("maxIterations"))
            .and_then(Value::as_u64)
            .is_some_and(|value| value > 0);
        let exit_wired = graph.edges.iter().any(|edge| {
            edge.source == loop_node.id && edge.source_handle == "exit" && !edge.loop_back()
        });
        let routes_out = graph.edges.iter().any(|edge| {
            members.contains(edge.source.as_str())
                && !members.contains(edge.target.as_str())
                && edge.target != loop_node.id
                && !edge.loop_back()
        });
        if !max_iterations && !exit_wired && !routes_out {
            errors.push(GraphValidationError::NonTerminatingLoop {
                node: loop_node.id.clone(),
            });
        }
    }
    errors
}

fn validate_roles_and_permissions(nodes: &[WorkflowNode]) -> Vec<GraphValidationError> {
    let mut errors = Vec::new();
    let mut roles: BTreeMap<String, Vec<(&WorkflowNode, String)>> = BTreeMap::new();
    for node in nodes.iter().filter(|node| node.kind == NodeKind::Agent) {
        let agent = node
            .text("agent")
            .or_else(|| node.text("agent_def_id"))
            .unwrap_or(node.id.as_str())
            .to_owned();
        let role = node.text("role").unwrap_or_default().to_ascii_lowercase();
        roles.entry(agent.clone()).or_default().push((node, role));
        if let Err(error) = validate_agent_permissions(node) {
            errors.push(error);
        }
    }
    for (agent, entries) in roles {
        let has_builder = entries.iter().any(|(_, role)| role == "builder");
        let has_tester = entries.iter().any(|(_, role)| role == "tester");
        let has_reviewer = entries.iter().any(|(_, role)| role == "reviewer");
        if (has_builder && has_tester) || (has_reviewer && (has_builder || has_tester)) {
            if let Some((node, _)) = entries.last() {
                errors.push(GraphValidationError::RoleContamination {
                    node: node.id.clone(),
                    agent,
                });
            }
        }
    }
    errors
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowNetworkTier {
    Closed,
    Internal,
    Open,
}

impl WorkflowNetworkTier {
    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "closed" | "none" => Some(Self::Closed),
            "internal" | "model" => Some(Self::Internal),
            "open" | "packages" => Some(Self::Open),
            _ => None,
        }
    }

    fn rank(&self) -> u8 {
        match self {
            Self::Closed => 0,
            Self::Internal => 1,
            Self::Open => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowWriteScope {
    None,
    Branch,
    Workspace,
}

impl WorkflowWriteScope {
    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "none" | "read_only" | "readonly" => Some(Self::None),
            "branch" => Some(Self::Branch),
            "workspace" | "project" => Some(Self::Workspace),
            _ => None,
        }
    }

    fn rank(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Branch => 1,
            Self::Workspace => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentPermissions {
    tools: BTreeSet<String>,
    network: Option<WorkflowNetworkTier>,
    write: Option<WorkflowWriteScope>,
}

impl AgentPermissions {
    pub fn tools(&self) -> &BTreeSet<String> {
        &self.tools
    }

    pub fn network(&self) -> Option<&WorkflowNetworkTier> {
        self.network.as_ref()
    }

    pub fn write(&self) -> Option<&WorkflowWriteScope> {
        self.write.as_ref()
    }
}

pub fn node_permissions(node: &WorkflowNode) -> Option<AgentPermissions> {
    (node.kind == NodeKind::Agent).then(|| AgentPermissions {
        tools: string_set(node.data.get("tools")),
        network: node.text("network").and_then(WorkflowNetworkTier::parse),
        write: node
            .text("write_scope")
            .or_else(|| node.text("write"))
            .and_then(WorkflowWriteScope::parse),
    })
}

pub fn validate_agent_permissions(node: &WorkflowNode) -> Result<(), GraphValidationError> {
    let Some(permissions) = node_permissions(node) else {
        return Ok(());
    };
    let definition_tools = node
        .data
        .get("definition_tools")
        .or_else(|| node.data.get("allowed_tools"))
        .map(|value| string_set(Some(value)));
    if let Some(definition_tools) = definition_tools {
        if !permissions.tools.is_subset(&definition_tools) {
            return Err(GraphValidationError::PermissionWidened {
                node: node.id.clone(),
                capability: "tools".into(),
            });
        }
    }
    if let (Some(node_network), Some(definition_network)) = (
        permissions.network.as_ref(),
        node.text("definition_network")
            .and_then(WorkflowNetworkTier::parse),
    ) {
        if node_network.rank() > definition_network.rank() {
            return Err(GraphValidationError::PermissionWidened {
                node: node.id.clone(),
                capability: "network".into(),
            });
        }
    }
    if let (Some(node_write), Some(definition_write)) = (
        permissions.write.as_ref(),
        node.text("definition_write_scope")
            .or_else(|| node.text("definition_write"))
            .and_then(WorkflowWriteScope::parse),
    ) {
        if node_write.rank() > definition_write.rank() {
            return Err(GraphValidationError::PermissionWidened {
                node: node.id.clone(),
                capability: "write_scope".into(),
            });
        }
    }
    Ok(())
}

fn string_set(value: Option<&Value>) -> BTreeSet<String> {
    match value {
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect(),
        Some(Value::String(value)) => value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        _ => BTreeSet::new(),
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WorkflowTaskDependency {
    pub task_id: String,
    pub blocked_by: String,
}

/// Convert graph edges between Task nodes into board `blocked_by` rows.
pub fn blocked_by_edges(graph: &WorkflowGraph) -> Vec<WorkflowTaskDependency> {
    let kinds: BTreeMap<_, _> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.kind))
        .collect();
    let mut dependencies: Vec<_> = graph
        .edges
        .iter()
        .filter(|edge| !edge.loop_back())
        .filter(|edge| {
            kinds.get(edge.source.as_str()) == Some(&NodeKind::Task)
                && kinds.get(edge.target.as_str()) == Some(&NodeKind::Task)
        })
        .map(|edge| WorkflowTaskDependency {
            task_id: edge.target.clone(),
            blocked_by: edge.source.clone(),
        })
        .collect();
    dependencies.sort();
    dependencies.dedup();
    dependencies
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn node(id: &str, kind: NodeKind, data: Value) -> WorkflowNode {
        WorkflowNode::new(id, kind, GraphPosition::new(10.0, 20.0), data)
    }

    fn edge(id: &str, source: &str, source_handle: &str, target: &str) -> GraphEdge {
        GraphEdge {
            id: id.into(),
            source: source.into(),
            source_handle: source_handle.into(),
            target: target.into(),
            target_handle: "in".into(),
            loop_back: None,
        }
    }

    fn valid_graph() -> WorkflowGraph {
        WorkflowGraph {
            version: 1,
            nodes: vec![
                node("task-a", NodeKind::Task, json!({})),
                node("task-b", NodeKind::Task, json!({})),
                node("verify", NodeKind::Verify, json!({"command": "cargo test"})),
            ],
            edges: vec![
                edge("e-a", "task-a", "out", "task-b"),
                edge("e-b", "task-b", "out", "verify"),
            ],
        }
    }

    #[test]
    fn graph_round_trips_exactly() {
        let graph = valid_graph();
        let json = serialize_graph(&graph);
        assert_eq!(
            json,
            deserialize_graph(&json).expect("deserialize").to_json()
        );
    }

    #[test]
    fn handles_are_typed() {
        assert_eq!(NodeKind::Condition.output_handles(), &["true", "false"]);
        assert_eq!(NodeKind::Verify.output_handles(), &["passed", "failed"]);
    }

    #[test]
    fn permissions_only_narrow() {
        let node = node(
            "builder",
            NodeKind::Agent,
            json!({
                "tools": ["git"],
                "definition_tools": ["git", "rg"],
                "network": "internal",
                "definition_network": "open",
                "write": "branch",
                "definition_write": "workspace"
            }),
        );
        assert!(validate_agent_permissions(&node).is_ok());
        let wider = WorkflowNode {
            data: json!({"tools": ["docker"], "definition_tools": ["git"]}),
            ..node
        };
        assert!(matches!(
            validate_agent_permissions(&wider),
            Err(GraphValidationError::PermissionWidened { capability, .. }) if capability == "tools"
        ));
    }

    #[test]
    fn task_edges_become_blocked_by_rows() {
        let dependencies = blocked_by_edges(&valid_graph());
        assert_eq!(
            dependencies,
            vec![WorkflowTaskDependency {
                task_id: "task-b".into(),
                blocked_by: "task-a".into()
            }]
        );
    }
}
