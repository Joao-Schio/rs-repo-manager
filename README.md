# rs-repo-manager

[![Cargo test](https://github.com/Joao-Schio/rs-repo-manager/actions/workflows/test.yml/badge.svg)](https://github.com/Joao-Schio/rs-repo-manager/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/Joao-Schio/rs-repo-manager/branch/main/graph/badge.svg)](https://codecov.io/gh/Joao-Schio/rs-repo-manager)

A simple project for managing Docker-based repositories running on my Raspberry Pi.

I could have written this application in almost any language — and probably would have been more productive using C# or Python — but it had been a while since I last worked on a Rust project, so I decided to use this as an opportunity to get back into the language.

## The problem

My current deployment workflow is simple, but annoying.

Whenever I push changes to one of my repositories, I need to manually SSH into my Raspberry Pi and run:

```text
git pull
docker compose down
docker compose up -d --build
```

That works reasonably well when I finish developing one feature and call it a day.

It becomes more disruptive when, for example, I fix something important on the `main` branch, push it, and then want to immediately continue developing another feature. In that situation I have to interrupt my coding session just to SSH into the Pi, deploy the change, and then return to development.

The goal of `rs-repo-manager` is to automate that deployment step.

## Why not just use a CI/CD pipeline?

That's a fair question.

I prefer keeping conservative firewall rules on my Raspberry Pi, with services such as SSH only accessible from my local network, rather than exposing the Pi to the internet just so an external CI/CD system can trigger deployments.

Instead, `rs-repo-manager` is intended to run locally on the Raspberry Pi, eventually through something such as `cron`.

The Pi checks the repositories itself, and no inbound deployment endpoint is required.

## First MVP milestone

The first MVP milestone is complete.

The application currently:

1. Receives the path to a JSON configuration file through a command-line argument.
2. Parses the configuration.
3. Checks each configured repository.
4. Runs `git pull`.
5. Detects whether the repository actually changed.
6. Skips deployment if nothing changed.
7. Executes the configured deployment sequence when changes are detected.
8. Continues processing other repositories even if one fails.
9. Reports repository failures after execution.

## Configuration

Example:

```json
{
  "repositories": [
    {
      "directory": "/home/user/my-service",
      "compose_down": true
    }
  ]
}
```

This configuration manages the repository located at:

```text
/home/user/my-service
```

When an update is detected, the current deployment flow is:

```text
git pull
docker compose down
docker compose up -d --build
```

Internally, the application compares the repository's Git `HEAD` before and after `git pull`.

If the commit does not change, Docker Compose is not executed for that repository and the manager continues to the next configured repository.

If a command fails, the failure is recorded and execution continues with the remaining repositories.

## Usage

```bash
rs-repo-manager /path/to/config.json
```

For example:

```bash
rs-repo-manager ~/.config/rs-repo-manager/config.json
```

The application returns a non-zero exit code when argument parsing, configuration loading, or repository execution fails, making it suitable for use with tools such as `cron`.

## Next steps

### Prevent concurrent executions

Add a locking mechanism so that two instances of `rs-repo-manager` cannot deploy the same repository at the same time.

This will become especially important once the application is executed automatically by `cron`.

### More configurable Docker Compose commands

The Docker Compose commands are currently fixed as:

```bash
docker compose down
docker compose up -d --build
```

There are situations where additional arguments may be useful, for example:

```bash
docker compose down -v
```

or:

```bash
docker compose up -d --build --scale service=2
```

A future version should allow these behaviors to be configured per repository rather than hard-coded into the execution model.
