use locus_core::run::CredentialProxyConfig;

#[test]
fn credential_proxy_accepts_only_the_host_proxy_root() {
    assert!(CredentialProxyConfig::new("http://host.docker.internal:43800/").is_ok());
    for endpoint in [
        "http://host.docker.internal:43800",
        "https://host.docker.internal:43800/",
        "http://host.docker.internal:8787/",
        "http://token@host.docker.internal:43800/",
        "http://host.docker.internal:43800/?token=secret",
        "http://host.docker.internal:43800/#secret",
        "http://host.docker.internal:43800/v1",
    ] {
        assert!(
            CredentialProxyConfig::new(endpoint).is_err(),
            "{endpoint} must not be accepted"
        );
    }
}
