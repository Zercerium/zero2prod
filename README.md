# zero2prod

Thanks a lot to **Luca Palmieri** for his great book [**Zero To Production In Rust**](https://www.zero2prod.com/index.html). I really enjoyed reading and coding along with it.

This repository contains the code from the book along with changes i made because i wanted to use [axum](https://docs.rs/axum/latest/axum/) (instead of [Actix Web](https://actix.rs)) and [SeaORM](https://www.sea-ql.org/SeaORM/) (instead of raw [SQLx](https://github.com/launchbadge/sqlx))

## ToDo

- [ ] (skipped the Digital Ocean Deployment) Local Deployment on HomeLab triggered through GitHub
  - [ ] zero downtime?
- [ ] revisit ConnectOptions in configuration.rs. There are no PgConnectOptions in SeaORM and ConnectOption from SeaORM don't allow you to use a builder like pattern. It only allow you to pass a connection String.
- [x] Cross Compilation for Alpine base image (even smaller bundle size and lower attack surface ^^)
  - [x] but also allow different platforms (amd64 & armv8)
- [ ] Chapter 7 Summary opportunities
  - [ ] What happens if a user tries to subscribe twice? Make sure that they receive two confirmation emails;
  - [ ] What happens if a user clicks on a confirmation link twice?
  - [ ] What happens if the subscription token is well-formatted but non-existent?
  - [ ] Add validation on the incoming token, we are currently passing the raw user input straight into a query (thanks sqlx for protecting us from SQL injections <3);
  - [ ] Use a proper templating solution for our emails (e.g.tera);
  - [ ] Anything that comes to your mind!
    - [ ] something goes wrong while email send api request
- [ ] config validation
- [x] testing if it works with mailersend
- [ ] Authentication middleware to bubble up the UnexpectedError, we need to access the state for our hmac secret, which is needed to sign the response. This isn't possible in a IntoResponse trait, cause there is no access to the state. see also [here](https://github.com/tokio-rs/axum/discussions/2272)
  - p. 454
- [x] try out strongly typed tower-sessions ([example](https://github.com/maxcountryman/tower-sessions/blob/main/examples/strongly-typed.rs)) (it was just the next chapter ^^;)
- [ ] p.495 implement that the seed admin can invite more admins
- [ ] seed first admin from env variable on first startup
- [ ] more startup logging
  - url, mode
- [ ] [axum-flash](https://github.com/davidpdrsn/axum-flash) doesn't work in safari, lets check it out

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

## E-Mail

- normally I don't want to include an external (for this tutorial) service so suggested postmark wouldn't be an option.
- cuttlefish & postal seems a bit overkill setup wise, also if in general they seem interesting
  - not the biggest ruby fan, performance wise
- would be interesting to test out libs like lettre, also if a lot of the functionality from an full powered solution like postmark, etc. will be lost.
- so back to mailproviders; Options:
  - Postmark
  - Mailgun
  - Mailjet
  - mailersend
  - sendgrid
- i will use mailersend (This selection was not based on any facts, just personal preferences)
