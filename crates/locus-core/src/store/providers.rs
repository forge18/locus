//! Persistence for provider references and their model catalogs (`core.providers`).
//!
//! Moved out of `services/provider.rs` so every query in the crate lives under `store/`.

use anyhow::anyhow;
use anyhow::{bail, Result};
use uuid::Uuid;

use crate::{
    services::provider::{
        selector_projection, ModelSelectorOption, ProviderModel, ProviderReference,
        ProviderVerificationMetadata,
    },
    store::Store,
};

impl Store {
    /// Persist only the provider identity and OS-keychain locator.
    pub async fn persist_provider_reference(&self, provider: &ProviderReference) -> Result<()> {
        sqlx::query(
            "INSERT INTO core.providers (id, identifier, keychain_reference)
             VALUES ($1, $2, $3)
             ON CONFLICT (id) DO UPDATE SET
                 identifier = EXCLUDED.identifier,
                 keychain_reference = EXCLUDED.keychain_reference,
                 updated_at = now()",
        )
        .bind(provider.id)
        .bind(&provider.identifier)
        .bind(provider.keychain_reference.as_str())
        .execute(self.pool())
        .await?;
        Ok(())
    }

    /// Replace the curated model catalog for one provider atomically.
    pub async fn persist_provider_models(
        &self,
        provider_id: Uuid,
        models: &[ProviderModel],
    ) -> Result<()> {
        if models.iter().any(|model| model.provider_id != provider_id) {
            bail!("provider model belongs to a different provider")
        }
        let mut model_ids = std::collections::HashSet::new();
        if models
            .iter()
            .any(|model| !model_ids.insert(&model.model_id))
        {
            bail!("provider model catalog contains duplicate model ids")
        }

        let mut transaction = self.pool().begin().await?;
        sqlx::query("DELETE FROM core.provider_models WHERE provider_id = $1")
            .bind(provider_id)
            .execute(&mut *transaction)
            .await?;
        for model in models {
            sqlx::query(
                "INSERT INTO core.provider_models (provider_id, model_id, alias, selector_included)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(model.provider_id)
            .bind(&model.model_id)
            .bind(&model.alias)
            .bind(model.selector_included)
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    /// Persist the secret-free result of the latest host-side verification.
    pub async fn persist_provider_verification_metadata(
        &self,
        provider_id: Uuid,
        metadata: &ProviderVerificationMetadata,
    ) -> Result<()> {
        let updated = sqlx::query(
            "UPDATE core.providers
             SET verification_at = $2::text::timestamptz,
                 verification_model_count = $3,
                 verification_status = $4,
                 verification_expires_at = $5::text::timestamptz,
                 updated_at = now()
             WHERE id = $1",
        )
        .bind(provider_id)
        .bind(&metadata.verified_at)
        .bind(i32::try_from(metadata.model_count).map_err(|_| anyhow!("model count is too large"))?)
        .bind(metadata.status.as_str())
        .bind(metadata.expires_at.as_deref())
        .execute(self.pool())
        .await?;
        if updated.rows_affected() != 1 {
            bail!("provider `{provider_id}` does not exist")
        }
        Ok(())
    }

    /// Load one provider's visible catalog in deterministic model-id order for a selector.
    pub async fn provider_selector_projection(
        &self,
        provider_id: Uuid,
    ) -> Result<Vec<ModelSelectorOption>> {
        use sqlx::Row;

        let rows = sqlx::query(
            "SELECT provider_id, model_id, alias, selector_included
             FROM core.provider_models
             WHERE provider_id = $1
             ORDER BY model_id",
        )
        .bind(provider_id)
        .fetch_all(self.pool())
        .await?;
        let models = rows
            .into_iter()
            .map(|row| {
                Ok(ProviderModel {
                    provider_id: row.try_get("provider_id")?,
                    model_id: row.try_get("model_id")?,
                    alias: row.try_get("alias")?,
                    selector_included: row.try_get("selector_included")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(selector_projection(&models))
    }
}
