# The resulting locus/egress-proxy image contains only this repository's static binary.
# The builder image is not a runtime dependency.
FROM rust:1.85-alpine AS build
WORKDIR /src
COPY main.rs .
RUN rustc -O -C target-feature=+crt-static main.rs -o /egress-proxy

FROM scratch
COPY --from=build /egress-proxy /egress-proxy
USER 65532:65532
EXPOSE 3128
ENTRYPOINT ["/egress-proxy"]
