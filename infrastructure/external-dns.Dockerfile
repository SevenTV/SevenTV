FROM golang:latest as builder

WORKDIR /build

RUN git clone https://github.com/TroyKomodo/external-dns -b troy/fix-external-ip

RUN cd external-dns && go build -o /build/external-dns

FROM bitnami/external-dns

USER root

COPY --from=builder /build/external-dns /opt/bitnami/external-dns/bin/

RUN chmod +x /opt/bitnami/external-dns/bin/external-dns

USER 1001
