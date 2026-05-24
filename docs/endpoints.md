### endpoints
POST   /webhooks                               Créer un webhook
PATCH  /webhooks/:id                           Modifier un webhook
DELETE /webhooks/:id                           Supprimer un webhook
POST   /webhooks/:id                           Déclencher un événement

GET    /webhooks/:id/events                    Lister les événements
POST   /webhooks/:id/events/:eid/replay        Rejouer un événement

GET    /webhooks/:id/metrics                   Métriques

POST   /webhooks/:id/destinations              Ajouter une destination
PATCH  /webhooks/:id/destinations/:did         Modifier une destination
DELETE /webhooks/:id/destinations/:did         Supprimer une destination