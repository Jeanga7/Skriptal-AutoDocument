# Skriptal-AutoDocument

AutoDocument est une plateforme innovante qui automatise la création de documents professionnels grâce à l'intelligence artificielle.

## Who to run?

📌 1 Lancer les conteneurs (le postgre est lancé avec docker)

Exécute :

```
cd backend
docker compose up --build
```

Installation docker de Postgres
`docker run --name my-postgres -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres`

Lancement du conteneur
`docker start my-postgres`

Pour exécuter psql via Docker :
`docker exec -it my-postgres psql -U postgres`
