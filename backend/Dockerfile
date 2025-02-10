# Base image
FROM rust:latest

# Installer les dépendances système nécessaires
RUN apt-get update && apt-get install -y \
    libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Ajouter un utilisateur non-root
RUN useradd -m rustuser
WORKDIR /app

# Assurer que les fichiers copiés ont les bonnes permissions
COPY --chown=rustuser:rustuser Cargo.toml Cargo.lock ./

# Passer à l'utilisateur non-root
USER rustuser

# Récupérer les dépendances
RUN cargo fetch

# Copier le reste du code et construire
COPY --chown=rustuser:rustuser . .
RUN cargo build --release

# Exposer le port de l'application
EXPOSE 8080

CMD ["./target/release/backend"]
