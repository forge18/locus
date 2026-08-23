use locus_core::runtime::run::CredentialProxyConfig;

#[test]
fn credential_proxy_accepts_only_the_host_proxy_root() {
    assert!(CredentialProxyConfig::new("http://host.docker.internal:44000/").is_ok());
    for endpoint in [
        "http://host.docker.internal:44000",
        "https://host.docker.internal:44000/",
        "http://host.docker.internal:8787/",
        "http://token@host.docker.internal:44000/",
        "http://host.docker.internal:44000/?token=secret",
        "http://host.docker.internal:44000/#secret",
        "http://host.docker.internal:44000/v1",
    ] {
        assert!(
            CredentialProxyConfig::new(endpoint).is_err(),
            "{endpoint} must not be accepted"
        );
    }
}
