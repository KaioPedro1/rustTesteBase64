# Use the official Rust image as the base image
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Build the dependencies separately to take advantage of Docker layer caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy the source code to the container
COPY src ./src

# Build the application
RUN cargo build --release

# Create a new stage for the final image
FROM debian:buster-slim

# Install system dependencies
RUN apt-get update && apt-get install -y openssl libssl1.1

# Set the working directory inside the container
WORKDIR /app

# Copy the built binary from the previous stage to the final image
COPY --from=builder /app/target/release/mockserver /app/mockserver

# Create the temp directory
RUN mkdir /app/temp

# Expose the port on which the Actix application listens
EXPOSE 8080

# Set the command to run your Actix web application
CMD ["/app/mockserver"]
