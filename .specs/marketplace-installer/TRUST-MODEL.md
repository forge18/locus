# Marketplace trust model

## Hosting

The local registry is authoritative until a project explicitly configures a remote index. A remote index
is addressed by an HTTPS URL and is cached by its content digest; the cache is not an authority by itself.

## Pinning

Every installed tool is identified by `name@version`. The image key is the sorted set of those pins, so
identical allowlists share one baked image. Changing a pin rebuilds the image. Changing catalog prose does
not.

## Trust and selection

Locus uses **selection**, not a closed curation list: trusted publishers and verified signatures gate
admission, then usage data ranks tools in the local index. A manifest must verify its binary digest and
its install command must pass the declared verify command before the image is usable. The first-party
Workshop CLI integration remains `gh`; other tools enter through the trusted user-plugin path.

Credentials stay in the host keychain and are never copied into a manifest, image layer, or agent
context. The allowlist is enforced while baking, so an unlisted tool is absent from the image.
