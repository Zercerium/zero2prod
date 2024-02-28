# zero2prod

Thanks a lot to **Luca Palmieri** for his great book [**Zero To Production In Rust**](https://www.zero2prod.com/index.html). I really enjoyed reading and coding along with it.

This repository contains the code from the book along with changes i made because i wanted to use [axum](https://docs.rs/axum/latest/axum/) (instead of [Actix Web](https://actix.rs)) and [SeaORM](https://www.sea-ql.org/SeaORM/) (instead of raw [SQLx](https://github.com/launchbadge/sqlx))

## ToDo
- [ ] (skipped the Digital Ocean Deployment) Local Deployment on HomeLab triggered through GitHub
- [ ] revisit ConnectOptions in configuration.rs. There are no PgConnectOptions in SeaORM and ConnectOption from SeaORM don't allow you to use a builder like pattern. It only allow you to pass a connection String.
- [x] Cross Compilation for Alpine base image (even smaller bundle size and lower attack surface ^^)
    - [x] but also allow different platforms (amd64 & armv8)

## Docker

### Build (for linux docker images)
supported TARGETPLATFORM: `linux/amd64`, `linux/arm64` \
own supported docker platform: `docker build -t zero2prod:musl-cross .`\
other supported docker platform : `docker build -t zero2prod:musl-cross --platform ${TARGETPLATFORM} .`\

pls replace, ${TARGETPLATFORM} with one of the supported targets and you can tag your image for example with `-t zero2prod:musl-cross-aarch64`

example for docker manifest:
```
docker manifest create \
{REPO}/{USER}/zero2prod:musl-cross \
--amend {REPO}/{USER}/zero2prod:musl-cross-amd64 \
--amend {REPO}/{USER}/zero2prod:musl-cross-arm64

docker manifest push {REPO}/{USER}/zero2prod:musl-cross
```