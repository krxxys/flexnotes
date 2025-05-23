# Flexnotes 
Flexnotes is note taking app backend created for educational purposes to learn writting backend in rust. Flexnotes build on top of axum and mongodb(to store notes, todos and users). There's a gui client written in vue but you can write one yourself! Feel free to write your own flexnotes client :))

# Project milestones
1. [X] JWT Auth 
2. [X] Login/Register system
3. [X] REST Notes system with todos nested 
5. [ ] Implement the swagger like documentation
6. [ ] Major refactoring 
7. [ ] Implement logger
    
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
Have Rust, Mongodb(if you want the local db, you can use cloud option)
```bash
git clone https://github.com/krxxys/flexnotes.git &&
cd flexnotes &&
cargo run
```
# Flexnotes client
Check: https://github.com/krxxys/flexnotes-client

