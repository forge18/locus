//! Persistence for credential-proxy egress audits (`agents.credential_proxy_audits`).
//!
//! Moved out of `store/mod.rs` so every query lives in its own module, and so the proxy
//! can record through a sink instead of holding a store. `sandbox` names nothing here.

use std::sync::Arc;

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::{
    sandbox::egress::{AuditSink, EgressTarget, EgressTier, OutboundAudit},
    store::Store,
};

impl Store {
    /// Persists one credential-proxy decision without any secret material.
    pub async fn persist_credential_proxy_audit(&self, audit: &OutboundAudit) -> Result<()> {
        let target = match audit.target {
            EgressTarget::Model => "model",
            EgressTarget::Package => "package",
            EgressTarget::Other => "other",
        };
        let tier = match audit.tier {
            EgressTier::None => "none",
            EgressTier::Model => "model",
            EgressTier::Packages => "packages",
            EgressTier::Open => "open",
        };
        sqlx::query(
            "INSERT INTO agents.credential_proxy_audits
             (id, run_id, target, tier, allowed, credential_class)
             VALUES ($1, $2::uuid, $3, $4, $5, $6)",
        )
        .bind(Uuid::new_v4())
        .bind(&audit.run_id)
        .bind(target)
        .bind(tier)
        .bind(audit.allowed)
        .bind(audit.credential_class)
        .execute(&self.pool)
        .await
        .context("persist credential proxy audit")?;
        Ok(())
    }
}

/// Adapts [`Store`] to the sink the credential proxy takes.
///
/// The proxy runs on a plain OS thread, so recording is synchronous. The runtime is built
/// once here rather than per outbound call.
pub struct StoreAuditSink {
    store: Store,
    /// `Option` only so `Drop` can hand it to `shutdown_background`. Dropping a runtime
    /// the ordinary way panics when it happens inside an async context, and the sink
    /// outlives the call that built it.
    runtime: Option<tokio::runtime::Runtime>,
}

impl StoreAuditSink {
    pub fn new(store: Store) -> Result<Arc<Self>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("create credential proxy audit runtime")?;
        Ok(Arc::new(Self {
            store,
            runtime: Some(runtime),
        }))
    }
}

impl Drop for StoreAuditSink {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

impl AuditSink for StoreAuditSink {
    fn record(&self, audit: &OutboundAudit) -> Result<()> {
        self.runtime
            .as_ref()
            .expect("audit runtime outlives every record call")
            .block_on(self.store.persist_credential_proxy_audit(audit))
    }
}

#[cfg(test)]
mod credential_proxy_audits {
    use crate::ids::{ProjectId, RunId, SessionId};
    use std::sync::Arc;

    use sqlx::{query, query_as, query_scalar};

    use crate::sandbox::{
        credential_proxy::CredentialProxy, egress::EgressTarget, egress::EgressTier,
    };
    use crate::store::{audits::StoreAuditSink, Store};

    #[tokio::test]
    async fn credential_proxy_audits_survive_store_reconnect() {
        let (container, _cleanup) =
            crate::testkit::postgres::start_postgres_named("locus-credential-audit").await;
        let database_url = container.database_url();
        let store = Store::connect(&database_url).await.expect("connect store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &crate::store::NoopMigrationBackup,
                &crate::store::test_backup_config(),
            )
            .await
            .expect("run migrations");

        let project_id = ProjectId::generate();
        let agent_id = uuid::Uuid::new_v4();
        let session_id = SessionId::generate();
        let run_id = RunId::generate();
        query("INSERT INTO core.projects (id, name) VALUES ($1, 'audit project')")
            .bind(project_id)
            .execute(store.pool())
            .await
            .expect("insert project");
        query("INSERT INTO agents.agent_defs (id, name, version, frontmatter, body) VALUES ($1, 'audit agent', 1, '{}'::jsonb, '')")
            .bind(agent_id)
            .execute(store.pool())
            .await
            .expect("insert agent");
        query("INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch) VALUES ($1, $2, $3, 'audit session', 'agent/audit')")
            .bind(session_id)
            .bind(project_id)
            .bind(agent_id)
            .execute(store.pool())
            .await
            .expect("insert session");
        query("INSERT INTO agents.runs (id, session_id, resolved_model_id, status) VALUES ($1, $2, 'model', 'running')")
            .bind(run_id)
            .bind(session_id)
            .execute(store.pool())
            .await
            .expect("insert run");

        let proxy = Arc::new(CredentialProxy::new("host-secret", "api_key"));
        proxy.attach_audit_sink(StoreAuditSink::new(store.clone()).expect("audit sink"));
        proxy
            .configure_run(&run_id.to_string(), "nonce", EgressTier::Model)
            .unwrap();
        let proxy_for_request = proxy.clone();
        tokio::task::spawn_blocking(move || {
            proxy_for_request
                .request(
                    &run_id.to_string(),
                    "nonce",
                    "sk-locus-sentinel",
                    EgressTarget::Model,
                    |_| Ok(()),
                )
                .expect("persist audit through proxy request");
        })
        .await
        .expect("join proxy request");
        drop(store);

        let recovered = Store::connect(&database_url)
            .await
            .expect("reconnect store");
        let row: (String, String, bool, String) = query_as(
            "SELECT target, tier, allowed, credential_class
             FROM agents.credential_proxy_audits WHERE run_id = $1",
        )
        .bind(run_id)
        .fetch_one(recovered.pool())
        .await
        .expect("read durable audit");
        assert_eq!(
            row,
            ("model".into(), "model".into(), true, "api_key".into())
        );
        let secret_count: i64 = query_scalar(
            "SELECT count(*) FROM agents.credential_proxy_audits
             WHERE credential_class = 'host-secret'",
        )
        .fetch_one(recovered.pool())
        .await
        .expect("inspect audit secrecy");
        assert_eq!(secret_count, 0);
    }
}
