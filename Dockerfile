FROM rust:1.80 AS builder

# Install musl
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*

# Write target rust arch to file
ARG TARGETARCH
RUN 	if [ $TARGETARCH = "amd64" ]; then \
		echo 'x86_64-unknown-linux-musl' > /arch.txt; \
	elif [ $TARGETARCH = "arm64" ]; then \
		echo 'aarch64-unknown-linux-musl' > /arch.txt; \
	else \
		echo "Invalid target arch"; \
		exit 1; \
    fi
RUN rustup target add $(cat /arch.txt)
WORKDIR /usr/src/garage-door-rust/
COPY . .
RUN cargo install --path . --target $(cat /arch.txt)

FROM scratch
COPY --from=builder /usr/local/cargo/bin/garage-door-rust /app.bin
#COPY --from=builder /usr/local/cargo/bin/garage-door-rust /usr/local/bin/garage-door-rust
ENTRYPOINT ["/app.bin"]
