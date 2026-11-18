# API Axum : Auth JWT + Notes

API REST en Rust (axum, sqlx, PostgreSQL) avec inscription, connexion par JWT, gestion des rôles (user/admin) et notes privées par utilisateur.

## Prérequis

- [Rust](https://rustup.rs) (édition 2021)
- PostgreSQL

## Installation

```bash
git clone https://github.com/TON_USER/TON_REPO.git
cd TON_REPO
cp .env.exemples .env
```

Éditer `.env` :

```env
DATABASE_URL=postgres://auth_user:auth_pass@localhost:5432/auth_db
JWT_SECRET=une_longue_chaine_aleatoire_de_32_caracteres_minimum
```

Générer un secret : `openssl rand -base64 48`

## Base de données

```sql
CREATE USER auth_user WITH PASSWORD 'auth_pass';
CREATE DATABASE auth_db OWNER auth_user;
```

Les tables `users` et `notes` sont créées automatiquement au démarrage.

## Lancer

```bash
cargo run
```

Le serveur écoute sur `http://localhost:3000`.

## Routes

| Méthode | Route | Auth | Description |
|---------|-------|------|-------------|
| POST | `/register` | non | Créer un compte |
| POST | `/login` | non | Obtenir un token JWT |
| GET | `/me` | oui | Profil de l'utilisateur connecté |
| GET | `/admin/users` | admin | Liste de tous les utilisateurs |
| POST | `/notes` | oui | Créer une note |
| GET | `/notes` | oui | Lister ses notes |
| DELETE | `/notes/{id}` | oui | Supprimer une de ses notes |

L'authentification se fait via le header `Authorization: Bearer <token>`.

## Tester avec curl

**Inscription** (attendu : `201`)

```bash
curl -i -X POST http://localhost:3000/register \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@test.com","password":"motdepasse123"}'
```

**Connexion** (attendu : `200` + `{"token":"..."}`)

```bash
curl -s -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@test.com","password":"motdepasse123"}'
```

Stocker le token :

```bash
TOKEN="colle_le_token_ici"
```

**Profil** (attendu : `200`)

```bash
curl -i http://localhost:3000/me -H "Authorization: Bearer $TOKEN"
```

**Créer une note** (attendu : `201`)

```bash
curl -i -X POST http://localhost:3000/notes \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Ma note","content":"Contenu"}'
```

**Lister ses notes** (attendu : `200`)

```bash
curl -i http://localhost:3000/notes -H "Authorization: Bearer $TOKEN"
```

**Supprimer une note** (attendu : `204`)

```bash
curl -i -X DELETE http://localhost:3000/notes/1 -H "Authorization: Bearer $TOKEN"
```

### Cas d'erreur

| Test | Code attendu |
|------|--------------|
| Email déjà utilisé à l'inscription | `409` |
| Mot de passe < 8 caractères ou email invalide | `400` |
| Mauvais mot de passe au login | `401` |
| Route protégée sans token | `401` |
| `/admin/users` avec un compte `user` | `403` |
| Supprimer une note inexistante ou d'un autre user | `404` |

### Tester la route admin

Passer un compte en admin directement en base :

```bash
psql -U auth_user -d auth_db -c "UPDATE users SET role='admin' WHERE email='alice@test.com';"
```

Puis se reconnecter (le rôle est inclus dans le token) et appeler :

```bash
curl -i http://localhost:3000/admin/users -H "Authorization: Bearer $NOUVEAU_TOKEN"
```

## Structure

```
src/
├── main.rs       # démarrage, routes, création des tables
├── handlers.rs   # register, login, me, list_users
├── notes.rs      # CRUD des notes
├── auth.rs       # extracteurs AuthUser / AdminUser
├── jwt.rs        # création et validation des tokens
├── password.rs   # hash Argon2
├── models.rs     # structures de données
├── error.rs      # erreurs et codes HTTP
└── state.rs      # état partagé (pool DB, secret JWT)
```

## Sécurité

- Mots de passe hashés avec Argon2
- Réponse identique pour email inconnu / mauvais mot de passe (anti-énumération)
- Hash factice pour limiter les attaques par timing
- Requêtes SQL paramétrées (pas d'injection SQL)
- Chaque note est filtrée par `user_id` : un utilisateur ne peut pas lire ou supprimer les notes d'un autre

## Auteur

@prsadamm
