# Flexnotes 
Flexnotes is note taking app for my educational purposes and learning. Backend is written on top of axum and mongodb.  
# Project milestones
```
[X] JWT Auth 
[X] Login/Register system
[X] REST Notes system that store note markdown 
[X] REST Todo list system
[X] Implement dockers for production
[X] Implement logger (request logs into db, errors into file logs)
[ ] Implement the swagger like documentation
```
# API Examples 
There's already a Insomnia HTTP Client data with the api demonstration 
## Authentication Routes (`/auth`)

| Path             | Method | Input Data                                              | Output Data                                                  |
| ---------------- | ------ | ------------------------------------------------------- | ------------------------------------------------------------ |
| `/auth/login`    | POST   | `{ username: String, password: String }`                | `{ token: String, refresh_token: String, username: String }` |
| `/auth/register` | POST   | `{ username: String, email: String, password: String }` | `{ token: String, refresh_token: String, username: String }` |
| `/auth/refresh`  | POST   | `{ refresh_token: String }`                             | `{ acces_token: String, refresh_token: String }`             |
| `/auth/check`    | GET    | JWT in Authorization header                             | HTTP 202 (ACCEPTED)                                          |

## Notes Routes (`/notes`)

| Path             | Method | Input Data                                                                       | Output Data                                               |
| ---------------- | ------ | -------------------------------------------------------------------------------- | --------------------------------------------------------- |
| `/notes/create`  | POST   | `{ title: String, content: String, tags: Vec<String> }`                          | `ObjectId` (created note ID)                              |
| `/notes/`        | GET    | None (uses JWT)                                                                  | `Vec<{ title: String, id: ObjectId, tags: Vec<String> }>` |
| `/notes/id/{id}` | GET    | `id: ObjectId` in path                                                           | `NoteInfo` (full note details)                            |
| `/notes/id/{id}` | PATCH  | `id: ObjectId` in path + `{ title: String, content: String, tags: Vec<String> }` | Updated `NoteInfo`                                        |
| `/notes/id/{id}` | DELETE | `id: ObjectId` in path                                                           | HTTP Status Code                                          |

## Todo Routes (Nested under `/notes`)

| Path                             | Method | Input Data                                                                                   | Output Data      |
| -------------------------------- | ------ | -------------------------------------------------------------------------------------------- | ---------------- |
| `/notes/id/{id}/todos`           | GET    | `id: ObjectId` (note ID) in path                                                             | `Vec<TodoInfo>`  |
| `/notes/id/{id}/todos`           | POST   | `id: ObjectId` (note ID) in path + `{ title: String, status: bool, priority: TodoPriority }` | HTTP Status Code |
| `/notes/id/{id}/todos/{todo_id}` | PATCH  | `(id: ObjectId, todo_id: ObjectId)` in path + Todo payload                                   | HTTP Status Code |
| `/notes/id/{id}/todos/{todo_id}` | DELETE | `(id: ObjectId, todo_id: ObjectId)` in path                                                  | HTTP Status Code |


1. **Authentication**: All routes except `/auth/*` require JWT in `Authorization` header
2. **Path Parameters**:
   - `{id}` = Note's ObjectId
   - `{todo_id}` = Todo's ObjectId
3. **Common Headers**:
   - `Authorization: Bearer <JWT>` for protected routes
   - `Content-Type: application/json` for request bodies
4. **CORS**: Configured to allow requests from `http://localhost:5173`



# Installation
1. Clone the repo 
```bash
git clone https://github.com/krxxys/flexnotes.git && cd flexnotes
```
2. Create .env (for the local) and .env.production (for the docker backend if u want to use docker) files based on the .env.example
3. To run localy you need local mongodb server or you can use mongo from the docker but set the good ports
```bash 
# To run locally 
Cargo run --release
# or 
Cargo build --release 
cd target/release/
chmod +x ./flexnotes 
./flexnotes
```
```bash 
# To run inside dockers 
docker compose build 
docker compose up 
```
# Flexnotes client
Check: https://github.com/krxxys/flexnotes-client

