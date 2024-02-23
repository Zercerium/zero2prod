# zero2prod

Thanks a lot to **Luca Palmieri** for his great book [**Zero To Production In Rust**](https://www.zero2prod.com/index.html). I really enjoyed reading and coding along with it.

This repository contains the code from the book along with changes i made because i wanted to use [axum](https://docs.rs/axum/latest/axum/) (instead of [Actix Web](https://actix.rs)) and [SeaORM](https://www.sea-ql.org/SeaORM/) (instead of raw [SQLx](https://github.com/launchbadge/sqlx))

## ToDo
- [ ] (skipped the Digital Ocean Deployment) Local Deployment on HomeLab triggered through GitHub
- [ ] revisit ConnectOptions in configuration.rs. There are no PgConnectOptions in SeaORM and ConnectOption from SeaORM don't allow you to use a builder like pattern. It only allow you to pass a connection String.