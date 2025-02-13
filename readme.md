# API Documentation

## Authentication

| Method | Endpoint        | Description              | Request Body | Response Body |
|--------|----------------|--------------------------|--------------|---------------|
| POST   | /auth/login    | User login               | `{ "username": "string", "password": "string" }` | `{ "access_token": "string", "refresh_token": "string", "username": "string" }` |
| POST   | /auth/register | User registration        | `{ "username": "string", "email": "string", "password": "string" }` | `{ "access_token": "string", "refresh_token": "string", "username": "string" }` |
| POST   | /auth/refresh  | Refresh access token     | `{ "refresh_token": "string" }` | `{ "access_token": "string", "refresh_token": "string" }` |

## Notes

| Method | Endpoint        | Description              | Request Body | Response Body |
|--------|----------------|--------------------------|--------------|---------------|
| POST   | /notes/create  | Create a new note        | `{ "title": "string", "content": "string", "tags": ["string"] }` | `{ "id": "ObjectId" }` |
| GET    | /notes/        | Get all user notes       | - | `[ { "title": "string", "id": "ObjectId", "tags": ["string"] } ]` |
| GET    | /notes/id/{id} | Get note by ID           | - | `{ "title": "string", "content": "string", "tags": ["string"] }` |
| PATCH  | /notes/id/{id} | Update a note            | `{ "title": "string", "content": "string", "tags": ["string"] }` | `{ "title": "string", "content": "string", "tags": ["string"] }` |
| DELETE | /notes/id/{id} | Delete a note            | - | `204 No Content` |
