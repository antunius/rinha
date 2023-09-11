# Etapa de compilação
FROM rust:latest as build
WORKDIR /app
COPY . .
COPY .env .env
RUN cargo build --release

# Etapa de produção
FROM debian:latest
WORKDIR /app
COPY .env /app/rinha
COPY --from=build /app/target/release/rinha /app/rinha
CMD ["./rinha"]
EXPOSE 80