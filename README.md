# cron-poll-discord

## Getting started

Create *.env* file with :

```
DISCORD_TOKEN=<discord bot token>
DATABASE_URL=<pg connection url>
```

Start dev environment:

```
make dev-start
```

Run services :

```
# bot
make run-bot

# api
make run-api

# sender
make run-sender
```

Run tests :

```
# unit tests
make test
```

Run integration tests:

> You will need [venom](https://github.com/ovh/venom)

```
# integration tests
make integration-tests
```
