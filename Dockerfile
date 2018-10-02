FROM clux/muslrust:nightly AS builder

COPY . /root/
WORKDIR /root/
RUN update-ca-certificates \
  && cargo test \
  && cargo build --release --target x86_64-unknown-linux-musl \
  && groupadd -r -g 48 d2l-sync \
  && useradd -r -u 48 -g 48 -c 'D2L User Sync' -d /var/lib/d2l-sync d2l-sync \
  && mkdir -p /rootfs/etc/ssl /rootfs/bin/ /rootfs/var/lib/d2l-sync \
  && chown -R 48.48 /rootfs/var/lib/d2l-sync/ \
  && cp /etc/passwd /etc/group /rootfs/etc/ \
  && cp -r /etc/ssl/certs /rootfs/etc/ssl/ \
  && cp /root/target/x86_64-unknown-linux-musl/release/d2l-sync /rootfs/bin/

FROM scratch AS final
COPY --from=builder /rootfs/ /
USER d2l-sync
WORKDIR /var/lib/d2l-sync/
CMD ["/bin/d2l-sync"]
