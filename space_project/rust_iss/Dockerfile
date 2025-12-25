FROM rust:1.83-slim as builder
WORKDIR /app

# Копируем сначала файлы описания зависимостей
COPY Cargo.toml Cargo.lock ./

# Создаем пустой проект-заглушку для кэширования зависимостей
RUN mkdir src && echo "fn main() {}" > src/main.rs
# Скачиваем зависимости (используя наш Cargo.lock!)
RUN cargo build --release
RUN rm -rf src

# Теперь копируем настоящий код
COPY . .

# Отключаем проверку БД
ENV SQLX_OFFLINE=true

# Собираем (теперь быстро, так как зависимости уже скачаны)
RUN cargo build --release

# Финальный образ
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust_iss /app/rust_iss
CMD ["/app/rust_iss"]
