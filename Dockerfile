FROM rust:1.77-buster

USER root
WORKDIR /src
COPY . /src
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get -y update
RUN apt-get -y install gcc-multilib openssl libssl-dev
RUN cargo build --bin noop-client --release --color never
RUN mv /src/target/release/noop-client /bin/noop-client
RUN chmod 755 /bin/noop-client
RUN rm -rf /src

# Endpoint will be ignored if the '--script' flag is provided.
ENTRYPOINT ["/bin/noop-client"]
CMD ["--endpoint=https://www.example.com/"]
