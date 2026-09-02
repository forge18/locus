import type { Envelope } from "./envelope";
import { dataProvider } from "./provider";

export interface ProviderModel {
  modelId: string;
  alias: string | null;
  selectorIncluded: boolean;
}

export interface ProviderSecretReplaceResponse {
  replaced: boolean;
}

export interface ProviderRecord {
  id: string;
  identifier: string;
  /** An OS-keychain locator; never a credential. */
  keychainReference: string;
  verificationAt: string | null;
  verificationModelCount: number | null;
  verificationStatus: "verified" | "failed" | null;
  verificationExpiresAt: string | null;
  authenticationMethod: "oauth" | "api-key" | "none";
  baseUrl: string | null;
  models: ProviderModel[];
}

export interface ProviderSaveRequest {
  id?: string;
  identifier: string;
  keychainReference: string;
  authenticationMethod: "oauth" | "api-key" | "none";
  baseUrl?: string;
}

export function fetchProviders(): Promise<Envelope<ProviderRecord[]>> {
  return dataProvider().query<ProviderRecord>("providers_list");
}

export function saveProvider(
  request: ProviderSaveRequest,
): Promise<Envelope<ProviderRecord>> {
  return dataProvider().queryOne<ProviderRecord>("provider_save", {
    request,
  });
}

export function saveProviderModels(
  providerId: string,
  models: ProviderModel[],
): Promise<Envelope<ProviderModel[]>> {
  return dataProvider().queryOne<ProviderModel[]>("provider_models_set", {
    request: { providerId, models },
  });
}

export function replaceProviderSecret(
  providerId: string,
  secret: string,
): Promise<Envelope<ProviderSecretReplaceResponse>> {
  return dataProvider().queryOne<ProviderSecretReplaceResponse>(
    "provider_secret_replace",
    { request: { providerId, secret } },
  );
}
