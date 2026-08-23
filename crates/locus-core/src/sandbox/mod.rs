//! Container sandbox primitives shared by run supervision and project services.
//!
//! The registry declares images; this module turns those declarations into deterministic
//! image, container, credential-proxy, and network requests without naming a harness.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use bollard::{
    query_parameters::{RemoveContainerOptions, StartContainerOptions, StopContainerOptions},
    Docker, API_DEFAULT_VERSION,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    harness::registry::{HarnessDefinition, Image},
    services::tools::{ImageTool, ProjectToolScope, RoleToolScope, ToolCatalog},
};

pub const PORT_START: u16 = 43_000;
pub const PORT_END: u16 = 43_999;
pub const CONFIG_SOURCE: &str = "/locus/config-ro";
pub const CONFIG_DESTINATION: &str = "/locus/config";
pub const LOCUS_SOCKET: &str = "/run/locus.sock";

pub mod credential_proxy;
pub mod docker;
pub mod egress;
pub mod image;
pub mod mounts;
pub mod ports;
pub mod services;
pub mod workspace;

pub use egress::{AuditSink, EgressTarget, EgressTier, OutboundAudit};
