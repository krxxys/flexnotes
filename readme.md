# API Endpoints

| **Method** | **Path**               | **Description**                     | **Input**                                                                 | **Output**                                                                 | **Errors**                                                                 |
|------------|------------------------|-------------------------------------|---------------------------------------------------------------------------|----------------------------------------------------------------------------|----------------------------------------------------------------------------|
| `POST`     | `/auth/login`          | Authenticates a user                | `{ "username": "string", "password": "string" }`                          | `{ "acces_token": "string", "token_type": "string", "username": "string" }`| `400`: Missing credentials, `401`: Wrong credentials, `500`: Token error   |
| `POST`     | `/auth/register`       | Registers a new user                | `{ "username": "string", "email": "string", "password": "string" }`       | `{ "acces_token": "string", "token_type": "string", "username": "string" }`| `400`: Missing credentials, `409`: User exists, `500`: Token/database error|
| `GET`      | `/auth/check`          | Validates JWT token                 | `Authorization: Bearer <JWT_TOKEN>`                                       | `200 OK`                                                                   | `401`: Invalid/expired token                                              |
| `POST`     | `/notes/create`        | Creates a new note                  | `{ "title": "string", "content": "string", "tags": ["string"] }`          | `200 OK`                                                                   | `401`: Invalid token, `500`: Database error                               |
| `GET`      | `/notes/`              | Retrieves all notes for the user    | `Authorization: Bearer <JWT_TOKEN>`                                       | `[{ "title": "string", "id": "ObjectId", "tags": ["string"] }]`            | `401`: Invalid token, `500`: Database error                               |
| `GET`      | `/notes/id/{id}`       | Retrieves a note by ID              | `Authorization: Bearer <JWT_TOKEN>`                                       | `{ "id": "ObjectId", "client_id": "ObjectId", "title": "string", ... }`    | `401`: Invalid token, `404`: Note not found, `500`: Database error         |
| `PATCH`    | `/notes/id/{id}`       | Updates a note by ID                | `{ "title": "string", "content": "string", "tags": ["string"] }`          | Updated note object                                                        | `401`: Invalid token, `404`: Note not found, `500`: Database error         |
| `DELETE`   | `/notes/id/{id}`       | Deletes a note by ID                | `Authorization: Bearer <JWT_TOKEN>`                                       | `200 OK`                                                                   | `401`: Invalid token, `404`: Note not found, `500`: Database error         |

---

### Notes:
- **Authentication**: All endpoints except `/auth/login` and `/auth/register` require a valid JWT token in the `Authorization` header.
- **Error Responses**: All errors return a JSON object with `error` and `code` fields.
- **Path Parameters**: `{id}` refers to a MongoDB `ObjectId`.