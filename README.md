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

- POST `/register`

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

- POST `/login`

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
