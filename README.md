# ywt-server

![](./assets/linabell.png)

This repository contains the source code and documentation for the *ywt* server, which is the web server designed for *面向《电子电路与系统基础》的助教智能体系统*, a project of Challenge Cup at Tsinghua University.

*ywt* stands for **Y**our **W**eb **T**A. We aimed to build a LLM-based intelligent assistant system for the course *Fundamentals of Electronic Circuits and Systems* at Tsinghua University.

## Build

Install the [Rust](https://www.rust-lang.org/) toolchain first. Then, you can build with cargo:

```bash
cargo build --release
```

This will create an executable file in the `target/release` directory.

## Configuration & Deployment

You need to have [MongoDB](https://www.mongodb.com/) installed and running to start *ywt*. Then, create a configuration file in JSON format:

```json
{
    "bind_address": "localhost",
    "bind_port": 8080,
    "mongo_uri": "mongodb://localhost:27017",
    "mongo_db": "ywt_db"
}
```

Change the `bind_address`, `bind_port`, `mongo_uri`, and `mongo_db` fields to your desired values. The app will use default values as above if you don't provide them.

You need to set environment variable `YWT_SECRET`, which is used as the secret key for JWT signing. If you don't set it, the app will use a default value of `ywt_secret`.

Environment variable `RUST_LOG` is used to set the log level. You can set it to `info`, `debug`, or `error`. If you don't set it, the app will use a default value of `info`.

You can set the environment variables in your shell or in a `.env` file. The app will automatically load the environment variables from the `.env` file if it exists.

Suppose the binary excutable you built is `ywt`, you can start the server by:

```text
Usage: ywt [OPTIONS]

Options:
  -c, --config <FILE>  Path to the configuration file
  -h, --help           Print help
  -V, --version        Print version
```

The server will start listening on the specified address and port, and connect to the MongoDB instance specified in the configuration file. 

## APIs

Notice that APIs with [Authentication required] require a valid JWT token in the `Authorization` header. The token is obtained by logging in with the `/login` API. Example:

```text
Authorization: Bearer <token>
```

### POST `/register`

Request:

```json
{
    "username": "ywt",
    "email": "ywt@example.com",
    "password": "testpassword"
}
```

Response:

```json
{
    "created_at": "2025-03-30 23:49:27.224212194 +08:00"
}
```

### POST `/login`

Request:

```json
{
    "username": "ywt",
    "password": "testpassword"
}
```

Response:

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VybmFtZSI6Inl3dCIsImlhdCI6MTc0MzQwMDk4MywiZXhwIjoxNzQzNDQ0MTgzfQ.UDtzBfJ9cS60wkSWW0QUH9vw_4wnKizcuSE4ctTeuKs"
}
```

This returns a JWT with JSON payload `{"username": , "iat": , "exp": }`. The token is valid for 12 hours.

### GET `/profile` [Authentication required]

Response:

```json
{
    "email": "ywt@example.com",
    "created_at": "2025-03-30 23:49:27.224212194 +08:00"
}
```

### GET `/problem/<problem_id>` [Authentication required]

Response:

```json
{
    "type": ["type1", "type2"],
    "image": "<base64_image>"
}
```

This returns problem image with the given ID in base64 format.

### POST `/count` [Authentication required]

Request:

```json
{
    "type": ["type1", "type2"]
}
```

Response:

```json
{
    "status": "success"
}
```

This API is used to count the different types of "knowledge points" that students mention in conversations with LLM assistant.

### GET `/count` [Authentication required]

Response:

```json
{
    "status": "success"
}
```

LLM assistant will call this API every time it receives a message from students.

### GET `/send_email` [Authentication required]

Response:

```json
{
    "status": "success"
}
```

This operation is used to send an email to all students, containing the statstics of the conversation with LLM assistant. 