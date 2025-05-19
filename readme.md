## 📘 Documentation de l'API – Gestion des exécutions

### 🌐 URL de base

```
http://127.0.0.1:8080
```
---

### Variables d'environnement 
```
RUST_LOG=info
TEMPORAL_URL= "http://localhost:7233"
SERVER_URL="127.0.0.1:8080"
```
---

### ▶️ 1. Créer une nouvelle exécution

**Méthode :** `POST`
**Route :** `/executions`
**Corps :** *Aucun*

**Description :**
Crée une nouvelle exécution en démarrant un workflow Temporal. L’ID, le workflow ID, et le run ID sont générés automatiquement.

**Réponses :**

* `200 OK` – Retourne l'objet `Execution` créé (au format JSON).
* `500 Internal Server Error` – Échec lors du démarrage du workflow ou de la création dans la base.

**Exemple de réponse réussie :**

```json
{
  "id": "uuid",
  "workflow_id": "workflow-id",
  "run_id": "run-id",
  "status": "RUNNING"
}
```

---

### 🔍 2. Récupérer une exécution par ID

**Méthode :** `GET`
**Route :** `/executions/{id}`
**Paramètres :**

* `id` : UUID de l'exécution

**Description :**
Récupère une exécution spécifique à partir de son identifiant unique.

**Réponses :**

* `200 OK` – Retourne l'objet `Execution`.
* `404 Not Found` – Aucune exécution trouvée avec cet ID.
* `500 Internal Server Error` – Erreur de récupération.

---

### 🗑️ 3. Supprimer une exécution

**Méthode :** `DELETE`
**Route :** `/executions/{id}`
**Paramètres :**

* `id` : UUID de l'exécution

**Description :**
Supprime l'exécution identifiée par l'UUID donné.

**Réponses :**

* `200 OK` – Exécution supprimée.
* `404 Not Found` – Exécution non trouvée.
* `500 Internal Server Error` – Erreur lors de la suppression.

---

### 📋 4. Lister toutes les exécutions

**Méthode :** `GET`
**Route :** `/executions`
**Paramètres :** Aucun

**Description :**
Retourne la liste de toutes les exécutions présentes dans la base de données.

**Réponses :**

* `200 OK` – Liste d'exécutions en JSON.
* `500 Internal Server Error` – Échec de la récupération.

**Exemple de réponse :**

```json
[
  {
    "id": "uuid",
    "workflow_id": "workflow-id",
    "run_id": "run-id",
    "status": "RUNNING"
  },
  ...
]
```

---

### 🧪 Conseils pour les tests

* Le champ `status` est défini automatiquement à `"RUNNING"` à la création.
* Vous pouvez appeler le GET juste après le POST pour vérifier la création.
* Supprimer une exécution ne l'annule pas côté Temporal — uniquement en base.
* Le scheduler (si actif) mettra à jour le `status` périodiquement si l'exécution est encore en cours.

---
